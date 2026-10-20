use gpui_transit::{NavRoute, RouteConfig};

#[derive(Clone, PartialEq, Eq, Hash, Debug, NavRoute)]
struct HomeRoute;

#[derive(Clone, PartialEq, Eq, Hash, Debug, NavRoute)]
enum AppRoute {
    Home,
    Chat(String),
}

#[test]
fn test_derive_nav_route() {
    let r1 = HomeRoute;
    assert_eq!(r1.route_name(), "tests::macro_tests::HomeRoute");

    let r2 = AppRoute::Chat("test".into());
    assert_eq!(r2, AppRoute::Chat("test".into()));
}
