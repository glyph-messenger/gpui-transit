//! Transitions and animation primitives for GPUI screen changes.

use std::time::Duration;

/// The visual animation style used when transitioning between screens.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TransitionKind {
    /// Instant screen switch without animation.
    #[default]
    Instant,
    /// Slide in from right (push) or slide out to right (pop).
    SlideHorizontal,
    /// Slide in from bottom (modal presentation).
    SlideVertical,
    /// Cross-fade between screens.
    Fade,
}

/// Configuration settings for a navigation transition animation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TransitionConfig {
    pub kind: TransitionKind,
    pub duration: Duration,
}

impl Default for TransitionConfig {
    fn default() -> Self {
        Self {
            kind: TransitionKind::Instant,
            duration: Duration::from_millis(150),
        }
    }
}

impl TransitionConfig {
    /// Instant transition without animation.
    pub const INSTANT: Self = Self {
        kind: TransitionKind::Instant,
        duration: Duration::ZERO,
    };

    /// Standard desktop slide animation.
    pub const SLIDE: Self = Self {
        kind: TransitionKind::SlideHorizontal,
        duration: Duration::from_millis(180),
    };

    /// Smooth crossfade transition.
    pub const FADE: Self = Self {
        kind: TransitionKind::Fade,
        duration: Duration::from_millis(150),
    };
}
