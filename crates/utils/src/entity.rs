#[cfg(target_has_atomic = "64")]
use std::sync::atomic::AtomicI64 as AtomicIdCursor;
#[cfg(target_has_atomic = "64")]
type IdCursor = i64;

/// Most modern platforms support 64-bit atomics, but some less-common platforms
/// do not. This fallback allows compilation using a 32-bit cursor instead, with
/// the caveat that some conversions may fail (and panic) at runtime.
#[cfg(not(target_has_atomic = "64"))]
use std::sync::atomic::AtomicIsize as AtomicIdCursor;
use std::sync::atomic::Ordering;

use pi_key_alloter::{KeyData, Key};
use pi_world::world::Entity;
#[cfg(not(target_has_atomic = "64"))]
type IdCursor = isize;

/// A [`World`]'s internal metadata store on all of its entities.
///
/// Contains metadata on:
///  - The generation of every entity.
///  - The alive/dead status of a particular entity. (i.e. "has entity 3 been despawned?")
///  - The location of the entity's components in memory (via [`EntityLocation`])
///
/// [`World`]: crate::world::World
#[derive(Debug)]
pub struct Entities {
    meta: Vec<EntityMeta>,

    /// The `pending` and `free_cursor` fields describe three sets of Entity IDs
    /// that have been freed or are in the process of being allocated:
    ///
    /// - The `freelist` IDs, previously freed by `free()`. These IDs are available to any of
    ///   [`alloc`], [`reserve_entity`] or [`reserve_entities`]. Allocation will always prefer
    ///   these over brand new IDs.
    ///
    /// - The `reserved` list of IDs that were once in the freelist, but got reserved by
    ///   [`reserve_entities`] or [`reserve_entity`]. They are now waiting for [`flush`] to make them
    ///   fully allocated.
    ///
    /// - The count of new IDs that do not yet exist in `self.meta`, but which we have handed out
    ///   and reserved. [`flush`] will allocate room for them in `self.meta`.
    ///
    /// The contents of `pending` look like this:
    ///
    /// ```txt
    /// ----------------------------
    /// |  freelist  |  reserved   |
    /// ----------------------------
    ///              ^             ^
    ///          free_cursor   pending.len()
    /// ```
    ///
    /// As IDs are allocated, `free_cursor` is atomically decremented, moving
    /// items from the freelist into the reserved list by sliding over the boundary.
    ///
    /// Once the freelist runs out, `free_cursor` starts going negative.
    /// The more negative it is, the more IDs have been reserved starting exactly at
    /// the end of `meta.len()`.
    ///
    /// This formulation allows us to reserve any number of IDs first from the freelist
    /// and then from the new IDs, using only a single atomic subtract.
    ///
    /// Once [`flush`] is done, `free_cursor` will equal `pending.len()`.
    ///
    /// [`alloc`]: Entities::alloc
    /// [`reserve_entity`]: Entities::reserve_entity
    /// [`reserve_entities`]: Entities::reserve_entities
    /// [`flush`]: Entities::flush
    pending: Vec<u32>,
    free_cursor: AtomicIdCursor,
    /// Stores the number of free entities for [`len`](Entities::len)
    len: u32,
}

impl Entities {
    pub(crate) const fn new() -> Self {
        Entities {
            meta: Vec::new(),
            pending: Vec::new(),
            free_cursor: AtomicIdCursor::new(0),
            len: 0,
        }
    }

    /// Reserve entity IDs concurrently.
    ///
    /// Storage for entity generation and location is lazily allocated by calling [`flush`](Entities::flush).
    pub fn reserve_entities(&self, count: u32) -> ReserveEntitiesIterator {
        // Use one atomic subtract to grab a range of new IDs. The range might be
        // entirely nonnegative, meaning all IDs come from the freelist, or entirely
        // negative, meaning they are all new IDs to allocate, or a mix of both.
        let range_end = self
            .free_cursor
            // Unwrap: these conversions can only fail on platforms that don't support 64-bit atomics
            // and use AtomicIsize instead (see note on `IdCursor`).
            .fetch_sub(IdCursor::try_from(count).unwrap(), Ordering::Relaxed);
        let range_start = range_end - IdCursor::try_from(count).unwrap();

        let freelist_range = range_start.max(0) as usize..range_end.max(0) as usize;

        let (new_id_start, new_id_end) = if range_start >= 0 {
            // We satisfied all requests from the freelist.
            (0, 0)
        } else {
            // We need to allocate some new Entity IDs outside of the range of self.meta.
            //
            // `range_start` covers some negative territory, e.g. `-3..6`.
            // Since the nonnegative values `0..6` are handled by the freelist, that
            // means we need to handle the negative range here.
            //
            // In this example, we truncate the end to 0, leaving us with `-3..0`.
            // Then we negate these values to indicate how far beyond the end of `meta.end()`
            // to go, yielding `meta.len()+0 .. meta.len()+3`.
            let base = self.meta.len() as IdCursor;

            let new_id_end = u32::try_from(base - range_start).expect("too many entities");

            // `new_id_end` is in range, so no need to check `start`.
            let new_id_start = (base - range_end.min(0)) as u32;

            (new_id_start, new_id_end)
        };

        ReserveEntitiesIterator {
            meta: &self.meta[..],
            index_iter: self.pending[freelist_range].iter(),
            index_range: new_id_start..new_id_end,
        }
    }

    /// Reserve one entity ID concurrently.
    ///
    /// Equivalent to `self.reserve_entities(1).next().unwrap()`, but more efficient.
    pub fn reserve_entity(&self) -> Entity {
        let n = self.free_cursor.fetch_sub(1, Ordering::Relaxed);
        if n > 0 {
            // Allocate from the freelist.
            let index = self.pending[(n - 1) as usize];
            // Entity {
            //     generation: self.meta[index as usize].generation,
            //     index,
            // }
            Entity::from(KeyData::from_ffi((u64::from(self.meta[index as usize].generation) << 32) | u64::from(index)))
        } else {
            // Grab a new ID, outside the range of `meta.len()`. `flush()` must
            // eventually be called to make it valid.
            //
            // As `self.free_cursor` goes more and more negative, we return IDs farther
            // and farther beyond `meta.len()`.
            // Entity {
            //     generation: 0,
            //     index: u32::try_from(self.meta.len() as IdCursor - n).expect("too many entities"),
            // }
            Entity::from(KeyData::from_ffi((u64::from(0u32) << 32) | u64::try_from(self.meta.len() as IdCursor - n).expect("too many entities")))
        }
    }

    /// Check that we do not have pending work requiring `flush()` to be called.
    fn verify_flushed(&mut self) {
        debug_assert!(
            !self.needs_flush(),
            "flush() needs to be called before this operation is legal"
        );
    }

    /// Allocate an entity ID directly.
    pub fn alloc(&mut self) -> Entity {
        self.verify_flushed();
        self.len += 1;
        if let Some(index) = self.pending.pop() {
            let new_free_cursor = self.pending.len() as IdCursor;
            *self.free_cursor.get_mut() = new_free_cursor;
            Entity::from(KeyData::from_ffi((u64::from(self.meta[index as usize].generation) << 32) | u64::from(index)))
        } else {
            let index = u32::try_from(self.meta.len()).expect("too many entities");
            self.meta.push(EntityMeta::EMPTY);
            Entity::from(KeyData::from_ffi((u64::from(0u32) << 32) | u64::from(index)))
        }
    }

    /// Allocate a specific entity ID, overwriting its generation.
    ///
    /// Returns the location of the entity currently using the given ID, if any. Location should be
    /// written immediately.
    pub fn alloc_at(&mut self, entity: Entity) -> Option<EntityLocation> {
        self.verify_flushed();

        let loc = if entity.data().index() as usize >= self.meta.len() {
            self.pending.extend((self.meta.len() as u32)..entity.data().index());
            let new_free_cursor = self.pending.len() as IdCursor;
            *self.free_cursor.get_mut() = new_free_cursor;
            self.meta
                .resize(entity.data().index() as usize + 1, EntityMeta::EMPTY);
            self.len += 1;
            None
        } else if let Some(index) = self.pending.iter().position(|item| *item == entity.data().index()) {
            self.pending.swap_remove(index);
            let new_free_cursor = self.pending.len() as IdCursor;
            *self.free_cursor.get_mut() = new_free_cursor;
            self.len += 1;
            None
        } else {
            Some(std::mem::replace(
                &mut self.meta[entity.data().index() as usize].location,
                EntityMeta::EMPTY.location,
            ))
        };

        self.meta[entity.data().index() as usize].generation = entity.data().version();

        loc
    }

    /// Allocate a specific entity ID, overwriting its generation.
    ///
    /// Returns the location of the entity currently using the given ID, if any.
    pub(crate) fn alloc_at_without_replacement(
        &mut self,
        entity: Entity,
    ) -> AllocAtWithoutReplacement {
        self.verify_flushed();

        let result = if entity.index() >= self.meta.len() {
            self.pending.extend((self.meta.len() as u32)..entity.index() as u32);
            let new_free_cursor = self.pending.len() as IdCursor;
            *self.free_cursor.get_mut() = new_free_cursor;
            self.meta
                .resize(entity.index() + 1, EntityMeta::EMPTY);
            self.len += 1;
            AllocAtWithoutReplacement::DidNotExist
        } else if let Some(index) = self.pending.iter().position(|item| *item == entity.data().index()) {
            self.pending.swap_remove(index);
            let new_free_cursor = self.pending.len() as IdCursor;
            *self.free_cursor.get_mut() = new_free_cursor;
            self.len += 1;
            AllocAtWithoutReplacement::DidNotExist
        } else {
            let current_meta = &self.meta[entity.index()];
            if current_meta.location.archetype_id == ArchetypeId::INVALID {
                AllocAtWithoutReplacement::DidNotExist
            } else if current_meta.generation == entity.data().version() {
                AllocAtWithoutReplacement::Exists(current_meta.location)
            } else {
                return AllocAtWithoutReplacement::ExistsWithWrongGeneration;
            }
        };

        self.meta[entity.index()].generation = entity.data().version();
        result
    }

    /// Destroy an entity, allowing it to be reused.
    ///
    /// Must not be called while reserved entities are awaiting `flush()`.
    pub fn free(&mut self, entity: Entity) -> Option<EntityLocation> {
        self.verify_flushed();

        let meta = &mut self.meta[entity.index()];
        if meta.generation != entity.data().version() {
            return None;
        }
        meta.generation += 1;

        let loc = std::mem::replace(&mut meta.location, EntityMeta::EMPTY.location);

        self.pending.push(entity.data().index());

        let new_free_cursor = self.pending.len() as IdCursor;
        *self.free_cursor.get_mut() = new_free_cursor;
        self.len -= 1;
        Some(loc)
    }

    /// Ensure at least `n` allocations can succeed without reallocating.
    pub fn reserve(&mut self, additional: u32) {
        self.verify_flushed();

        let freelist_size = *self.free_cursor.get_mut();
        // Unwrap: these conversions can only fail on platforms that don't support 64-bit atomics
        // and use AtomicIsize instead (see note on `IdCursor`).
        let shortfall = IdCursor::try_from(additional).unwrap() - freelist_size;
        if shortfall > 0 {
            self.meta.reserve(shortfall as usize);
        }
    }

    /// Returns true if the [`Entities`] contains [`entity`](Entity).
    // This will return false for entities which have been freed, even if
    // not reallocated since the generation is incremented in `free`
    pub fn contains(&self, entity: Entity) -> bool {
        self.resolve_from_id(entity.data().index())
            .map_or(false, |e| e.data().version() == entity.data().version())
    }

    /// Clears all [`Entity`] from the World.
    pub fn clear(&mut self) {
        self.meta.clear();
        self.pending.clear();
        *self.free_cursor.get_mut() = 0;
        self.len = 0;
    }

    /// Returns the location of an [`Entity`].
    /// Note: for pending entities, returns `Some(EntityLocation::INVALID)`.
    #[inline]
    pub fn get(&self, entity: Entity) -> Option<EntityLocation> {
        if let Some(meta) = self.meta.get(entity.index()) {
            if meta.generation != entity.data().version()
                || meta.location.archetype_id == ArchetypeId::INVALID
            {
                return None;
            }
            Some(meta.location)
        } else {
            None
        }
    }

    /// Updates the location of an [`Entity`]. This must be called when moving the components of
    /// the entity around in storage.
    ///
    /// # Safety
    ///  - `index` must be a valid entity index.
    ///  - `location` must be valid for the entity at `index` or immediately made valid afterwards
    ///    before handing control to unknown code.
    #[inline]
    pub(crate) unsafe fn set(&mut self, index: u32, location: EntityLocation) {
        // SAFETY: Caller guarantees that `index` a valid entity index
        self.meta.get_unchecked_mut(index as usize).location = location;
    }

    /// Increments the `generation` of a freed [`Entity`]. The next entity ID allocated with this
    /// `index` will count `generation` starting from the prior `generation` + the specified
    /// value + 1.
    ///
    /// Does nothing if no entity with this `index` has been allocated yet.
    pub(crate) fn reserve_generations(&mut self, index: u32, generations: u32) -> bool {
        if (index as usize) >= self.meta.len() {
            return false;
        }

        let meta = &mut self.meta[index as usize];
        if meta.location.archetype_id == ArchetypeId::INVALID {
            meta.generation += generations;
            true
        } else {
            false
        }
    }

    /// Get the [`Entity`] with a given id, if it exists in this [`Entities`] collection
    /// Returns `None` if this [`Entity`] is outside of the range of currently reserved Entities
    ///
    /// Note: This method may return [`Entities`](Entity) which are currently free
    /// Note that [`contains`](Entities::contains) will correctly return false for freed
    /// entities, since it checks the generation
    pub fn resolve_from_id(&self, index: u32) -> Option<Entity> {
        let idu = index as usize;
        if let Some(&EntityMeta { generation, .. }) = self.meta.get(idu) {
            Some(Entity::from(KeyData::from_ffi(
                (u64::from(generation) << 32) | u64::from(index),
            )))
        } else {
            // `id` is outside of the meta list - check whether it is reserved but not yet flushed.
            let free_cursor = self.free_cursor.load(Ordering::Relaxed);
            // If this entity was manually created, then free_cursor might be positive
            // Returning None handles that case correctly
            let num_pending = usize::try_from(-free_cursor).ok()?;
            (idu < self.meta.len() + num_pending).then_some(Entity::from(KeyData::from_ffi(
                (u64::from(0u32) << 32) | u64::from(index),
            )))
        }
    }

    fn needs_flush(&mut self) -> bool {
        *self.free_cursor.get_mut() != self.pending.len() as IdCursor
    }

    /// Allocates space for entities previously reserved with [`reserve_entity`](Entities::reserve_entity) or
    /// [`reserve_entities`](Entities::reserve_entities), then initializes each one using the supplied function.
    ///
    /// # Safety
    /// Flush _must_ set the entity location to the correct [`ArchetypeId`] for the given [`Entity`]
    /// each time init is called. This _can_ be [`ArchetypeId::INVALID`], provided the [`Entity`]
    /// has not been assigned to an [`Archetype`][crate::archetype::Archetype].
    ///
    /// Note: freshly-allocated entities (ones which don't come from the pending list) are guaranteed
    /// to be initialized with the invalid archetype.
    pub unsafe fn flush(&mut self, mut init: impl FnMut(Entity, &mut EntityLocation)) {
        let free_cursor = self.free_cursor.get_mut();
        let current_free_cursor = *free_cursor;

        let new_free_cursor = if current_free_cursor >= 0 {
            current_free_cursor as usize
        } else {
            let old_meta_len = self.meta.len();
            let new_meta_len = old_meta_len + -current_free_cursor as usize;
            self.meta.resize(new_meta_len, EntityMeta::EMPTY);
            self.len += -current_free_cursor as u32;
            for (index, meta) in self.meta.iter_mut().enumerate().skip(old_meta_len) {
                init(
                    Entity::from(KeyData::from_ffi(
                        (u64::from(meta.generation) << 32) | u64::from(index as u32),
                    )),
                    &mut meta.location,
                );
            }

            *free_cursor = 0;
            0
        };

        self.len += (self.pending.len() - new_free_cursor) as u32;
        for index in self.pending.drain(new_free_cursor..) {
            let meta = &mut self.meta[index as usize];
            init(
                Entity::from(KeyData::from_ffi(
                    (u64::from(meta.generation) << 32) | u64::from(index),
                )),
                &mut meta.location,
            );
        }
    }

    /// Flushes all reserved entities to an "invalid" state. Attempting to retrieve them will return `None`
    /// unless they are later populated with a valid archetype.
    pub fn flush_as_invalid(&mut self) {
        // SAFETY: as per `flush` safety docs, the archetype id can be set to [`ArchetypeId::INVALID`] if
        // the [`Entity`] has not been assigned to an [`Archetype`][crate::archetype::Archetype], which is the case here
        unsafe {
            self.flush(|_entity, location| {
                location.archetype_id = ArchetypeId::INVALID;
            });
        }
    }

    /// # Safety
    ///
    /// This function is safe if and only if the world this Entities is on has no entities.
    pub unsafe fn flush_and_reserve_invalid_assuming_no_entities(&mut self, count: usize) {
        let free_cursor = self.free_cursor.get_mut();
        *free_cursor = 0;
        self.meta.reserve(count);
        // the EntityMeta struct only contains integers, and it is valid to have all bytes set to u8::MAX
        self.meta.as_mut_ptr().write_bytes(u8::MAX, count);
        self.meta.set_len(count);

        self.len = count as u32;
    }

    /// The count of all entities in the [`World`] that have ever been allocated
    /// including the entities that are currently freed.
    ///
    /// This does not include entities that have been reserved but have never been
    /// allocated yet.
    ///
    /// [`World`]: crate::world::World
    #[inline]
    pub fn total_count(&self) -> usize {
        self.meta.len()
    }

    /// The count of currently allocated entities.
    #[inline]
    pub fn len(&self) -> u32 {
        self.len
    }

    /// Checks if any entity is currently active.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

// This type is repr(C) to ensure that the layout and values within it can be safe to fully fill
// with u8::MAX, as required by [`Entities::flush_and_reserve_invalid_assuming_no_entities`].
// Safety:
// This type must not contain any pointers at any level, and be safe to fully fill with u8::MAX.
/// Metadata for an [`Entity`].
#[derive(Copy, Clone, Debug)]
#[repr(C)]
struct EntityMeta {
    /// The current generation of the [`Entity`].
    pub generation: u32,
    /// The current location of the [`Entity`]
    pub location: EntityLocation,
}

impl EntityMeta {
    /// meta for **pending entity**
    const EMPTY: EntityMeta = EntityMeta {
        generation: 0,
        location: EntityLocation::INVALID,
    };
}

// This type is repr(C) to ensure that the layout and values within it can be safe to fully fill
// with u8::MAX, as required by [`Entities::flush_and_reserve_invalid_assuming_no_entities`].
// SAFETY:
// This type must not contain any pointers at any level, and be safe to fully fill with u8::MAX.
/// A location of an entity in an archetype.
#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(C)]
pub struct EntityLocation {
    /// The ID of the [`Archetype`] the [`Entity`] belongs to.
    ///
    /// [`Archetype`]: crate::archetype::Archetype
    pub archetype_id: ArchetypeId,

    /// The index of the [`Entity`] within its [`Archetype`].
    ///
    /// [`Archetype`]: crate::archetype::Archetype
    pub archetype_row: ArchetypeRow,

    /// The ID of the [`Table`] the [`Entity`] belongs to.
    ///
    /// [`Table`]: crate::storage::Table
    pub table_id: TableId,

    /// The index of the [`Entity`] within its [`Table`].
    ///
    /// [`Table`]: crate::storage::Table
    pub table_row: TableRow,
}

impl EntityLocation {
    /// location for **pending entity** and **invalid entity**
    const INVALID: EntityLocation = EntityLocation {
        archetype_id: ArchetypeId::INVALID,
        archetype_row: ArchetypeRow::INVALID,
        table_id: TableId::INVALID,
        table_row: TableRow::INVALID,
    };
}

/// An opaque unique ID for a [`Table`] within a [`World`].
///
/// Can be used with [`Tables::get`] to fetch the corresponding
/// table.
///
/// Each [`Archetype`] always points to a table via [`Archetype::table_id`].
/// Multiple archetypes can point to the same table so long as the components
/// stored in the table are identical, but do not share the same sparse set
/// components.
///
/// [`World`]: crate::world::World
/// [`Archetype`]: crate::archetype::Archetype
/// [`Archetype::table_id`]: crate::archetype::Archetype::table_id
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
// SAFETY: Must be repr(transparent) due to the safety requirements on EntityLocation
#[repr(transparent)]
pub struct TableId(u32);

impl TableId {
    pub(crate) const INVALID: TableId = TableId(u32::MAX);

    /// Creates a new [`TableId`].
    ///
    /// `index` *must* be retrieved from calling [`TableId::index`] on a `TableId` you got
    /// from a table of a given [`World`] or the created ID may be invalid.
    ///
    /// [`World`]: crate::world::World
    #[inline]
    pub fn new(index: usize) -> Self {
        TableId(index as u32)
    }

    /// Gets the underlying table index from the ID.
    #[inline]
    pub fn index(self) -> usize {
        self.0 as usize
    }

    /// The [`TableId`] of the [`Table`] without any components.
    #[inline]
    pub const fn empty() -> TableId {
        TableId(0)
    }
}

/// A opaque newtype for rows in [`Table`]s. Specifies a single row in a specific table.
///
/// Values of this type are retrievable from [`Archetype::entity_table_row`] and can be
/// used alongside [`Archetype::table_id`] to fetch the exact table and row where an
/// [`Entity`]'s
///
/// Values of this type are only valid so long as entities have not moved around.
/// Adding and removing components from an entity, or despawning it will invalidate
/// potentially any table row in the table the entity was previously stored in. Users
/// should *always* fetch the appropriate row from the entity's [`Archetype`] before
/// fetching the entity's components.
///
/// [`Archetype`]: crate::archetype::Archetype
/// [`Archetype::entity_table_row`]: crate::archetype::Archetype::entity_table_row
/// [`Archetype::table_id`]: crate::archetype::Archetype::table_id
/// [`Entity`]: crate::entity::Entity
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
// SAFETY: Must be repr(transparent) due to the safety requirements on EntityLocation
#[repr(transparent)]
pub struct TableRow(u32);

impl TableRow {
    pub(crate) const INVALID: TableRow = TableRow(u32::MAX);

    /// Creates a `TableRow`.
    #[inline]
    pub const fn new(index: usize) -> Self {
        Self(index as u32)
    }

    /// Gets the index of the row.
    #[inline]
    pub const fn index(self) -> usize {
        self.0 as usize
    }
}

/// An opaque unique ID for a single [`Archetype`] within a [`World`].
///
/// Archetype IDs are only valid for a given World, and are not globally unique.
/// Attempting to use an archetype ID on a world that it wasn't sourced from will
/// not return the archetype with the same components. The only exception to this is
/// [`EMPTY`] which is guaranteed to be identical for all Worlds.
///
/// [`World`]: crate::world::World
/// [`EMPTY`]: crate::archetype::ArchetypeId::EMPTY
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
// SAFETY: Must be repr(transparent) due to the safety requirements on EntityLocation
#[repr(transparent)]
pub struct ArchetypeId(u32);

impl ArchetypeId {
    /// The ID for the [`Archetype`] without any components.
    pub const EMPTY: ArchetypeId = ArchetypeId(0);
    /// # Safety:
    ///
    /// This must always have an all-1s bit pattern to ensure soundness in fast entity id space allocation.
    pub const INVALID: ArchetypeId = ArchetypeId(u32::MAX);

    #[inline]
    pub(crate) const fn new(index: usize) -> Self {
        ArchetypeId(index as u32)
    }

    #[inline]
    pub(crate) fn index(self) -> usize {
        self.0 as usize
    }
}

/// An opaque location within a [`Archetype`].
///
/// This can be used in conjunction with [`ArchetypeId`] to find the exact location
/// of an [`Entity`] within a [`World`]. An entity's archetype and index can be
/// retrieved via [`Entities::get`].
///
/// [`World`]: crate::world::World
/// [`Entities::get`]: crate::entity::Entities
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
// SAFETY: Must be repr(transparent) due to the safety requirements on EntityLocation
#[repr(transparent)]
pub struct ArchetypeRow(u32);

impl ArchetypeRow {
    /// Index indicating an invalid archetype row.
    /// This is meant to be used as a placeholder.
    pub const INVALID: ArchetypeRow = ArchetypeRow(u32::MAX);

    /// Creates a `ArchetypeRow`.
    #[inline]
    pub const fn new(index: usize) -> Self {
        Self(index as u32)
    }

    /// Gets the index of the row.
    #[inline]
    pub const fn index(self) -> usize {
        self.0 as usize
    }
}

/// An [`Iterator`] returning a sequence of [`Entity`] values from
/// [`Entities::reserve_entities`](crate::entity::Entities::reserve_entities).
pub struct ReserveEntitiesIterator<'a> {
    // Metas, so we can recover the current generation for anything in the freelist.
    meta: &'a [EntityMeta],

    // Reserved indices formerly in the freelist to hand out.
    index_iter: std::slice::Iter<'a, u32>,

    // New Entity indices to hand out, outside the range of meta.len().
    index_range: std::ops::Range<u32>,
}

impl<'a> Iterator for ReserveEntitiesIterator<'a> {
    type Item = Entity;

    fn next(&mut self) -> Option<Self::Item> {
        self.index_iter
            .next()
            .map(|&index| {
                Entity::from(KeyData::from_ffi(
                    (u64::from(self.meta[index as usize].generation) << 32) | u64::from(index),
                ))
            })
            .or_else(|| {
                self.index_range
                    .next()
                    .map(|index| Entity::from(KeyData::from_ffi((0 << 32) | u64::from(index))))
            })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.index_iter.len() + self.index_range.len();
        (len, Some(len))
    }
}

impl<'a> core::iter::ExactSizeIterator for ReserveEntitiesIterator<'a> {}
impl<'a> core::iter::FusedIterator for ReserveEntitiesIterator<'a> {}

pub(crate) enum AllocAtWithoutReplacement {
    Exists(EntityLocation),
    DidNotExist,
    ExistsWithWrongGeneration,
}