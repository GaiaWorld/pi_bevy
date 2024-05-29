#![allow(clippy::type_complexity)]
#![warn(missing_docs)]
#![feature(stmt_expr_attributes)]
//! This crate provides logging functions and configuration for [Bevy](https://bevyengine.org)
//! apps, and automatically configures platform specific log handlers (i.e. WASM or Android).
//!
//! The macros provided for logging are reexported from [`tracing`](https://docs.rs/tracing),
//! and behave identically to it.
//!
//! By default, the [`LogPlugin`] from this crate is included in Bevy's `DefaultPlugins`
//! and the logging macros can be used out of the box, if used.
//!
//! For more fine-tuned control over logging behavior, set up the [`LogPlugin`] or
//! `DefaultPlugins` during app initialization.

use std::io::Write;
#[cfg(feature = "trace")]
use std::panic;

#[cfg(target_os = "android")]
mod android_tracing;

#[cfg(feature = "trace_tracy_memory")]
#[global_allocator]
static GLOBAL: tracy_client::ProfiledAllocator<std::alloc::System> =
    tracy_client::ProfiledAllocator::new(std::alloc::System, 100);

// pub mod prelude {
//     //! The Bevy Log Prelude.
//     #[doc(hidden)]
//     pub use bevy::utils::tracing::{
//         debug, debug_span, error, error_span, info, info_span, trace, trace_span, warn, warn_span,
//     };
// }

// use bevy_ecs::system::Resource;
// pub use bevy::utils::tracing::{
//     debug, debug_span, error, error_span, info, info_span, trace, trace_span, warn, warn_span,
//     Level,
// };

use pi_world::prelude::{App, Plugin};
// use bevy_app::{App, Plugin};
use tracing_log::LogTracer;
#[cfg(feature = "tracing-chrome")]
use tracing_subscriber::fmt::{format::DefaultFields, FormattedFields};
use tracing_subscriber::{prelude::*, registry::Registry, EnvFilter};

/// Adds logging to Apps. This plugin is part of the `DefaultPlugins`. Adding
/// this plugin will setup a collector appropriate to your target platform:
/// * Using [`tracing-subscriber`](https://crates.io/crates/tracing-subscriber) by default,
/// logging to `stdout`.
/// * Using [`android_log-sys`](https://crates.io/crates/android_log-sys) on Android,
/// logging to Android logs.
/// * Using [`tracing-wasm`](https://crates.io/crates/tracing-wasm) in WASM, logging
/// to the browser console.
///
/// You can configure this plugin.
/// ```no_run
/// # use bevy_app::{App, NoopPluginGroup as DefaultPlugins, PluginGroup};
/// # use bevy_log::LogPlugin;
/// # use bevy::utils::tracing::Level;
/// fn main() {
///     App::new()
///         .add_plugins(DefaultPlugins.set(LogPlugin {
///             level: Level::DEBUG,
///             filter: "wgpu=error,bevy_render=info,bevy_ecs=trace".to_string(),
///         }))
///         .run();
/// }
/// ```
///
/// Log level can also be changed using the `RUST_LOG` environment variable.
/// For example, using `RUST_LOG=wgpu=error,bevy_render=info,bevy_ecs=trace cargo run ..`
///
/// It has the same syntax as the field [`LogPlugin::filter`], see [`EnvFilter`].
/// If you define the `RUST_LOG` environment variable, the [`LogPlugin`] settings
/// will be ignored.
///
/// If you want to setup your own tracing collector, you should disable this
/// plugin from `DefaultPlugins`:
/// ```no_run
/// # use bevy_app::{App, NoopPluginGroup as DefaultPlugins, PluginGroup};
/// # use bevy_log::LogPlugin;
/// fn main() {
///     App::new()
///         .add_plugins(DefaultPlugins.build().disable::<LogPlugin>())
///         .run();
/// }
/// ```
///
/// # Panics
///
/// This plugin should not be added multiple times in the same process. This plugin
/// sets up global logging configuration for **all** Apps in a given process, and
/// rerunning the same initialization multiple times will lead to a panic.
pub struct LogPlugin<T: Write + Send + Sync + 'static> {
    /// Filters logs using the [`EnvFilter`] format
    pub filter: String,

    /// Filters out logs that are "less than" the given level.
    /// This can be further filtered using the `filter` setting.
    pub level: tracing::Level,

	/// 
	pub chrome_write: Option<T>,
}



/// 日志过滤处理器，如果添加LogPlugin，该类型的一个实例会被放置在Resource中， 外部可通过该单例，重设过滤条件
// #[derive(Resource)]
pub struct LogFilterHandle(pub tracing_subscriber::reload::Handle<EnvFilter, Registry>);

impl<T: Write + Send + Sync + 'static> Default for LogPlugin<T> {
    fn default() -> Self {
        Self {
            filter: "wgpu=error,naga=warn".to_string(),
            level: tracing::Level::INFO,
			chrome_write: None,
        }
    }
}

impl<T: Write + Send + Sync + 'static> Plugin for LogPlugin<T> {
    #[cfg_attr(not(feature = "tracing-chrome"), allow(unused_variables))]
    fn build(&self, app: &mut App) {
        #[cfg(feature = "trace")]
        {
            let old_handler = panic::take_hook();
            panic::set_hook(Box::new(move |infos| {
                println!("{}", tracing_error::SpanTrace::capture());
                old_handler(infos);
            }));
        }

        let finished_subscriber;
        let default_filter = { format!("{},{}", self.level, self.filter) };
        let filter_layer = EnvFilter::try_from_default_env()
            .or_else(|_| EnvFilter::try_new(&default_filter))
            .unwrap();
		let (filter_layer, reload_handle) = tracing_subscriber::reload::Layer::new(filter_layer);
        let subscriber = Registry::default().with(filter_layer);
		app.world.insert_single_res(LogFilterHandle(reload_handle));

        #[cfg(feature = "trace")]
        let subscriber = subscriber.with(tracing_error::ErrorLayer::default());

        #[cfg(all(not(target_arch = "wasm32"), not(target_os = "android")))]
        {
            #[cfg(feature = "tracing-chrome")]
            let chrome_layer = {
                let mut layer = tracing_chrome::ChromeLayerBuilder::new();
                if let Ok(path) = std::env::var("TRACE_CHROME") {
                    layer = layer.file(path);
                }
                let (chrome_layer, guard) = layer
                    .name_fn(Box::new(|event_or_span| match event_or_span {
                        tracing_chrome::EventOrSpan::Event(event) => event.metadata().name().into(),
                        tracing_chrome::EventOrSpan::Span(span) => {
                            if let Some(fields) =
                                span.extensions().get::<FormattedFields<DefaultFields>>()
                            {
                                format!("{}: {}", span.metadata().name(), fields.fields.as_str())
                            } else {
                                span.metadata().name().into()
                            }
                        }
                    }))
                    .build();
                app.world.insert_single_res(guard);
                chrome_layer
            };

            #[cfg(feature = "tracing-tracy")]
            let tracy_layer = tracing_tracy::TracyLayer::new();

            let fmt_layer = tracing_subscriber::fmt::Layer::default().with_writer(std::io::stderr);

            // bevy_render::renderer logs a `tracy.frame_mark` event every frame
            // at Level::INFO. Formatted logs should omit it.
            #[cfg(feature = "tracing-tracy")]
            let fmt_layer =
                fmt_layer.with_filter(tracing_subscriber::filter::FilterFn::new(|meta| {
                    meta.fields().field("tracy.frame_mark").is_none()
                }));

            let subscriber = subscriber.with(fmt_layer);

            #[cfg(feature = "tracing-chrome")]
            let subscriber = subscriber.with(chrome_layer);
            #[cfg(feature = "tracing-tracy")]
            let subscriber = subscriber.with(tracy_layer);

            finished_subscriber = subscriber;
        }

        #[cfg(target_arch = "wasm32")]
        {
            console_error_panic_hook::set_once();
            // finished_subscriber = subscriber.with(tracing_wasm::WASMLayer::new(
            //     tracing_wasm::WASMLayerConfig::default(),
            // ));
			// finished_subscriber = subscriber.with(tracing_browser_subscriber::BrowserLayer::new());
			// tracing_browser_subscriber::configure_as_global_default();
			// #[cfg(feature = "tracing_chrome_wasm")]
			// if let Some(chrome_write) = unsafe {&mut *(self as *const Self as usize as *mut Self)}.chrome_write.take() {
			// 	let chrome_layer = {
			// 		let mut layer = tracing_chrome_wasm::ChromeLayerBuilder::new();
			// 		layer = layer.writer(chrome_write);
			// 		let (chrome_layer, guard) = layer
			// 			.name_fn(Box::new(|event_or_span| match event_or_span {
			// 				tracing_chrome_wasm::EventOrSpan::Event(event) => event.metadata().name().into(),
			// 				tracing_chrome_wasm::EventOrSpan::Span(span) => {
			// 					if let Some(fields) =
			// 						span.extensions().get::<tracing_subscriber::fmt::FormattedFields<tracing_subscriber::fmt::format::DefaultFields>>()
			// 					{
			// 						format!("{}: {}", span.metadata().name(), fields.fields.as_str())
			// 					} else {
			// 						span.metadata().name().into()
			// 					}
			// 				}
			// 			}))
			// 			.build();
			// 		app.world.insert_non_send_resource(guard);
			// 		chrome_layer
			// 	};
			// 	let subscriber = subscriber.with(chrome_layer);
			// 	finished_subscriber = subscriber;
			// } else {
			// 	panic!("need chrome Writer!");
			// 	// finished_subscriber = subscriber.with(tracing_browser_subscriber::BrowserLayer::new());
			// }

			// #[cfg(all(not(feature="tracing_chrome_wasm"), feature = "tracing-wasm"))] 
			finished_subscriber = subscriber.with(tracing_wasm::WASMLayer::new(
				tracing_wasm::WASMLayerConfig::default(),
			));
			
        }

        #[cfg(target_os = "android")]
        {
            finished_subscriber = subscriber.with(android_tracing::AndroidLayer::default());
        }

        let logger_already_set = LogTracer::init().is_err();
        let subscriber_already_set =
            tracing::subscriber::set_global_default(finished_subscriber).is_err();

        match (logger_already_set, subscriber_already_set) {
            (true, true) => tracing::warn!(
                "Could not set global logger and tracing subscriber as they are already set. Consider disabling LogPlugin."
            ),
            (true, _) => tracing::warn!("Could not set global logger as it is already set. Consider disabling LogPlugin."),
            (_, true) => tracing::warn!("Could not set global tracing subscriber as it is already set. Consider disabling LogPlugin."),
            _ => (),
        }
    }
}
