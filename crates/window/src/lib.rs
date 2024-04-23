#![allow(clippy::type_complexity)]

#[warn(missing_docs)]
mod cursor;
mod event;
mod raw_handle;
mod system;
mod window;

pub use crate::raw_handle::*;

// use bevy_ecs::{event::{Event, Events}, system::{Resource, Res}, schedule::{IntoSystemConfigs, SystemSet}};
pub use cursor::*;
pub use event::*;
use pi_world::{prelude::App, single_res::SingleRes};
pub use system::*;
pub use window::*;

pub mod prelude {
    #[doc(hidden)]
    pub use crate::{
        CursorEntered, CursorIcon, CursorLeft, CursorMoved, FileDragAndDrop, Ime, MonitorSelection,
        ReceivedCharacter, Window, WindowMoved, WindowPlugin, WindowPosition,
        WindowResizeConstraints,
    };
}

// use bevy_app::prelude::*;
// use std::path::PathBuf;

impl Default for WindowPlugin {
    fn default() -> Self {
        WindowPlugin {
            primary_window: Some(Window::default()),
            exit_condition: ExitCondition::OnAllClosed,
            close_when_requested: true,
        }
    }
}

/// A [`Plugin`] that defines an interface for windowing support in Bevy.
pub struct WindowPlugin {
    /// Settings for the primary window.
    ///
    /// `Some(custom_window)` will spawn an entity with `custom_window` and [`PrimaryWindow`] as components.
    /// `None` will not spawn a primary window.
    ///
    /// Defaults to `Some(Window::default())`.
    ///
    /// Note that if there are no windows the App will exit (by default) due to
    /// [`exit_on_all_closed`].
    pub primary_window: Option<Window>,

    /// Whether to exit the app when there are no open windows.
    ///
    /// If disabling this, ensure that you send the [`bevy_app::AppExit`]
    /// event when the app should exit. If this does not occur, you will
    /// create 'headless' processes (processes without windows), which may
    /// surprise your users. It is recommended to leave this setting to
    /// either [`ExitCondition::OnAllClosed`] or [`ExitCondition::OnPrimaryClosed`].
    ///
    /// [`ExitCondition::OnAllClosed`] will add [`exit_on_all_closed`] to [`Update`].
    /// [`ExitCondition::OnPrimaryClosed`] will add [`exit_on_primary_closed`] to [`Update`].
    pub exit_condition: ExitCondition,

    /// Whether to close windows when they are requested to be closed (i.e.
    /// when the close button is pressed).
    ///
    /// If true, this plugin will add [`close_when_requested`] to [`Update`].
    /// If this system (or a replacement) is not running, the close button will have no effect.
    /// This may surprise your users. It is recommended to leave this setting as `true`.
    pub close_when_requested: bool,
}

#[derive(Debug, Default, Clone, Copy)]
pub enum FrameState {
    #[default]
    Active,
    UnActive,
}

pub fn should_run(state: SingleRes<FrameState>) -> bool {
    if let FrameState::Active = *state {
        true
    } else {
        false
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct FrameSet;

// pub trait AddFrameEvent {
//     // 添加事件， 该实现每帧清理一次
//     fn add_frame_event<T: Event>(&mut self) -> &mut Self;
// }

// impl AddFrameEvent for App {
//     fn add_frame_event<T: Event>(&mut self) -> &mut Self {
//         if !self.world.contains_resource::<Events<T>>() {
//             self.init_resource::<Events<T>>()
//                 .add_systems(PostUpdate, Events::<T>::update_system.run_if(should_run).after(FrameSet)); // 在每帧结束时清理事件
//         }
//         self
//     }
// }

// impl Plugin for WindowPlugin {
//     fn build(&self, app: &mut App) {
//         // User convenience events
//         app.add_frame_event::<WindowResized>()
//             .add_frame_event::<WindowCreated>()
//             .add_frame_event::<WindowClosed>()
//             .add_frame_event::<WindowCloseRequested>()
//             .add_frame_event::<WindowDestroyed>()
//             .add_frame_event::<RequestRedraw>()
//             // .add_frame_event::<CursorMoved>()
//             // .add_frame_event::<CursorEntered>()
//             // .add_frame_event::<CursorLeft>()
//             // .add_frame_event::<ReceivedCharacter>()
//             .add_frame_event::<Ime>()
//             // .add_frame_event::<WindowFocused>()
//             // .add_frame_event::<WindowScaleFactorChanged>()
//             // .add_frame_event::<WindowBackendScaleFactorChanged>()
//             // .add_frame_event::<FileDragAndDrop>()
//             // .add_frame_event::<WindowMoved>()
//             // .add_frame_event::<WindowThemeChanged>()
// 			;

//         if let Some(primary_window) = &self.primary_window {
//             app.world
//                 .spawn(primary_window.clone())
//                 .insert(PrimaryWindow);
//         }

//         match self.exit_condition {
//             ExitCondition::OnPrimaryClosed => {
//                 app.add_systems(PostUpdate, exit_on_primary_closed);
//             }
//             ExitCondition::OnAllClosed => {
//                 app.add_systems(PostUpdate, exit_on_all_closed);
//             }
//             ExitCondition::DontExit => {}
//         }

//         if self.close_when_requested {
//             // Need to run before `exit_on_*` systems
//             app.add_systems(Update, close_when_requested);
//         }

//         // Register event types
//         // app.register_type::<WindowResized>()
//         //     .register_type::<RequestRedraw>()
//         //     .register_type::<WindowCreated>()
//         //     .register_type::<WindowCloseRequested>()
//         //     .register_type::<WindowClosed>()
//         //     .register_type::<CursorMoved>()
//         //     .register_type::<CursorEntered>()
//         //     .register_type::<CursorLeft>()
//         //     .register_type::<ReceivedCharacter>()
//         //     .register_type::<WindowFocused>()
//         //     .register_type::<WindowScaleFactorChanged>()
//         //     .register_type::<WindowBackendScaleFactorChanged>()
//         //     .register_type::<FileDragAndDrop>()
//         //     .register_type::<WindowMoved>()
//         //     .register_type::<WindowThemeChanged>();

//         // // Register window descriptor and related types
//         // app.register_type::<Window>()
//         //     .register_type::<PrimaryWindow>()
//         //     .register_type::<Cursor>()
//         //     .register_type::<CursorIcon>()
//         //     .register_type::<CursorGrabMode>()
//         //     .register_type::<CompositeAlphaMode>()
//         //     .register_type::<WindowResolution>()
//         //     .register_type::<WindowPosition>()
//         //     .register_type::<WindowMode>()
//         //     .register_type::<WindowLevel>()
//         //     .register_type::<PresentMode>()
//         //     .register_type::<InternalWindowState>()
//         //     .register_type::<MonitorSelection>()
//         //     .register_type::<WindowResizeConstraints>()
//         //     .register_type::<WindowTheme>();

//         // // Register `PathBuf` as it's used by `FileDragAndDrop`
//         // app.register_type::<PathBuf>();
//     }
// }

/// Defines the specific conditions the application should exit on
#[derive(Clone)]
pub enum ExitCondition {
    /// Close application when the primary window is closed
    ///
    /// The plugin will add [`exit_on_primary_closed`] to [`Update`].
    OnPrimaryClosed,
    /// Close application when all windows are closed
    ///
    /// The plugin will add [`exit_on_all_closed`] to [`Update`].
    OnAllClosed,
    /// Keep application running headless even after closing all windows
    ///
    /// If selecting this, ensure that you send the [`bevy_app::AppExit`]
    /// event when the app should exit. If this does not occur, you will
    /// create 'headless' processes (processes without windows), which may
    /// surprise your users.
    DontExit,
}
