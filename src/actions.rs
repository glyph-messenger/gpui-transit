//! Native GPUI Actions and keyboard shortcuts integration for navigation.

use gpui::actions;

// Declare navigation actions using GPUI's action system
actions!(
    transit,
    [
        /// Navigate back to the previous screen in the active stack.
        NavigateBack,
        /// Navigate forward in history (if supported).
        NavigateForward,
        /// Dismiss the active modal overlay, sheet, or popover.
        DismissOverlay,
        /// Pop all screens down to the root.
        NavigateHome,
    ]
);
