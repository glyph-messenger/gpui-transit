use std::any::Any;
use std::fmt::Debug;
use std::hash::Hash;

use gpui_kit::{Entity, FocusHandle, Subscription};

use crate::lifecycle::ScreenLifecycle;
use crate::result::AnyResultSender;

/// Marker trait for route configurations.
///
/// Route configs must be lightweight, cloneable, equatable, and hashable
/// (analogous to Kotlin Decompose serializable Configs).
pub trait RouteConfig: Clone + PartialEq + Eq + Hash + Debug + 'static {
    /// Returns the unique route identifier or path string if applicable.
    fn route_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
}

impl<T> RouteConfig for T where T: Clone + PartialEq + Eq + Hash + Debug + 'static {}

/// A type-erased helper trait for route configurations.
pub trait AnyRouteConfig: Debug + 'static {
    fn as_any(&self) -> &dyn Any;
    fn clone_box(&self) -> Box<dyn AnyRouteConfig>;
    fn equals(&self, other: &dyn AnyRouteConfig) -> bool;
    fn type_id_of(&self) -> std::any::TypeId {
        std::any::TypeId::of::<Self>()
    }
}

impl<T: RouteConfig> AnyRouteConfig for T {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn AnyRouteConfig> {
        Box::new(self.clone())
    }

    fn equals(&self, other: &dyn AnyRouteConfig) -> bool {
        other.as_any().downcast_ref::<T>().is_some_and(|o| o == self)
    }
}

/// An active navigation record holding a route configuration, its live component instance,
/// and its lifecycle metadata.
pub struct NavEntry<C, V> {
    pub config: C,
    pub entity: Entity<V>,
    pub _subscription: Option<Subscription>,
    pub lifecycle: ScreenLifecycle,
    pub saved_focus: Option<FocusHandle>,
    pub result_sender: Option<Box<dyn AnyResultSender>>,
}

impl<C, V> NavEntry<C, V> {
    /// Creates a new navigation entry in the default `Resumed` state.
    pub fn new(config: C, entity: Entity<V>, subscription: Option<Subscription>) -> Self {
        Self {
            config,
            entity,
            _subscription: subscription,
            lifecycle: ScreenLifecycle::Resumed,
            saved_focus: None,
            result_sender: None,
        }
    }

    /// Creates a new navigation entry with an associated result sender.
    pub fn with_result(
        config: C,
        entity: Entity<V>,
        subscription: Option<Subscription>,
        sender: Box<dyn AnyResultSender>,
    ) -> Self {
        Self {
            config,
            entity,
            _subscription: subscription,
            lifecycle: ScreenLifecycle::Resumed,
            saved_focus: None,
            result_sender: Some(sender),
        }
    }

    /// Read-only access to the route configuration.
    pub fn config(&self) -> &C {
        &self.config
    }

    /// Read-only access to the live entity.
    pub fn entity(&self) -> &Entity<V> {
        &self.entity
    }

    /// Read-only access to the current lifecycle state.
    pub fn lifecycle(&self) -> ScreenLifecycle {
        self.lifecycle
    }

    /// Sets the lifecycle state.
    pub fn set_lifecycle(&mut self, state: ScreenLifecycle) {
        self.lifecycle = state;
    }

    /// Replaces the active event subscription.
    pub fn set_subscription(&mut self, subscription: Option<Subscription>) {
        self._subscription = subscription;
    }

    /// Detaches the subscription (e.g. when the screen is suspended in background or LRU cache).
    pub fn detach_subscription(&mut self) -> Option<Subscription> {
        self._subscription.take()
    }

    /// Saves the active focus handle before being covered by another screen.
    pub fn save_focus(&mut self, focus: Option<FocusHandle>) {
        self.saved_focus = focus;
    }

    /// Takes the saved focus handle for restoration.
    pub fn take_saved_focus(&mut self) -> Option<FocusHandle> {
        self.saved_focus.take()
    }
}

impl<C: Debug, V: 'static> Debug for NavEntry<C, V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NavEntry")
            .field("config", &self.config)
            .field("entity", &self.entity.entity_id())
            .field("lifecycle", &self.lifecycle)
            .field("has_focus", &self.saved_focus.is_some())
            .finish()
    }
}
