//! Declarative ScreenHost macro for routing heterogeneous MVI screens.

/// Generates an `enum` wrapper over heterogeneous screens (`Entity<ScreenA>`, `Entity<ScreenB>`).
///
/// Automatically implements:
/// - Screen constructors that subscribe to child output events and forward them via `cx.emit(...)`.
/// - `gpui::Render`, delegating directly into `.into_any_element()`.
/// - `gpui::EventEmitter<UnifiedOutput>`.
/// - `NavFocusable`, delegating focus to the active screen.
///
/// # Example
/// ```rust,ignore
/// define_route_host! {
///     pub enum InspectorHost, InspectorOutput {
///         Profile: profile(ProfileFeature, ProfileOutput => InspectorOutput::Profile),
///         Media: media(SharedMediaFeature, SharedMediaOutput => InspectorOutput::Media),
///     }
/// }
/// ```
#[macro_export]
macro_rules! define_route_host {
    // Syntax with explicit constructor: Variant: ctor(Screen, Output => Mapper)
    (
        $(#[$meta:meta])*
        $vis:vis enum $host_name:ident, $output_type:ident {
            $(
                $variant:ident : $ctor:ident (
                    $screen:ty,
                    $child_output:ty => $mapper:expr
                )
            ),* $(,)?
        }
    ) => {
        $(#[$meta])*
        $vis enum $host_name {
            $(
                $variant {
                    entity: $crate::gpui::Entity<$screen>,
                    _subscription: $crate::gpui::Subscription,
                },
            )*
        }

        #[allow(non_snake_case)]
        impl $host_name {
            $(
                /// Creates a new variant embedding the child screen and subscribing to its output.
                pub fn $ctor(
                    child: $crate::gpui::Entity<$screen>,
                    cx: &mut $crate::gpui::Context<Self>,
                ) -> Self {
                    let mapper = $mapper;
                    let _subscription = cx.subscribe(&child, move |_this, _child, event: &$child_output, cx| {
                        let unified_output: $output_type = mapper(event.clone());
                        cx.emit(unified_output);
                    });

                    Self::$variant {
                        entity: child,
                        _subscription,
                    }
                }
            )*
        }

        impl $crate::gpui::EventEmitter<$output_type> for $host_name {}

        impl $crate::gpui::Render for $host_name {
            fn render(
                &mut self,
                _window: &mut $crate::gpui::Window,
                _cx: &mut $crate::gpui::Context<Self>,
            ) -> impl $crate::gpui::IntoElement {
                match self {
                    $(
                        Self::$variant { entity, .. } => {
                            $crate::gpui::IntoElement::into_any_element(entity.clone())
                        }
                    )*
                }
            }
        }

        impl $crate::focus::NavFocusable for $host_name {
            fn focus_handle(&self, cx: &$crate::gpui::App) -> ::std::option::Option<$crate::gpui::FocusHandle> {
                match self {
                    $(
                        Self::$variant { entity, .. } => {
                            $crate::focus::NavFocusable::focus_handle(&*entity.read(cx), cx)
                        }
                    )*
                }
            }
        }
    };

    // Shorthand syntax without explicit constructor (uses Variant name):
    (
        $(#[$meta:meta])*
        $vis:vis enum $host_name:ident, $output_type:ident {
            $(
                $variant:ident (
                    $screen:ty,
                    $child_output:ty => $mapper:expr
                )
            ),* $(,)?
        }
    ) => {
        $crate::define_route_host! {
            $(#[$meta])*
            $vis enum $host_name, $output_type {
                $(
                    $variant : $variant (
                        $screen,
                        $child_output => $mapper
                    )
                ),*
            }
        }
    };
}
