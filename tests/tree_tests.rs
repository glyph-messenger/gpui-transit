use gpui_kit::prelude::*;
use gpui_kit::{App, Context, IntoElement, Render, Window, div};
use gpui_transit::{NavigationFlow, NavigationNode, StackNavigator};

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
enum SubRoute {
    First,
    Second,
}

struct DummyScreen;

impl Render for DummyScreen {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
    }
}

struct RootHost;

impl Render for RootHost {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
    }
}

#[test]
fn test_navigation_tree_node_bubbling() {
    let app = App::headless();
    app.run(|cx| {
        let parent = cx.new(|_| RootHost);
        parent.update(cx, |_host, cx| {
            let mut stack: StackNavigator<SubRoute, DummyScreen> = StackNavigator::new_empty();

            // Empty stack -> BubbleUp
            assert!(!stack.can_pop());

            // Push First -> len 1 -> can_pop is false (cannot pop root)
            stack.push_simple(SubRoute::First, cx, |_r, _cx| DummyScreen);
            assert!(!stack.can_pop());

            // Push Second -> len 2 -> can_pop is true
            stack.push_simple(SubRoute::Second, cx, |_r, _cx| DummyScreen);
            assert!(stack.can_pop());

            // Pop Second -> Handled
            let window_stub = unsafe { &mut *(1 as *mut Window) }; // Not dereferenced in test
            let flow = stack.handle_pop(window_stub, cx);
            assert_eq!(flow, NavigationFlow::Handled);
            assert_eq!(stack.len(), 1);

            // Pop at root -> BubbleUp
            let flow2 = stack.handle_pop(window_stub, cx);
            assert_eq!(flow2, NavigationFlow::BubbleUp);
            assert_eq!(stack.len(), 1);
        });
    });
}
