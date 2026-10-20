use gpui_kit::prelude::*;
use gpui_kit::{App, Context, EventEmitter, IntoElement, Render, Window, div};
use gpui_transit::{ResultRouteConfig, ResultSender, StackNavigator};

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
struct PickerRoute;

impl ResultRouteConfig for PickerRoute {
    type Result = String;
}

struct PickerScreen {
    sender: ResultSender<String>,
}

impl Render for PickerScreen {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div().child("Picker")
    }
}

#[derive(Clone, Debug)]
struct EmptyOutput;

impl EventEmitter<EmptyOutput> for PickerScreen {}

struct RootHost;

impl Render for RootHost {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
    }
}

#[tokio::test]
async fn test_push_for_result_success() {
    let app = App::headless();
    app.run(|cx| {
        let parent = cx.new(|_| RootHost);
        parent.update(cx, |_host, cx| {
            let mut stack = StackNavigator::new_empty();

            let rx = stack.push_for_result(
                PickerRoute,
                cx,
                |_route, mut sender, _cx| {
                    // Send result
                    let _ = sender.send("Selected Item 42".to_string());
                    PickerScreen { sender }
                },
                |_p, _c, _e: &EmptyOutput, _cx| {},
            );

            // Channel should have received the value
            let val = rx.now_or_never().expect("Should be ready immediately");
            assert_eq!(val, Ok("Selected Item 42".to_string()));
        });
    });
}
