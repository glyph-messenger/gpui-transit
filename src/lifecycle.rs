//! Lifecycle engine for GPUI components in navigation stacks.
//!
//! Provides lifecycle hooks (`Resumed`, `Paused`, `Destroyed`) allowing screens to:
//! - Pause expensive background tasks, timers, polling, or animations when covered by a new screen.
//! - Resume operations and refresh data when popping back to the screen.
//! - Cleanly release memory, flush drafts to disk, or tear down network streams when destroyed.

use gpui_kit::Context;

/// The presentation lifecycle state of a screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScreenLifecycle {
    /// The screen is fully active, visible, and may receive user input and focus.
    #[default]
    Resumed,

    /// The screen is retained in memory (e.g. buried deeper in a navigation stack
    /// or parked in an LRU cache slot), but is currently hidden from view.
    ///
    /// Heavy operations (timers, video decoding, polling) should be suspended.
    Paused,

    /// The screen has been popped from the stack, evicted from the cache,
    /// or dismissed from an overlay. Its resources are about to be dropped.
    Destroyed,
}

impl ScreenLifecycle {
    /// Returns `true` if the screen is currently active and visible.
    pub fn is_resumed(&self) -> bool {
        matches!(self, Self::Resumed)
    }

    /// Returns `true` if the screen is suspended in the background.
    pub fn is_paused(&self) -> bool {
        matches!(self, Self::Paused)
    }

    /// Returns `true` if the screen has been permanently removed.
    pub fn is_destroyed(&self) -> bool {
        matches!(self, Self::Destroyed)
    }
}

/// Trait implemented by components that want to react to navigation lifecycle transitions.
pub trait NavLifecycle {
    /// Invoked whenever the screen transitions between `Resumed`, `Paused`, and `Destroyed`.
    fn on_lifecycle_change(&mut self, state: ScreenLifecycle, cx: &mut Context<Self>)
    where
        Self: Sized,
    {
        let _ = (state, cx);
    }

    /// Convenience callback invoked when the screen becomes active/visible.
    fn on_resume(&mut self, cx: &mut Context<Self>)
    where
        Self: Sized,
    {
        let _ = cx;
    }

    /// Convenience callback invoked when the screen is suspended in the background.
    fn on_pause(&mut self, cx: &mut Context<Self>)
    where
        Self: Sized,
    {
        let _ = cx;
    }

    /// Convenience callback invoked when the screen is permanently evicted or popped.
    fn on_destroy(&mut self, cx: &mut Context<Self>)
    where
        Self: Sized,
    {
        let _ = cx;
    }
}
