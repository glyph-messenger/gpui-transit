//! Navigation guards and interceptors for preventing loss of unsaved changes,
//! verifying permissions, and orchestrating asynchronous confirmation dialogs.

use std::any::Any;
use std::sync::Arc;
use gpui_kit::App;

/// The decision produced by a navigation guard.
pub enum GuardDecision {
    /// Proceed with navigation immediately.
    Allow,

    /// Completely abort the navigation request; the current screen remains active.
    Prevent,

    /// Request confirmation from the user (e.g., showing a modal dialog).
    ///
    /// The guard provides a closure that takes a continuation callback `on_confirmed`.
    /// When the user confirms the action, invoking `on_confirmed()` executes the deferred navigation.
    Confirm(Box<dyn FnOnce(Box<dyn FnOnce(&mut App) + Send + 'static>, &mut App) + 'static>),
}

/// Trait implemented by screens or coordinators that can intercept navigation transitions.
pub trait NavGuard {
    /// Evaluates whether leaving this screen should be prevented, allowed, or confirmed.
    ///
    /// - `from_route`: The route configuration of the screen being left.
    /// - `to_route`: The optional target route (or `None` if popping/closing).
    fn check_navigation(
        &self,
        from_route: &dyn Any,
        to_route: Option<&dyn Any>,
        cx: &App,
    ) -> GuardDecision {
        let _ = (from_route, to_route, cx);
        GuardDecision::Allow
    }
}

/// A registered global or contextual navigation interceptor closure.
pub type NavInterceptor = Arc<
    dyn Fn(&dyn Any, Option<&dyn Any>, &App) -> GuardDecision + Send + Sync + 'static,
>;
