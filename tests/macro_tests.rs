use gpui_transit::{NavRoute, RouteConfig};

#[derive(Clone, PartialEq, Eq, Hash, Debug, NavRoute)]
struct HomeRoute;

#[derive(Clone, PartialEq, Eq, Hash, Debug, NavRoute)]
enum AppRoute {
    Chat(String),
}

#[test]
fn test_derive_nav_route() {
    let r1 = HomeRoute;
    assert!(r1.route_name().ends_with("HomeRoute"));

    let r2 = AppRoute::Chat("test".into());
    assert_eq!(r2, AppRoute::Chat("test".into()));
}
