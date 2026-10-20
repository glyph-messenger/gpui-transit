use gpui_transit::PersistedNavigationState;

#[test]
fn test_persisted_navigation_state_serialization() {
    let state = PersistedNavigationState {
        stack_routes: vec!["/home".into(), "/chat/42".into(), "/profile".into()],
        active_slot: Some("/chat/42".into()),
        active_tab: Some(1),
    };

    let json = state.to_json().expect("Failed to serialize to JSON");
    assert!(json.contains("/chat/42"));
    assert!(json.contains("/profile"));

    let restored = PersistedNavigationState::from_json(&json).expect("Failed to deserialize");
    assert_eq!(restored.stack_routes.len(), 3);
    assert_eq!(restored.active_slot, Some("/chat/42".into()));
    assert_eq!(restored.active_tab, Some(1));
}
