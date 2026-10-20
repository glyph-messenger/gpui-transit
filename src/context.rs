//! Unified navigation context combining Window and GPUI Context.

use gpui::{App, Context, Window};

/// Unified navigation context wrapper providing ergonomic access to both the active `Window`
/// and the GPUI `Context<'c, P>` or `App`.
pub struct NavContext<'a, 'b, 'c, P> {
    pub window: &'a mut Window,
    pub cx: &'b mut Context<'c, P>,
}

impl<'a, 'b, 'c, P> NavContext<'a, 'b, 'c, P> {
    /// Creates a new `NavContext` pairing a Window and Context.
    pub fn new(window: &'a mut Window, cx: &'b mut Context<'c, P>) -> Self {
        Self { window, cx }
    }

    /// Access the underlying App reference.
    pub fn app(&self) -> &App {
        self.cx
    }

    /// Access the mutable App reference.
    pub fn app_mut(&mut self) -> &mut App {
        self.cx
    }
}
