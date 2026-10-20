use std::sync::mpsc;
use gpui_kit::{App, ClickEvent, Context, Render, WeakEntity, Window};

/// Trait for an MVI (Model-View-Intent) component.
///
/// Follows unidirectional data flow (UDF) inspired by The Composable Architecture (TCA):
/// - `State`: Component's presentation state.
/// - `Intent`: User actions or asynchronous events directed at the component.
/// - `Output`: Semantic events emitted by this component to its parent/coordinator.
pub trait MviComponent: Render + 'static {
    type State;
    type Intent: 'static;
    type Output: 'static;

    /// Read-only reference to the current state.
    fn state(&self) -> &Self::State;

    /// Dispatches an intent to mutate state and produce side effects.
    fn dispatch(&mut self, intent: Self::Intent, window: &mut Window, cx: &mut Context<Self>);

    /// Optional headless dispatch when window is not available.
    fn dispatch_headless(&mut self, _intent: Self::Intent, _cx: &mut Context<Self>) {}
}

/// A lightweight, clonable handle to an MVI component's `WeakEntity`.
///
/// Allows safely dispatching intents from UI event handlers (`.on_click`, `.on_mouse_down`, etc.)
/// without manual entity upgrade and `update()` boilerplate.
#[derive(Debug)]
pub struct Dispatcher<C: MviComponent> {
    weak: WeakEntity<C>,
}

impl<C: MviComponent> Clone for Dispatcher<C> {
    fn clone(&self) -> Self {
        Self { weak: self.weak.clone() }
    }
}

impl<C: MviComponent> Dispatcher<C> {
    /// Creates a new dispatcher wrapping a `WeakEntity`.
    pub fn new(weak: WeakEntity<C>) -> Self {
        Self { weak }
    }

    /// Dispatches an intent to the underlying component if it is still alive.
    ///
    /// Returns `true` if the entity was updated, `false` if it was dropped.
    pub fn dispatch(&self, intent: C::Intent, window: &mut Window, cx: &mut App) -> bool {
        if let Some(entity) = self.weak.upgrade() {
            entity.update(cx, |this, cx| {
                this.dispatch(intent, window, cx);
            });
            true
        } else {
            false
        }
    }

    /// Headless dispatch without window access.
    pub fn dispatch_headless(&self, intent: C::Intent, cx: &mut App) -> bool {
        if let Some(entity) = self.weak.upgrade() {
            entity.update(cx, |this, cx| {
                this.dispatch_headless(intent, cx);
            });
            true
        } else {
            false
        }
    }

    /// Returns a listener closure suitable for GPUI's `.on_click(...)`.
    pub fn on_click(
        &self,
        intent: C::Intent,
    ) -> impl Fn(&ClickEvent, &mut Window, &mut App) + 'static
    where
        C::Intent: Clone,
    {
        let dispatcher = self.clone();
        move |_event, window, cx| {
            dispatcher.dispatch(intent.clone(), window, cx);
        }
    }

    /// Returns an event listener closure that transforms an event into an intent.
    pub fn action<E: 'static>(
        &self,
        make_intent: impl Fn(&E) -> C::Intent + 'static,
    ) -> impl Fn(&E, &mut Window, &mut App) + 'static {
        let dispatcher = self.clone();
        move |event, window, cx| {
            let intent = make_intent(event);
            dispatcher.dispatch(intent, window, cx);
        }
    }
}

/// Extension trait for `Context<C>` to easily obtain a `Dispatcher<C>`.
pub trait MviContextExt<C: MviComponent> {
    /// Creates a lightweight `Dispatcher<C>` for this context.
    fn dispatcher(&self) -> Dispatcher<C>;
}

impl<C: MviComponent> MviContextExt<C> for Context<'_, C> {
    fn dispatcher(&self) -> Dispatcher<C> {
        Dispatcher::new(self.entity().downgrade())
    }
}

/// Generic navigation commands that can be dispatched as MVI side-effects.
#[derive(Debug, Clone)]
pub enum NavigationEffect {
    Pop,
    PopToRoot,
    DismissOverlay,
}

/// Lightweight handle for triggering navigation side-effects from anywhere in an MVI store.
#[derive(Clone, Debug)]
pub struct NavigatorHandle {
    tx: mpsc::Sender<NavigationEffect>,
}

impl NavigatorHandle {
    /// Creates a new `NavigatorHandle` and returns the receiver for coordinators to poll or bind.
    pub fn channel() -> (Self, mpsc::Receiver<NavigationEffect>) {
        let (tx, rx) = mpsc::channel();
        (Self { tx }, rx)
    }

    /// Sends a pop command.
    pub fn pop(&self) -> bool {
        self.tx.send(NavigationEffect::Pop).is_ok()
    }

    /// Sends a pop-to-root command.
    pub fn pop_to_root(&self) -> bool {
        self.tx.send(NavigationEffect::PopToRoot).is_ok()
    }

    /// Sends a dismiss overlay command.
    pub fn dismiss_overlay(&self) -> bool {
        self.tx.send(NavigationEffect::DismissOverlay).is_ok()
    }
}
