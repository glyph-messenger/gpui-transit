//! Declarative ScreenHost macro for routing heterogeneous MVI screens.

/// Generates an `enum` wrapper over heterogeneous screens (`Entity<ScreenA>`, `Entity<ScreenB>`).
///
/// Automatically implements:
/// - Screen constructors that subscribe to child output events and forward them via `cx.emit(...)`.
/// - `gpui_kit::Render`, delegating directly into `.into_any_element()`.
/// - `gpui_kit::EventEmitter<UnifiedOutput>`.
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
                    entity: ::gpui_kit::Entity<$screen>,
                    _subscription: ::gpui_kit::Subscription,
                },
            )*
        }

        #[allow(non_snake_case)]
        impl $host_name {
            $(
                /// Creates a new variant embedding the child screen and subscribing to its output.
                pub fn $ctor(
                    child: ::gpui_kit::Entity<$screen>,
                    cx: &mut ::gpui_kit::Context<Self>,
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

        impl ::gpui_kit::EventEmitter<$output_type> for $host_name {}

        impl ::gpui_kit::Render for $host_name {
            fn render(
                &mut self,
                _window: &mut ::gpui_kit::Window,
                _cx: &mut ::gpui_kit::Context<Self>,
            ) -> impl ::gpui_kit::IntoElement {
                match self {
                    $(
                        Self::$variant { entity, .. } => {
                            ::gpui_kit::IntoElement::into_any_element(entity.clone())
                        }
                    )*
                }
            }
        }

        impl $crate::focus::NavFocusable for $host_name {
            fn focus_handle(&self, cx: &::gpui_kit::App) -> ::std::option::Option<::gpui_kit::FocusHandle> {
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
