//! # gpui-transit
//!
//! Production-grade, declarative, hierarchical navigation framework for GPUI with MVI
//! (Model-View-Intent) and Feature-Sliced Design (FSD) architecture.
//!
//! Inspired by:
//! - **Kotlin Decompose**: Clean separation between serializable/type-safe `RouteConfig`
//!   and live component instances (`Entity<V>`) in memory.
//! - **The Composable Architecture (TCA)**: Unidirectional data flow where children
//!   communicate with parent coordinators exclusively via semantic `Output` events.

pub mod actions;
pub mod context;
pub mod focus;
pub mod guard;
pub mod lifecycle;
pub mod mvi;
pub mod overlay;
pub mod registry;
pub mod restoration;
pub mod result;
pub mod route;
pub mod router_macro;
pub mod showcase;
pub mod slot;
pub mod stack;
pub mod transitions;
pub mod tree;

pub use actions::{DismissOverlay, NavigateBack, NavigateForward, NavigateHome};
pub use context::NavContext;
pub use focus::{FocusStack, NavFocusable, WindowFocusExt};
pub use guard::{GuardDecision, NavGuard, NavInterceptor};
pub use lifecycle::{NavLifecycle, ScreenLifecycle};
pub use mvi::{Dispatcher, MviComponent, MviContextExt, NavigationEffect, NavigatorHandle};
pub use overlay::{OverlayKind, OverlayNavigator};
pub use registry::{RouteBuilder, RouteRegistry};
pub use restoration::PersistedNavigationState;
pub use result::{AnyResultSender, NavigationResultError, ResultRouteConfig, ResultSender};
pub use route::{AnyRouteConfig, NavEntry, RouteConfig};
pub use slot::SlotNavigator;
pub use stack::StackNavigator;
pub use transitions::{TransitionConfig, TransitionKind};
pub use tree::{NavigationFlow, NavigationNode, NavigationNodeId};

#[cfg(feature = "derive")]
pub use gpui_transit_macros::NavRoute;
