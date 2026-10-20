//! Dynamic route registry decoupling route configs from concrete view types.
//!
//! Replaces monomorphic enum hosts with a flexible registry where screens can be registered
//! independently by route config type.

use std::any::{Any, TypeId};
use std::collections::HashMap;
use gpui_kit::{AnyElement, App, IntoElement, Window, div};

use crate::route::RouteConfig;

/// Trait object for building an `AnyElement` from a type-erased route config.
pub trait RouteBuilder: 'static {
    fn build_element(&self, config: &dyn Any, window: &mut Window, cx: &mut App) -> AnyElement;
}

struct TypedRouteBuilder<C, F> {
    builder: F,
    _marker: std::marker::PhantomData<C>,
}

impl<C: RouteConfig, F> RouteBuilder for TypedRouteBuilder<C, F>
where
    F: Fn(&C, &mut Window, &mut App) -> AnyElement + 'static,
{
    fn build_element(&self, config: &dyn Any, window: &mut Window, cx: &mut App) -> AnyElement {
        if let Some(cfg) = config.downcast_ref::<C>() {
            (self.builder)(cfg, window, cx)
        } else {
            div().into_any_element()
        }
    }
}

/// Centralized registry holding screen builder factories for different route configs.
#[derive(Default)]
pub struct RouteRegistry {
    builders: HashMap<TypeId, Box<dyn RouteBuilder>>,
}

impl RouteRegistry {
    /// Creates an empty route registry.
    pub fn new() -> Self {
        Self { builders: HashMap::new() }
    }

    /// Registers a view factory for route type `C`.
    pub fn register<C: RouteConfig, E: IntoElement + 'static>(
        &mut self,
        factory: impl Fn(&C, &mut Window, &mut App) -> E + 'static,
    ) {
        let boxed_builder = TypedRouteBuilder {
            builder: move |cfg: &C, window: &mut Window, cx: &mut App| {
                factory(cfg, window, cx).into_any_element()
            },
            _marker: std::marker::PhantomData,
        };
        self.builders.insert(TypeId::of::<C>(), Box::new(boxed_builder));
    }

    /// Returns `true` if a builder is registered for route type `C`.
    pub fn has<C: RouteConfig>(&self) -> bool {
        self.builders.contains_key(&TypeId::of::<C>())
    }

    /// Instantiates and renders the route configuration if a builder was registered.
    pub fn render<C: RouteConfig>(
        &self,
        config: &C,
        window: &mut Window,
        cx: &mut App,
    ) -> Option<AnyElement> {
        let builder = self.builders.get(&TypeId::of::<C>())?;
        Some(builder.build_element(config, window, cx))
    }
}
