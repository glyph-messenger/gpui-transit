# gpui-transit

[![Crates.io](https://img.shields.io/crates/v/gpui-transit.svg)](https://crates.io/crates/gpui-transit)
[![Documentation](https://docs.rs/gpui-transit/badge.svg)](https://docs.rs/gpui-transit)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)

**gpui-transit** is a production-grade, type-safe, declarative navigation framework for [GPUI](https://github.com/zed-industries/zed) (the high-performance GPU-accelerated UI framework from Zed).

Built for complex, modern desktop applications, `gpui-transit` combines the architectural purity of **Kotlin Decompose** (separating lightweight serializable configs from live component instances) with the unidirectional flow of **The Composable Architecture (TCA)** and GPUI's native reactivity.

---

## Key Features

- **Lifecycle Engine (`NavLifecycle`)**: Full lifecycle management (`Resumed`, `Paused`, `Destroyed`) with automatic subscription detaching for background screens to eliminate memory leaks and rogue event triggers.
- **Dynamic Route Registry (`RouteRegistry`)**: Register screens cleanly by route type without cumbersome enum wrappers or macro bottlenecks.
- **Typed Push-for-Result (`push_for_result`)**: Await typed results from picker or modal screens via Rust `async`/`await` and channels with RAII cancellation.
- **Navigation Primitives**:
  - `StackNavigator`: History stack with automatic focus restoration, declarative reconciliation (`sync_stack`), and duplicate prevention.
  - `SlotNavigator`: Single active slot with a bounded LRU cache for multi-tab or chat interfaces.
  - `OverlayNavigator` (Overlays 2.0): Stacked modals, popovers, drawers, and toasts with focus trapping and backdrop dismissal.
- **Nested Navigation & Back Bubbling**: Hierarchical tree of navigators that automatically bubbles up Back actions (`Escape`, `Cmd+[`, mouse back button).
- **Navigation Guards & Interceptors**: Prevent accidental loss of unsaved changes before leaving screens.
- **Focus Management**: Deep focus stack restoration preserves and restores focused element handles seamlessly across transitions.
- **State Restoration & Deep Linking**: Full serialization and restoration of navigation state.

---

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT License](LICENSE-MIT) at your option.
