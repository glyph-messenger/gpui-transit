//! Hierarchical navigation tree supporting nested navigators and event bubbling.
//!
//! In desktop applications, navigation is naturally hierarchical:
//! ```text
//! RootWindow
//!  └── OverlayNavigator (Dialogs, Sheets)
//!       └── SplitView
//!            ├── SidebarSlot
//!            └── WorkspaceTabs
//!                 └── ActiveTab (StackNavigator)
//! ```
//! When the user triggers `NavigateBack` (e.g. `Escape`, `Cmd+[`, or mouse Back),
//! the event bubbles up:
//! 1. Dismisses the top modal overlay if one is open.
//! 2. Otherwise pops the innermost active stack.
//! 3. If that stack is at root, passes the back action to its parent.

use std::sync::atomic::{AtomicUsize, Ordering};
use gpui::{App, Window};

static NEXT_NODE_ID: AtomicUsize = AtomicUsize::new(1);

/// Unique identifier for a node in the navigation tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NavigationNodeId(usize);

impl NavigationNodeId {
    /// Generates a new unique navigation node identifier.
    pub fn next() -> Self {
        Self(NEXT_NODE_ID.fetch_add(1, Ordering::Relaxed))
    }
}

/// The flow result of attempting to handle a back navigation action.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavigationFlow {
    /// The event was fully consumed and handled by this node.
    Handled,
    /// This node could not handle the action (e.g. it is at its root screen)
    /// and the action should bubble up to the parent navigator.
    BubbleUp,
}

impl NavigationFlow {
    pub fn is_handled(&self) -> bool {
        matches!(self, Self::Handled)
    }

    pub fn is_bubble(&self) -> bool {
        matches!(self, Self::BubbleUp)
    }
}

/// Trait implemented by any navigator that participates in the navigation tree hierarchy.
pub trait NavigationNode {
    /// Returns the unique identifier of this node.
    fn node_id(&self) -> NavigationNodeId;

    /// Returns `true` if this navigator has a screen or state it can pop.
    fn can_pop(&self) -> bool;

    /// Attempts to pop the current screen.
    ///
    /// Returns `NavigationFlow::Handled` if successful, or `NavigationFlow::BubbleUp`
    /// if the navigator is already at root or cannot pop.
    fn handle_pop(&mut self, window: &mut Window, cx: &mut App) -> NavigationFlow;
}
