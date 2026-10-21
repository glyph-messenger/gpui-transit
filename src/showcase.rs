//! # Showcase: Telegram-like Desktop Architecture
//!
//! Demonstrates the full integration of:
//! - **MVI (Model-View-Intent)** with unidirectional data flow and `Dispatcher<C>`.
//! - **Feature-Sliced Design (FSD)**: Isolated slices in `features/` communicating upward strictly
//!   through typed `Output` events.
//! - **Kotlin Decompose-style Navigation**: Type-safe `RouteConfig` mapped to live `Entity<V>` instances.
//! - **3 Navigation Primitives**:
//!   1. `SlotNavigator` with LRU-cache (5 chats) for the central panel.
//!   2. `StackNavigator` with `define_route_host!` for the inspector column.
//!   3. `OverlayNavigator` for modal dialogs with focus trapping and backdrop dismissal.

use gpui::prelude::*;
use gpui::{
    App, ClickEvent, Context, Entity, EventEmitter, FocusHandle, IntoElement, ParentElement,
    Render, Styled, Window, div, px, rgb,
};

use crate::focus::NavFocusable;
use crate::mvi::{MviComponent, MviContextExt};
use crate::overlay::OverlayNavigator;
use crate::slot::SlotNavigator;
use crate::stack::StackNavigator;

// ============================================================================
// 1. Shared Layer (`shared/`)
// ============================================================================

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct PeerId(pub String);

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ChatRoomRoute(pub PeerId);

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum InspectorRoute {
    Profile(PeerId),
    Media(PeerId),
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum ModalRoute {
    Settings,
}

// ============================================================================
// 2. Feature: Chat List (`features/chat_list/`)
// ============================================================================

pub struct ChatListItem {
    pub id: PeerId,
    pub name: String,
    pub last_message: String,
}

pub struct ChatListState {
    pub chats: Vec<ChatListItem>,
    pub selected_peer: Option<PeerId>,
}

#[derive(Clone, Debug)]
pub enum ChatListIntent {
    Select(PeerId),
    OpenSettings,
}

#[derive(Clone, Debug)]
pub enum ChatListOutput {
    ChatSelected(PeerId),
    OpenSettings,
}

pub struct ChatListFeature {
    state: ChatListState,
}

impl ChatListFeature {
    pub fn new() -> Self {
        Self {
            state: ChatListState {
                chats: vec![
                    ChatListItem {
                        id: PeerId("pavel_durov".into()),
                        name: "Pavel Durov".into(),
                        last_message: "Welcome to GPUI MVI Navigation!".into(),
                    },
                    ChatListItem {
                        id: PeerId("rust_crew".into()),
                        name: "Rust Core Team".into(),
                        last_message: "GATs and async traits are awesome.".into(),
                    },
                    ChatListItem {
                        id: PeerId("zed_industries".into()),
                        name: "Zed Industries".into(),
                        last_message: "GPUI 2.0 architecture is blazing fast.".into(),
                    },
                ],
                selected_peer: None,
            },
        }
    }
}

impl Default for ChatListFeature {
    fn default() -> Self {
        Self::new()
    }
}

impl EventEmitter<ChatListOutput> for ChatListFeature {}

impl MviComponent for ChatListFeature {
    type State = ChatListState;
    type Intent = ChatListIntent;
    type Output = ChatListOutput;

    fn state(&self) -> &Self::State {
        &self.state
    }

    fn dispatch(&mut self, intent: Self::Intent, _window: &mut Window, cx: &mut Context<Self>) {
        match intent {
            ChatListIntent::Select(peer_id) => {
                self.state.selected_peer = Some(peer_id.clone());
                cx.emit(ChatListOutput::ChatSelected(peer_id));
                cx.notify();
            }
            ChatListIntent::OpenSettings => {
                cx.emit(ChatListOutput::OpenSettings);
            }
        }
    }
}

impl Render for ChatListFeature {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let dispatcher = cx.dispatcher();

        let header = div()
            .flex()
            .items_center()
            .justify_between()
            .p_3()
            .border_b_1()
            .border_color(rgb(0x2a2a2e))
            .child(div().font_weight(gpui::FontWeight::BOLD).child("Chats"))
            .child(
                div()
                    .id("settings-btn")
                    .cursor_pointer()
                    .p_1()
                    .rounded_md()
                    .hover(|s| s.bg(rgb(0x3a3a40)))
                    .on_click(dispatcher.on_click(ChatListIntent::OpenSettings))
                    .child("⚙ Settings"),
            );

        let mut list = div().flex().flex_col().overflow_hidden();

        for chat in &self.state.chats {
            let peer = chat.id.clone();
            let is_selected = self.state.selected_peer.as_ref() == Some(&peer);
            let item_bg = if is_selected { rgb(0x2b5278) } else { rgb(0x1e1e22) };

            list = list.child(
                div()
                    .id(gpui::SharedString::from(format!("chat-item-{}", peer.0)))
                    .flex()
                    .flex_col()
                    .p_3()
                    .cursor_pointer()
                    .bg(item_bg)
                    .hover(|s| s.bg(if is_selected { rgb(0x2b5278) } else { rgb(0x25252a) }))
                    .on_click(dispatcher.on_click(ChatListIntent::Select(peer)))
                    .child(div().flex().items_center().justify_between().child(
                        div().font_weight(gpui::FontWeight::SEMIBOLD).child(chat.name.clone()),
                    ))
                    .child(
                        div().text_sm().text_color(rgb(0x8a8a92)).child(chat.last_message.clone()),
                    ),
            );
        }

        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(rgb(0x18181b))
            .text_color(rgb(0xffffff))
            .child(header)
            .child(list)
    }
}

// ============================================================================
// 3. Feature: Chat Room (`features/chat_room/`)
// ============================================================================

pub struct ChatRoomState {
    pub peer_id: PeerId,
    pub draft_text: String,
    pub messages: Vec<String>,
}

#[derive(Clone, Debug)]
pub enum ChatRoomIntent {
    UpdateDraft(String),
    SendMessage,
    OpenProfile,
}

#[derive(Clone, Debug)]
pub enum ChatRoomOutput {
    OpenProfile(PeerId),
}

pub struct ChatRoomFeature {
    state: ChatRoomState,
    focus_handle: FocusHandle,
}

impl ChatRoomFeature {
    pub fn new(peer_id: PeerId, cx: &mut Context<Self>) -> Self {
        Self {
            state: ChatRoomState {
                peer_id: peer_id.clone(),
                draft_text: format!("Draft for {}", peer_id.0),
                messages: vec![
                    format!("Hello! This is a message in {}", peer_id.0),
                    "Switching chats preserves this draft and scroll thanks to LRU cache!".into(),
                ],
            },
            focus_handle: cx.focus_handle(),
        }
    }
}

impl EventEmitter<ChatRoomOutput> for ChatRoomFeature {}

impl NavFocusable for ChatRoomFeature {
    fn focus_handle(&self, _cx: &App) -> Option<FocusHandle> {
        Some(self.focus_handle.clone())
    }
}

impl MviComponent for ChatRoomFeature {
    type State = ChatRoomState;
    type Intent = ChatRoomIntent;
    type Output = ChatRoomOutput;

    fn state(&self) -> &Self::State {
        &self.state
    }

    fn dispatch(&mut self, intent: Self::Intent, _window: &mut Window, cx: &mut Context<Self>) {
        match intent {
            ChatRoomIntent::UpdateDraft(text) => {
                self.state.draft_text = text;
                cx.notify();
            }
            ChatRoomIntent::SendMessage => {
                if !self.state.draft_text.trim().is_empty() {
                    let msg = std::mem::take(&mut self.state.draft_text);
                    self.state.messages.push(msg);
                    cx.notify();
                }
            }
            ChatRoomIntent::OpenProfile => {
                cx.emit(ChatRoomOutput::OpenProfile(self.state.peer_id.clone()));
            }
        }
    }
}

impl Render for ChatRoomFeature {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let dispatcher = cx.dispatcher();

        let top_bar = div()
            .flex()
            .items_center()
            .justify_between()
            .p_3()
            .border_b_1()
            .border_color(rgb(0x2a2a2e))
            .bg(rgb(0x18181b))
            .child(
                div()
                    .font_weight(gpui::FontWeight::BOLD)
                    .child(format!("Chat: @{}", self.state.peer_id.0)),
            )
            .child(
                div()
                    .id("profile-btn")
                    .cursor_pointer()
                    .p_1()
                    .rounded_md()
                    .hover(|s| s.bg(rgb(0x3a3a40)))
                    .on_click(dispatcher.on_click(ChatRoomIntent::OpenProfile))
                    .child("👤 View Profile"),
            );

        let mut messages_box = div().flex().flex_col().flex_1().p_4().gap_2().overflow_hidden();
        for msg in &self.state.messages {
            messages_box = messages_box.child(
                div().bg(rgb(0x2b5278)).p_2().rounded_lg().max_w(px(320.)).child(msg.clone()),
            );
        }

        let input_bar = div()
            .flex()
            .items_center()
            .p_3()
            .border_t_1()
            .border_color(rgb(0x2a2a2e))
            .bg(rgb(0x18181b))
            .gap_2()
            .child(
                div()
                    .flex_1()
                    .p_2()
                    .rounded_md()
                    .bg(rgb(0x27272a))
                    .child(format!("Draft: {}", self.state.draft_text)),
            )
            .child(
                div()
                    .id("send-btn")
                    .cursor_pointer()
                    .p_2()
                    .rounded_md()
                    .bg(rgb(0x2563eb))
                    .hover(|s| s.bg(rgb(0x1d4ed8)))
                    .on_click(dispatcher.on_click(ChatRoomIntent::SendMessage))
                    .child("Send"),
            );

        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(rgb(0x0e0e10))
            .text_color(rgb(0xffffff))
            .track_focus(&self.focus_handle)
            .child(top_bar)
            .child(messages_box)
            .child(input_bar)
    }
}

// ============================================================================
// 4. Feature: Inspector Sub-Screens (`features/inspector/`)
// ============================================================================

// --- Profile Sub-Screen ---

pub struct ProfileState {
    pub peer_id: PeerId,
    pub name: String,
    pub bio: String,
}

#[derive(Clone, Debug)]
pub enum ProfileIntent {
    ViewMedia,
}

#[derive(Clone, Debug)]
pub enum ProfileOutput {
    OpenSharedMedia(PeerId),
}

pub struct ProfileFeature {
    state: ProfileState,
    focus_handle: FocusHandle,
}

impl ProfileFeature {
    pub fn new(peer_id: PeerId, cx: &mut Context<Self>) -> Self {
        Self {
            state: ProfileState {
                peer_id: peer_id.clone(),
                name: format!("User @{}", peer_id.0),
                bio: "Senior Rust engineer building high-performance UIs in GPUI.".into(),
            },
            focus_handle: cx.focus_handle(),
        }
    }
}

impl EventEmitter<ProfileOutput> for ProfileFeature {}

impl NavFocusable for ProfileFeature {
    fn focus_handle(&self, _cx: &App) -> Option<FocusHandle> {
        Some(self.focus_handle.clone())
    }
}

impl MviComponent for ProfileFeature {
    type State = ProfileState;
    type Intent = ProfileIntent;
    type Output = ProfileOutput;

    fn state(&self) -> &Self::State {
        &self.state
    }

    fn dispatch(&mut self, intent: Self::Intent, _window: &mut Window, cx: &mut Context<Self>) {
        match intent {
            ProfileIntent::ViewMedia => {
                cx.emit(ProfileOutput::OpenSharedMedia(self.state.peer_id.clone()));
            }
        }
    }
}

impl Render for ProfileFeature {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let dispatcher = cx.dispatcher();

        div()
            .size_full()
            .flex()
            .flex_col()
            .p_4()
            .gap_3()
            .bg(rgb(0x18181b))
            .text_color(rgb(0xffffff))
            .track_focus(&self.focus_handle)
            .child(div().font_weight(gpui::FontWeight::BOLD).text_lg().child("User Info"))
            .child(div().font_weight(gpui::FontWeight::SEMIBOLD).child(self.state.name.clone()))
            .child(div().text_sm().text_color(rgb(0x9ca3af)).child(self.state.bio.clone()))
            .child(
                div()
                    .id("view-media-btn")
                    .cursor_pointer()
                    .p_2()
                    .mt_4()
                    .rounded_md()
                    .bg(rgb(0x27272a))
                    .hover(|s| s.bg(rgb(0x3f3f46)))
                    .on_click(dispatcher.on_click(ProfileIntent::ViewMedia))
                    .child("🖼 View Shared Media (Push Stack) →"),
            )
    }
}

// --- Shared Media Sub-Screen ---

pub struct SharedMediaState {
    pub peer_id: PeerId,
    pub items: Vec<String>,
}

#[derive(Clone, Debug)]
pub enum SharedMediaIntent {
    Back,
}

#[derive(Clone, Debug)]
pub enum SharedMediaOutput {
    Back,
}

pub struct SharedMediaFeature {
    state: SharedMediaState,
    focus_handle: FocusHandle,
}

impl SharedMediaFeature {
    pub fn new(peer_id: PeerId, cx: &mut Context<Self>) -> Self {
        Self {
            state: SharedMediaState {
                peer_id: peer_id.clone(),
                items: vec!["architecture_diagram.png".into(), "spec_v2.pdf".into()],
            },
            focus_handle: cx.focus_handle(),
        }
    }
}

impl EventEmitter<SharedMediaOutput> for SharedMediaFeature {}

impl NavFocusable for SharedMediaFeature {
    fn focus_handle(&self, _cx: &App) -> Option<FocusHandle> {
        Some(self.focus_handle.clone())
    }
}

impl MviComponent for SharedMediaFeature {
    type State = SharedMediaState;
    type Intent = SharedMediaIntent;
    type Output = SharedMediaOutput;

    fn state(&self) -> &Self::State {
        &self.state
    }

    fn dispatch(&mut self, intent: Self::Intent, _window: &mut Window, cx: &mut Context<Self>) {
        match intent {
            SharedMediaIntent::Back => {
                cx.emit(SharedMediaOutput::Back);
            }
        }
    }
}

impl Render for SharedMediaFeature {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let dispatcher = cx.dispatcher();

        let header = div()
            .flex()
            .items_center()
            .gap_2()
            .border_b_1()
            .border_color(rgb(0x2a2a2e))
            .pb_2()
            .child(
                div()
                    .id("back-btn")
                    .cursor_pointer()
                    .p_1()
                    .rounded_md()
                    .hover(|s| s.bg(rgb(0x3a3a40)))
                    .on_click(dispatcher.on_click(SharedMediaIntent::Back))
                    .child("← Back"),
            )
            .child(div().font_weight(gpui::FontWeight::BOLD).child("Shared Media"));

        let mut items_box = div().flex().flex_col().gap_2().mt_3();
        for item in &self.state.items {
            items_box = items_box.child(
                div().p_2().rounded_md().bg(rgb(0x27272a)).text_sm().child(format!("📁 {}", item)),
            );
        }

        div()
            .size_full()
            .flex()
            .flex_col()
            .p_4()
            .bg(rgb(0x18181b))
            .text_color(rgb(0xffffff))
            .track_focus(&self.focus_handle)
            .child(header)
            .child(items_box)
    }
}

// ============================================================================
// 5. Heterogeneous Route Host (`router_macro.rs`)
// ============================================================================

#[derive(Clone, Debug)]
pub enum InspectorOutput {
    Profile(ProfileOutput),
    Media(SharedMediaOutput),
}

crate::define_route_host! {
    pub enum InspectorHost, InspectorOutput {
        Profile: profile(ProfileFeature, ProfileOutput => InspectorOutput::Profile),
        Media: media(SharedMediaFeature, SharedMediaOutput => InspectorOutput::Media),
    }
}

// ============================================================================
// 6. Feature: Settings Modal (`features/settings/`)
// ============================================================================

pub struct SettingsState {
    pub dark_mode: bool,
}

#[derive(Clone, Debug)]
pub enum SettingsIntent {
    ToggleDarkMode,
    Close,
}

#[derive(Clone, Debug)]
pub enum SettingsOutput {
    Close,
}

pub struct SettingsModal {
    state: SettingsState,
    focus_handle: FocusHandle,
}

impl SettingsModal {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self { state: SettingsState { dark_mode: true }, focus_handle: cx.focus_handle() }
    }
}

impl EventEmitter<SettingsOutput> for SettingsModal {}

impl NavFocusable for SettingsModal {
    fn focus_handle(&self, _cx: &App) -> Option<FocusHandle> {
        Some(self.focus_handle.clone())
    }
}

impl MviComponent for SettingsModal {
    type State = SettingsState;
    type Intent = SettingsIntent;
    type Output = SettingsOutput;

    fn state(&self) -> &Self::State {
        &self.state
    }

    fn dispatch(&mut self, intent: Self::Intent, _window: &mut Window, cx: &mut Context<Self>) {
        match intent {
            SettingsIntent::ToggleDarkMode => {
                self.state.dark_mode = !self.state.dark_mode;
                cx.notify();
            }
            SettingsIntent::Close => {
                cx.emit(SettingsOutput::Close);
            }
        }
    }
}

impl Render for SettingsModal {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let dispatcher = cx.dispatcher();

        div()
            .w(px(380.))
            .p_6()
            .rounded_xl()
            .bg(rgb(0x27272a))
            .border_1()
            .border_color(rgb(0x3f3f46))
            .text_color(rgb(0xffffff))
            .track_focus(&self.focus_handle)
            .shadow_lg()
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .border_b_1()
                    .border_color(rgb(0x3f3f46))
                    .pb_3()
                    .child(
                        div().text_lg().font_weight(gpui::FontWeight::BOLD).child("Settings"),
                    )
                    .child(
                        div()
                            .id("close-settings-x")
                            .cursor_pointer()
                            .p_1()
                            .on_click(dispatcher.on_click(SettingsIntent::Close))
                            .child("✕"),
                    ),
            )
            .child(
                div().flex().items_center().justify_between().py_4().child("Dark Theme").child(
                    div()
                        .id("theme-toggle")
                        .cursor_pointer()
                        .px_3()
                        .py_1()
                        .rounded_md()
                        .bg(rgb(0x3b82f6))
                        .on_click(dispatcher.on_click(SettingsIntent::ToggleDarkMode))
                        .child(if self.state.dark_mode { "ON" } else { "OFF" }),
                ),
            )
            .child(
                div().flex().justify_end().mt_4().child(
                    div()
                        .id("close-settings-btn")
                        .cursor_pointer()
                        .px_4()
                        .py_2()
                        .rounded_md()
                        .bg(rgb(0x3f3f46))
                        .hover(|s| s.bg(rgb(0x52525b)))
                        .on_click(dispatcher.on_click(SettingsIntent::Close))
                        .child("Close"),
                ),
            )
    }
}

// ============================================================================
// 7. App / Pages Layer: Workspace Coordinator (`app/workspace/`)
// ============================================================================

/// Root coordinator and presenter (`WorkspaceStore`).
///
/// Follows FSD architecture:
/// - Owns feature entities and navigation primitives.
/// - Handles `Output` events from child features and translates them into navigation mutations.
/// - Keeps all features 100% decoupled from one another.
pub struct WorkspaceStore {
    // Features
    chat_list: Entity<ChatListFeature>,

    // Navigators
    chat_slot: SlotNavigator<ChatRoomRoute, ChatRoomFeature>,
    inspector_stack: StackNavigator<InspectorRoute, InspectorHost>,
    overlay: OverlayNavigator<ModalRoute, SettingsModal>,

    // Subscriptions
    _subscriptions: Vec<gpui::Subscription>,
}

impl WorkspaceStore {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let chat_list = cx.new(|_| ChatListFeature::new());

        // Central chat slot with LRU cache capacity = 5
        let chat_slot = SlotNavigator::new(5);

        // Right inspector stack
        let inspector_stack = StackNavigator::new_empty();

        // Modal overlay stack
        let overlay = OverlayNavigator::new();

        let mut subscriptions = Vec::new();

        // Subscribe to ChatListOutput
        subscriptions.push(cx.subscribe(&chat_list, |this, _child, event, cx| match event {
            ChatListOutput::ChatSelected(peer_id) => {
                let peer = peer_id.clone();
                // 1. Activate or retrieve from LRU cache in central panel
                this.chat_slot.activate(
                    ChatRoomRoute(peer.clone()),
                    cx,
                    |route, cx| ChatRoomFeature::new(route.0.clone(), cx),
                    |this, _child, event, cx| match event {
                        ChatRoomOutput::OpenProfile(peer) => {
                            this.open_profile(peer.clone(), cx);
                        }
                    },
                );

                // 2. Clear & push profile into the inspector stack
                this.open_profile(peer, cx);
            }
            ChatListOutput::OpenSettings => {
                this.overlay.present(
                    ModalRoute::Settings,
                    cx,
                    |_route, cx| SettingsModal::new(cx),
                    |this, _child, event, cx| match event {
                        SettingsOutput::Close => {
                            this.overlay.dismiss_top(cx);
                        }
                    },
                );
            }
        }));

        Self { chat_list, chat_slot, inspector_stack, overlay, _subscriptions: subscriptions }
    }

    /// Opens profile in the inspector stack and wires its output.
    fn open_profile(&mut self, peer_id: PeerId, cx: &mut Context<Self>) {
        self.inspector_stack.clear_and_push(
            InspectorRoute::Profile(peer_id.clone()),
            cx,
            move |route, cx| match route {
                InspectorRoute::Profile(p) => {
                    let child = cx.new(|cx| ProfileFeature::new(p.clone(), cx));
                    InspectorHost::profile(child, cx)
                }
                InspectorRoute::Media(p) => {
                    let child = cx.new(|cx| SharedMediaFeature::new(p.clone(), cx));
                    InspectorHost::media(child, cx)
                }
            },
            |this, _child, event, cx| match event {
                InspectorOutput::Profile(ProfileOutput::OpenSharedMedia(peer)) => {
                    this.open_shared_media(peer.clone(), cx);
                }
                InspectorOutput::Media(SharedMediaOutput::Back) => {
                    this.inspector_stack.pop(cx);
                }
            },
        );
    }

    /// Pushes shared media onto the inspector stack.
    fn open_shared_media(&mut self, peer_id: PeerId, cx: &mut Context<Self>) {
        self.inspector_stack.push(
            InspectorRoute::Media(peer_id.clone()),
            cx,
            move |route, cx| match route {
                InspectorRoute::Profile(p) => {
                    let child = cx.new(|cx| ProfileFeature::new(p.clone(), cx));
                    InspectorHost::profile(child, cx)
                }
                InspectorRoute::Media(p) => {
                    let child = cx.new(|cx| SharedMediaFeature::new(p.clone(), cx));
                    InspectorHost::media(child, cx)
                }
            },
            |this, _child, event, cx| match event {
                InspectorOutput::Profile(ProfileOutput::OpenSharedMedia(peer)) => {
                    this.open_shared_media(peer.clone(), cx);
                }
                InspectorOutput::Media(SharedMediaOutput::Back) => {
                    this.inspector_stack.pop(cx);
                }
            },
        );
    }
}

impl Render for WorkspaceStore {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Column 1: Left Chat List (fixed width 280px)
        let left_col = div()
            .w(px(280.))
            .h_full()
            .border_r_1()
            .border_color(rgb(0x27272a))
            .child(self.chat_list.clone());

        // Column 2: Center Active Chat Slot (flex-1)
        let center_col = div().flex_1().h_full().child(self.chat_slot.render_or_else(|| {
            div()
                .size_full()
                .flex()
                .items_center()
                .justify_center()
                .bg(rgb(0x0e0e10))
                .text_color(rgb(0x71717a))
                .child("Select a chat from the left sidebar to start messaging")
        }));

        // Column 3: Right Inspector Stack (fixed width 320px)
        let right_col = div().w(px(320.)).h_full().border_l_1().border_color(rgb(0x27272a)).child(
            if self.inspector_stack.is_empty() {
                div()
                    .size_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .bg(rgb(0x18181b))
                    .text_color(rgb(0x71717a))
                    .child("No user profile selected")
                    .into_any_element()
            } else {
                self.inspector_stack.render_active()
            },
        );

        let main_3col_layout = div()
            .size_full()
            .flex()
            .bg(rgb(0x09090b))
            .child(left_col)
            .child(center_col)
            .child(right_col);

        // Modal Overlay Layer (rendered on top if present)
        let modal_layer = self.overlay.render_overlay(Some(cx.listener(
            |this: &mut Self, _event: &ClickEvent, _window: &mut Window, cx| {
                this.overlay.dismiss_top(cx);
            },
        )));

        let mut root = div().size_full().relative().child(main_3col_layout);

        if let Some(modal) = modal_layer {
            root = root.child(modal);
        }

        root
    }
}
