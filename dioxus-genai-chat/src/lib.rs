//! A configurable [Dioxus] + [Bulma] chat UI.
//!
//! The crate provides a [`ChatSurface`] component plus a small data model
//! ([`ChatTranscript`], [`ChatMessage`], [`ChatMessagePayload`]) for rendering
//! chat conversations, including:
//!
//! - chained, collapsible reasoning timelines ([`Reasoning`]),
//! - inline message controls — buttons, selectors, toggles ([`InlineControl`]),
//!   surfaced through [`ChatSurface`]'s `on_action` handler,
//! - spinning status indicators, tool calls, progress, and errors.
//!
//! With the default `genai` feature enabled, [`ChatTranscript::to_genai_request`]
//! converts a transcript into a [`genai`] chat request. Disable default features
//! to build for `wasm32-unknown-unknown` (the web target), where `genai` is not
//! available.
//!
//! [Dioxus]: https://dioxuslabs.com/
//! [Bulma]: https://bulma.io/
//! [`genai`]: https://docs.rs/genai/

use dioxus::prelude::*;
use dioxus_bulma::prelude::*;
#[cfg(feature = "genai")]
use genai::chat::{ChatMessage as GenAiChatMessage, ChatRequest, ChatRole as GenAiChatRole, ToolResponse};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChatRole {
    System,
    User,
    Assistant,
    Tool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ToolCallStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

impl ToolCallStatus {
    pub fn as_label(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Running => "running",
            Self::Completed => "completed",
            Self::Failed => "failed",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolCall {
    pub name: String,
    pub arguments: Value,
    pub status: ToolCallStatus,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProgressState {
    pub label: String,
    pub percent: f32,
}

/// Lifecycle of a single step inside a chained reasoning trace.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StepStatus {
    /// Not started yet.
    Pending,
    /// Currently running.
    Active,
    /// Finished successfully.
    Done,
    /// Finished with an error.
    Failed,
}

/// A single phase in a chained "thinking" trace (à la VS Code agent steps).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReasoningStep {
    pub title: String,
    /// Optional secondary line (e.g. a file path, a short result).
    #[serde(default)]
    pub detail: Option<String>,
    pub status: StepStatus,
}

impl ReasoningStep {
    pub fn new(title: impl Into<String>, status: StepStatus) -> Self {
        Self { title: title.into(), detail: None, status }
    }

    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }
}

/// A collapsible group of chained reasoning steps shown as a connected timeline.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Reasoning {
    /// One-line summary shown on the collapsed header (e.g. "Thought for 4s").
    pub summary: String,
    pub steps: Vec<ReasoningStep>,
    /// Whether the panel starts collapsed.
    #[serde(default)]
    pub collapsed: bool,
}

/// Visual emphasis for an inline control.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ControlStyle {
    Primary,
    #[default]
    Neutral,
    Danger,
    Ghost,
}

/// An option in an inline [`InlineControl::Select`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SelectOption {
    pub value: String,
    pub label: String,
}

impl SelectOption {
    pub fn new(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self { value: value.into(), label: label.into() }
    }
}

/// A small interactive element rendered inline inside a message.
///
/// Interactions are surfaced through the [`ChatSurface`] `on_action` handler;
/// the component is "controlled", so update the transcript in response to events.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum InlineControl {
    Button {
        id: String,
        label: String,
        #[serde(default)]
        style: ControlStyle,
        #[serde(default)]
        disabled: bool,
    },
    Select {
        id: String,
        #[serde(default)]
        label: Option<String>,
        options: Vec<SelectOption>,
        #[serde(default)]
        selected: Option<String>,
    },
    Toggle {
        id: String,
        label: String,
        #[serde(default)]
        value: bool,
    },
}

/// The kind of interaction that produced a [`ControlEvent`].
#[derive(Debug, Clone, PartialEq)]
pub enum ControlValue {
    /// A button was clicked.
    Clicked,
    /// A select changed to the given option value.
    Selected(String),
    /// A toggle changed to the given state.
    Toggled(bool),
}

/// Emitted when the user interacts with an [`InlineControl`].
#[derive(Debug, Clone, PartialEq)]
pub struct ControlEvent {
    /// The `id` of the control that was interacted with.
    pub id: String,
    pub value: ControlValue,
}

/// Whether a piece of attached context is a file or a directory.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContextKind {
    File,
    Directory,
}

/// A piece of context (a file or directory) attached to the next message.
///
/// The pending list is owned by the caller and passed to [`ChatSurface`] via
/// `attachments`; the component renders it and emits [`ContextEvent`]s. How files
/// and directories are actually chosen (native dialog, browser input, typed path)
/// is up to the caller — see the `on_context` handler.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContextItem {
    /// Stable id used when the user removes the item.
    pub id: String,
    /// Display label, e.g. a file name or directory path.
    pub label: String,
    pub kind: ContextKind,
}

impl ContextItem {
    pub fn file(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self { id: id.into(), label: label.into(), kind: ContextKind::File }
    }

    pub fn directory(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self { id: id.into(), label: label.into(), kind: ContextKind::Directory }
    }
}

/// Emitted when the user adds or removes context via the input area.
///
/// Handle this in [`ChatSurface`]'s `on_context` and update the `attachments`
/// list you pass back in (a controlled component, like [`InlineControl`]).
#[derive(Debug, Clone, PartialEq)]
pub enum ContextEvent {
    /// The user asked to attach file(s); open a picker and add [`ContextItem`]s.
    AddFilesRequested,
    /// The user asked to add a directory as context.
    AddDirectoryRequested,
    /// The user removed the attached item with this `id`.
    Remove(String),
}

/// The role of a single line in a [`FileDiff`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiffKind {
    /// An added line (rendered green, prefixed `+`).
    Added,
    /// A removed line (rendered red, prefixed `-`).
    Removed,
    /// An unchanged context line.
    Context,
}

/// A single line within a unified [`FileDiff`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiffLine {
    pub kind: DiffKind,
    pub content: String,
}

impl DiffLine {
    pub fn added(content: impl Into<String>) -> Self {
        Self { kind: DiffKind::Added, content: content.into() }
    }

    pub fn removed(content: impl Into<String>) -> Self {
        Self { kind: DiffKind::Removed, content: content.into() }
    }

    pub fn context(content: impl Into<String>) -> Self {
        Self { kind: DiffKind::Context, content: content.into() }
    }
}

/// A unified diff for a single file, rendered with an apply animation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FileDiff {
    /// File path shown in the diff header.
    pub path: String,
    pub lines: Vec<DiffLine>,
    /// When `true`, changed lines stream in (staggered) and flash on apply.
    /// Set `false` to render the final state with no animation.
    #[serde(default = "default_true")]
    pub animate: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ChatMessagePayload {
    Text(String),
    Markdown(String),
    /// A chained, collapsible reasoning/thinking trace.
    Reasoning(Reasoning),
    ToolCall(ToolCall),
    ToolResult { name: String, content: String },
    Progress(ProgressState),
    /// A spinning status line with a custom label (e.g. "Running tests…").
    Status(String),
    /// A row of inline controls (buttons / selectors / toggles).
    Controls(Vec<InlineControl>),
    /// A unified file diff, optionally animated as it is applied.
    Diff(FileDiff),
    Typing,
    Error(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: ChatRole,
    pub payload: ChatMessagePayload,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct ChatTranscript {
    pub messages: Vec<ChatMessage>,
}

impl ChatTranscript {
    pub fn push(&mut self, role: ChatRole, payload: ChatMessagePayload) {
        self.messages.push(ChatMessage { role, payload });
    }

    #[cfg(feature = "genai")]
    pub fn to_genai_request(&self) -> ChatRequest {
        let mut system: Option<String> = None;
        let mut messages = Vec::new();

        for message in &self.messages {
            match (&message.role, &message.payload) {
                (ChatRole::System, ChatMessagePayload::Text(content))
                | (ChatRole::System, ChatMessagePayload::Markdown(content)) => {
                    if system.is_none() {
                        system = Some(content.clone());
                    } else {
                        messages.push(GenAiChatMessage::system(content.clone()));
                    }
                }
                (ChatRole::User, ChatMessagePayload::Text(content))
                | (ChatRole::User, ChatMessagePayload::Markdown(content)) => {
                    messages.push(GenAiChatMessage::user(content.clone()));
                }
                (ChatRole::Assistant, ChatMessagePayload::Text(content))
                | (ChatRole::Assistant, ChatMessagePayload::Markdown(content)) => {
                    messages.push(GenAiChatMessage::assistant(content.clone()));
                }
                (ChatRole::Tool, ChatMessagePayload::Text(content))
                | (ChatRole::Tool, ChatMessagePayload::Markdown(content)) => {
                    messages.push(GenAiChatMessage {
                        role: GenAiChatRole::Tool,
                        content: format!("Tool output: {content}").into(),
                        options: None,
                    });
                }
                (_, ChatMessagePayload::ToolResult { name, content }) => {
                    messages.push(GenAiChatMessage::from(
                        ToolResponse::new(name.clone(), content.clone()),
                    ));
                }
                (_, ChatMessagePayload::ToolCall(call)) => {
                    messages.push(GenAiChatMessage::assistant(format!(
                        "Tool call requested: {} with {}",
                        call.name, call.arguments
                    )));
                }
                (_, ChatMessagePayload::Error(content)) => {
                    messages.push(GenAiChatMessage::assistant(content.clone()));
                }
                (_, ChatMessagePayload::Progress(_))
                | (_, ChatMessagePayload::Reasoning(_))
                | (_, ChatMessagePayload::Status(_))
                | (_, ChatMessagePayload::Controls(_))
                | (_, ChatMessagePayload::Diff(_))
                | (_, ChatMessagePayload::Typing) => {}
            }
        }

        let mut request = ChatRequest::new(messages);
        request.system = system;
        request
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChatControls {
    pub show_input: bool,
    pub show_send_button: bool,
    pub show_stop_button: bool,
    pub show_retry_button: bool,
    pub show_clear_button: bool,
    pub input_enabled: bool,
    pub placeholder: String,
    /// Show the "attach files" affordance. Off by default — enabling it requires
    /// wiring [`ChatSurface`]'s `on_context` handler to do the picking.
    pub allow_file_attachments: bool,
    /// Show the "add directory" affordance.
    pub allow_directory_context: bool,
    /// Label for the attach-files button.
    pub attach_files_label: String,
    /// Label for the add-directory button.
    pub add_directory_label: String,
}

impl Default for ChatControls {
    fn default() -> Self {
        Self {
            show_input: true,
            show_send_button: true,
            show_stop_button: true,
            show_retry_button: true,
            show_clear_button: true,
            input_enabled: true,
            placeholder: "Ask anything, or invoke a tool...".to_string(),
            allow_file_attachments: false,
            allow_directory_context: false,
            attach_files_label: "📎 Add files".to_string(),
            add_directory_label: "📁 Add folder".to_string(),
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct ChatSurfaceProps {
    pub transcript: ChatTranscript,
    #[props(default)]
    pub controls: ChatControls,
    #[props(default)]
    pub title: Option<String>,
    #[props(default)]
    pub theme: Option<BulmaTheme>,
    /// Fired when the user interacts with an inline [`InlineControl`].
    #[props(default)]
    pub on_action: EventHandler<ControlEvent>,
    /// Context (files/directories) currently attached to the next message.
    /// Owned by the caller; render-only here.
    #[props(default)]
    pub attachments: Vec<ContextItem>,
    /// Fired when the user adds or removes context via the input area.
    #[props(default)]
    pub on_context: EventHandler<ContextEvent>,
}

/// Scoped styling for the compact captions and the chained reasoning timeline.
/// Colors are pulled from Bulma CSS variables so it adapts to light/dark themes.
const CHAT_SURFACE_CSS: &str = r#"
.gc-msg { margin-bottom: 1.1rem; }
.gc-caption { display: flex; align-items: center; gap: 0.4rem; margin-bottom: 0.25rem; }
.gc-dot { width: 0.5rem; height: 0.5rem; border-radius: 50%; display: inline-block; flex: none; }
.gc-msg-system .gc-dot { background: #b5b5b5; }
.gc-msg-user .gc-dot { background: var(--bulma-primary, #00d1b2); }
.gc-msg-assistant .gc-dot { background: var(--bulma-link, #485fc7); }
.gc-msg-tool .gc-dot { background: var(--bulma-info, #3e8ed0); }
.gc-role { font-size: 0.7rem; font-weight: 600; letter-spacing: 0.04em; text-transform: uppercase; color: var(--bulma-text-weak, #7a7a7a); }
.gc-body { padding-left: 0.9rem; border-left: 2px solid var(--bulma-border-weak, #ededed); margin-left: 0.24rem; }
.gc-body > p { margin: 0; }
.gc-muted { color: var(--bulma-text-weak, #7a7a7a); }

.gc-code { padding: 0.6rem 0.75rem; border-radius: 0.5rem; background: var(--bulma-scheme-main-bis, #f5f7fa); font-size: 0.8rem; overflow-x: auto; margin-top: 0.4rem; white-space: pre; }
.gc-tool-line { display: flex; align-items: center; gap: 0.5rem; }
.gc-tool-name { font-family: monospace; font-weight: 600; font-size: 0.85rem; color: var(--bulma-text, #363636); }
.gc-chip { font-size: 0.65rem; text-transform: uppercase; letter-spacing: 0.03em; padding: 0.1rem 0.45rem; border-radius: 999px; font-weight: 600; }
.gc-chip-pending { background: rgba(255,183,15,.18); color: #b87503; }
.gc-chip-active { background: rgba(62,142,208,.18); color: #2b6cb0; }
.gc-chip-done { background: rgba(72,199,142,.18); color: #257953; }
.gc-chip-failed { background: rgba(241,70,104,.18); color: #c81e4b; }

.gc-reasoning { border: 1px solid var(--bulma-border-weak, #ededed); border-radius: 0.6rem; background: var(--bulma-scheme-main-bis, #f8f9fb); padding: 0.5rem 0.75rem; }
.gc-reasoning-summary { cursor: pointer; display: flex; align-items: center; gap: 0.5rem; list-style: none; font-size: 0.78rem; color: var(--bulma-text-weak, #7a7a7a); font-weight: 600; }
.gc-reasoning-summary::-webkit-details-marker { display: none; }
.gc-chevron { width: 0; height: 0; border-left: 5px solid currentColor; border-top: 4px solid transparent; border-bottom: 4px solid transparent; transition: transform .15s ease; flex: none; }
.gc-reasoning[open] .gc-chevron { transform: rotate(90deg); }

.gc-timeline { list-style: none; margin: 0.6rem 0 0.15rem; padding: 0; }
.gc-step { position: relative; padding: 0 0 0.7rem 1.5rem; }
.gc-step:last-child { padding-bottom: 0; }
.gc-step::before { content: ""; position: absolute; left: 0.4rem; top: 1.05rem; bottom: -0.1rem; width: 2px; background: var(--bulma-border, #dbdbdb); }
.gc-step:last-child::before { display: none; }
.gc-step-marker { position: absolute; left: 0; top: 0.05rem; width: 0.85rem; height: 0.85rem; display: inline-flex; align-items: center; justify-content: center; font-size: 0.72rem; line-height: 1; }
.gc-step-pending .gc-step-marker { color: var(--bulma-text-weak, #b5b5b5); }
.gc-step-active .gc-step-marker { color: var(--bulma-info, #3e8ed0); animation: gc-pulse 1.2s ease-in-out infinite; }
.gc-step-done .gc-step-marker { color: var(--bulma-success, #48c78e); }
.gc-step-failed .gc-step-marker { color: var(--bulma-danger, #f14668); }
.gc-step-content { display: flex; flex-direction: column; }
.gc-step-title { font-size: 0.85rem; color: var(--bulma-text, #363636); }
.gc-step-pending .gc-step-title { color: var(--bulma-text-weak, #7a7a7a); }
.gc-step-detail { font-size: 0.75rem; color: var(--bulma-text-weak, #7a7a7a); }

.gc-status { display: flex; align-items: center; gap: 0.5rem; color: var(--bulma-text-weak, #7a7a7a); font-size: 0.85rem; }
.gc-spinner { width: 0.95rem; height: 0.95rem; border: 2px solid currentColor; border-top-color: transparent; border-radius: 50%; display: inline-block; flex: none; animation: gc-spin 0.7s linear infinite; }
.gc-spinner-sm { width: 0.75rem; height: 0.75rem; border-width: 2px; }
.gc-step-active .gc-step-marker .gc-spinner { color: var(--bulma-info, #3e8ed0); }

.gc-controls { display: flex; flex-wrap: wrap; gap: 0.5rem; align-items: center; }
.gc-control-group { display: inline-flex; align-items: center; gap: 0.35rem; }
.gc-control-label { font-size: 0.78rem; color: var(--bulma-text-weak, #7a7a7a); }
.gc-toggle { display: inline-flex; align-items: center; gap: 0.35rem; font-size: 0.85rem; }

.gc-input-box { border: 1px solid var(--bulma-border, #dbdbdb); border-radius: 0.7rem; background: var(--bulma-scheme-main, #fff); padding: 0.5rem 0.6rem; transition: border-color 0.15s ease, box-shadow 0.15s ease; }
.gc-input-box:focus-within { border-color: var(--bulma-link, #485fc7); box-shadow: 0 0 0 2px rgba(72, 95, 199, 0.12); }
.gc-input-box textarea, .gc-input-box .textarea { border: none !important; box-shadow: none !important; background: transparent; padding: 0.1rem; min-height: 3.5rem; resize: vertical; }

.gc-attachments { display: flex; flex-wrap: wrap; gap: 0.4rem; margin-bottom: 0.4rem; }
.gc-attachment { display: inline-flex; align-items: center; gap: 0.35rem; padding: 0.1rem 0.25rem 0.1rem 0.5rem; border-radius: 999px; background: var(--bulma-scheme-main-bis, #f5f7fa); border: 1px solid var(--bulma-border-weak, #ededed); font-size: 0.76rem; }
.gc-attachment-kind { font-size: 0.8rem; line-height: 1; }
.gc-attachment-label { color: var(--bulma-text, #363636); }
.gc-attachment-remove { cursor: pointer; border: none; background: none; color: var(--bulma-text-weak, #7a7a7a); font-size: 1rem; line-height: 1; padding: 0 0.15rem; }
.gc-attachment-remove:hover { color: var(--bulma-danger, #f14668); }
.gc-context-actions { display: flex; flex-wrap: wrap; gap: 0.15rem; margin-top: 0.3rem; }
.gc-context-btn { display: inline-flex; align-items: center; gap: 0.3rem; cursor: pointer; border: none; background: transparent; color: var(--bulma-text-weak, #7a7a7a); font-size: 0.76rem; padding: 0.2rem 0.4rem; border-radius: 0.4rem; line-height: 1; transition: background 0.12s ease, color 0.12s ease; }
.gc-context-btn:hover:not(:disabled) { background: var(--bulma-scheme-main-bis, #f0f1f4); color: var(--bulma-text, #363636); }
.gc-context-btn:disabled { opacity: 0.5; cursor: default; }
.gc-diff { border: 1px solid var(--bulma-border-weak, #ededed); border-radius: 0.6rem; overflow: hidden; font-size: 0.8rem; }
.gc-diff-header { display: flex; align-items: center; gap: 0.6rem; padding: 0.4rem 0.7rem; background: var(--bulma-scheme-main-bis, #f5f7fa); border-bottom: 1px solid var(--bulma-border-weak, #ededed); }
.gc-diff-file { font-family: monospace; font-weight: 600; color: var(--bulma-text, #363636); }
.gc-diff-stat-add { color: #257953; font-weight: 600; }
.gc-diff-stat-del { color: #c81e4b; font-weight: 600; }
.gc-diff-body { font-family: monospace; padding: 0.3rem 0; overflow-x: auto; }
.gc-diff-line { display: flex; gap: 0.5rem; padding: 0 0.7rem; white-space: pre; line-height: 1.5; }
.gc-diff-gutter { width: 0.8rem; text-align: center; flex: none; color: var(--bulma-text-weak, #b5b5b5); user-select: none; }
.gc-diff-code { flex: 1 1 auto; }
.gc-diff-added { background: rgba(72, 199, 142, 0.14); box-shadow: inset 2px 0 0 #48c78e; }
.gc-diff-added .gc-diff-gutter { color: #257953; }
.gc-diff-removed { background: rgba(241, 70, 104, 0.14); box-shadow: inset 2px 0 0 #f14668; }
.gc-diff-removed .gc-diff-gutter { color: #c81e4b; }
.gc-diff-context { color: var(--bulma-text-weak, #7a7a7a); }
.gc-diff-animate { animation: gc-diff-reveal 0.28s ease both; }
.gc-diff-animate.gc-diff-added { animation: gc-diff-reveal 0.28s ease both, gc-flash-add 1.1s ease; }
.gc-diff-animate.gc-diff-removed { animation: gc-diff-reveal 0.28s ease both, gc-flash-remove 1.1s ease; }

@keyframes gc-pulse { 0%, 100% { opacity: 1; } 50% { opacity: 0.3; } }
@keyframes gc-spin { to { transform: rotate(360deg); } }
@keyframes gc-diff-reveal { from { opacity: 0; transform: translateY(-3px); } to { opacity: 1; transform: none; } }
@keyframes gc-flash-add { 0% { background: rgba(72, 199, 142, 0.55); } 100% { background: rgba(72, 199, 142, 0.14); } }
@keyframes gc-flash-remove { 0% { background: rgba(241, 70, 104, 0.55); } 100% { background: rgba(241, 70, 104, 0.14); } }
"#;

#[component]
pub fn ChatSurface(props: ChatSurfaceProps) -> Element {
    let title = props
        .title
        .clone()
        .unwrap_or_else(|| "Dioxus GenAI Chat".to_string());
    let on_context = props.on_context;
    let show_context_actions =
        props.controls.allow_file_attachments || props.controls.allow_directory_context;

    rsx! {
        BulmaProvider {
            theme: props.theme,
            load_bulma_css: true,
            style { dangerous_inner_html: CHAT_SURFACE_CSS }
            Section {
                class: "py-5",
                Container {
                    class: "is-max-desktop",
                    BulmaBox {
                        style: "border-radius: 16px; box-shadow: 0 12px 40px rgba(10, 10, 10, 0.1);",
                        BulmaTitle {
                            size: TitleSize::Is4,
                            "{title}"
                        }

                        for (idx, message) in props.transcript.messages.iter().enumerate() {
                            ChatBubble {
                                key: "{idx}",
                                message: message.clone(),
                                on_action: props.on_action,
                            }
                        }

                        if props.controls.show_input {
                            div {
                                class: "gc-input-box",
                                if !props.attachments.is_empty() {
                                    div {
                                        class: "gc-attachments",
                                        for item in props.attachments.iter() {
                                            {
                                                let id = item.id.clone();
                                                rsx! {
                                                    span {
                                                        key: "{item.id}",
                                                        class: "gc-attachment",
                                                        span { class: "gc-attachment-kind", "{context_kind_icon(item.kind)}" }
                                                        span { class: "gc-attachment-label", "{item.label}" }
                                                        button {
                                                            class: "gc-attachment-remove",
                                                            title: "Remove",
                                                            onclick: move |_| on_context.call(ContextEvent::Remove(id.clone())),
                                                            "×"
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                Textarea {
                                    value: String::new(),
                                    placeholder: props.controls.placeholder.clone(),
                                    rows: 3,
                                    disabled: !props.controls.input_enabled,
                                    readonly: !props.controls.input_enabled,
                                }
                                if show_context_actions {
                                    div {
                                        class: "gc-context-actions",
                                        if props.controls.allow_file_attachments {
                                            button {
                                                class: "gc-context-btn",
                                                disabled: !props.controls.input_enabled,
                                                onclick: move |_| on_context.call(ContextEvent::AddFilesRequested),
                                                "{props.controls.attach_files_label}"
                                            }
                                        }
                                        if props.controls.allow_directory_context {
                                            button {
                                                class: "gc-context-btn",
                                                disabled: !props.controls.input_enabled,
                                                onclick: move |_| on_context.call(ContextEvent::AddDirectoryRequested),
                                                "{props.controls.add_directory_label}"
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        Buttons {
                            if props.controls.show_send_button {
                                Button {
                                    color: BulmaColor::Primary,
                                    "Send"
                                }
                            }
                            if props.controls.show_stop_button {
                                Button {
                                    color: BulmaColor::Warning,
                                    outlined: true,
                                    "Stop"
                                }
                            }
                            if props.controls.show_retry_button {
                                Button {
                                    color: BulmaColor::Info,
                                    outlined: true,
                                    "Retry"
                                }
                            }
                            if props.controls.show_clear_button {
                                Button {
                                    color: BulmaColor::Danger,
                                    outlined: true,
                                    "Clear"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct ChatBubbleProps {
    message: ChatMessage,
    on_action: EventHandler<ControlEvent>,
}

#[component]
fn ChatBubble(props: ChatBubbleProps) -> Element {
    let role = &props.message.role;

    rsx! {
        div {
            class: "gc-msg gc-msg-{role_slug(role)}",
            div {
                class: "gc-caption",
                span { class: "gc-dot" }
                span { class: "gc-role", "{role_label(role)}" }
            }
            div {
                class: "gc-body",
                {match &props.message.payload {
                    ChatMessagePayload::Text(content) | ChatMessagePayload::Markdown(content) => rsx! {
                        p { "{content}" }
                    },
                    ChatMessagePayload::Reasoning(reasoning) => rsx! {
                        ReasoningPanel { reasoning: reasoning.clone() }
                    },
                    ChatMessagePayload::Status(label) => rsx! {
                        div {
                            class: "gc-status",
                            span { class: "gc-spinner" }
                            span { "{label}" }
                        }
                    },
                    ChatMessagePayload::Controls(controls) => rsx! {
                        ControlBar { controls: controls.clone(), on_action: props.on_action }
                    },
                    ChatMessagePayload::Diff(diff) => rsx! {
                        DiffView { diff: diff.clone() }
                    },
                    ChatMessagePayload::Typing => rsx! {
                        div {
                            class: "gc-status",
                            span { class: "gc-spinner" }
                            span { "Thinking…" }
                        }
                    },
                    ChatMessagePayload::Error(content) => rsx! {
                        p { class: "has-text-danger has-text-weight-semibold", "{content}" }
                    },
                    ChatMessagePayload::Progress(progress) => rsx! {
                        div {
                            p { class: "gc-muted mb-1", "{progress.label}" }
                            Progress {
                                color: BulmaColor::Info,
                                value: progress.percent.clamp(0.0, 100.0),
                                max: 100.0,
                                "{progress.percent.clamp(0.0, 100.0).round()}%"
                            }
                        }
                    },
                    ChatMessagePayload::ToolCall(call) => {
                        let args = serde_json::to_string_pretty(&call.arguments)
                            .unwrap_or_else(|_| call.arguments.to_string());
                        let running = matches!(call.status, ToolCallStatus::Running);
                        rsx! {
                            div {
                                div {
                                    class: "gc-tool-line",
                                    span { class: "gc-tool-name", "{call.name}" }
                                    if running {
                                        span { class: "gc-spinner gc-spinner-sm" }
                                    }
                                    span {
                                        class: "gc-chip gc-chip-{step_slug(tool_step_status(&call.status))}",
                                        "{call.status.as_label()}"
                                    }
                                }
                                pre { class: "gc-code", "{args}" }
                            }
                        }
                    }
                    ChatMessagePayload::ToolResult { name, content } => rsx! {
                        div {
                            div {
                                class: "gc-tool-line",
                                span { class: "gc-tool-name", "{name}" }
                                span { class: "gc-chip gc-chip-done", "result" }
                            }
                            pre { class: "gc-code", "{content}" }
                        }
                    },
                }}
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct ControlBarProps {
    controls: Vec<InlineControl>,
    on_action: EventHandler<ControlEvent>,
}

/// A row of small inline controls (buttons, selectors, toggles).
#[component]
fn ControlBar(props: ControlBarProps) -> Element {
    let on_action = props.on_action;

    rsx! {
        div {
            class: "gc-controls",
            for (idx, control) in props.controls.iter().enumerate() {
                {match control {
                    InlineControl::Button { id, label, style, disabled } => {
                        let id = id.clone();
                        rsx! {
                            button {
                                key: "{idx}",
                                class: "button is-small {control_style_class(*style)}",
                                disabled: *disabled,
                                onclick: move |_| on_action.call(ControlEvent {
                                    id: id.clone(),
                                    value: ControlValue::Clicked,
                                }),
                                "{label}"
                            }
                        }
                    }
                    InlineControl::Select { id, label, options, selected } => {
                        let id = id.clone();
                        rsx! {
                            div {
                                key: "{idx}",
                                class: "gc-control-group",
                                if let Some(label) = label {
                                    span { class: "gc-control-label", "{label}" }
                                }
                                div {
                                    class: "select is-small",
                                    select {
                                        onchange: move |evt| on_action.call(ControlEvent {
                                            id: id.clone(),
                                            value: ControlValue::Selected(evt.value()),
                                        }),
                                        for opt in options.iter() {
                                            option {
                                                key: "{opt.value}",
                                                value: "{opt.value}",
                                                selected: selected.as_deref() == Some(opt.value.as_str()),
                                                "{opt.label}"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    InlineControl::Toggle { id, label, value } => {
                        let id = id.clone();
                        rsx! {
                            label {
                                key: "{idx}",
                                class: "checkbox gc-toggle",
                                input {
                                    r#type: "checkbox",
                                    checked: *value,
                                    onchange: move |evt| on_action.call(ControlEvent {
                                        id: id.clone(),
                                        value: ControlValue::Toggled(evt.checked()),
                                    }),
                                }
                                span { "{label}" }
                            }
                        }
                    }
                }}
            }
        }
    }
}

fn control_style_class(style: ControlStyle) -> &'static str {
    match style {
        ControlStyle::Primary => "is-primary",
        ControlStyle::Neutral => "",
        ControlStyle::Danger => "is-danger is-light",
        ControlStyle::Ghost => "is-ghost",
    }
}

#[derive(Props, Clone, PartialEq)]
struct DiffViewProps {
    diff: FileDiff,
}

/// Renders a unified file diff. When `diff.animate` is set, changed lines stream
/// in top-to-bottom (staggered) and flash green/red as they are "applied".
#[component]
fn DiffView(props: DiffViewProps) -> Element {
    let diff = &props.diff;
    let animate = diff.animate;
    let added = diff.lines.iter().filter(|l| l.kind == DiffKind::Added).count();
    let removed = diff.lines.iter().filter(|l| l.kind == DiffKind::Removed).count();

    rsx! {
        div {
            class: "gc-diff",
            div {
                class: "gc-diff-header",
                span { class: "gc-diff-file", "{diff.path}" }
                span { class: "gc-diff-stat-add", "+{added}" }
                span { class: "gc-diff-stat-del", "-{removed}" }
            }
            div {
                class: "gc-diff-body",
                for (idx, line) in diff.lines.iter().enumerate() {
                    {
                        let kind = line.kind;
                        let class = if animate {
                            format!("gc-diff-line gc-diff-{} gc-diff-animate", diff_kind_slug(kind))
                        } else {
                            format!("gc-diff-line gc-diff-{}", diff_kind_slug(kind))
                        };
                        // Stagger the reveal so the diff appears to apply top-to-bottom.
                        let style = if animate {
                            format!("animation-delay: {}ms", idx * 45)
                        } else {
                            String::new()
                        };
                        rsx! {
                            div {
                                key: "{idx}",
                                class: "{class}",
                                style: "{style}",
                                span { class: "gc-diff-gutter", "{diff_sign(kind)}" }
                                span { class: "gc-diff-code", "{line.content}" }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn diff_kind_slug(kind: DiffKind) -> &'static str {
    match kind {
        DiffKind::Added => "added",
        DiffKind::Removed => "removed",
        DiffKind::Context => "context",
    }
}

fn diff_sign(kind: DiffKind) -> &'static str {
    match kind {
        DiffKind::Added => "+",
        DiffKind::Removed => "-",
        DiffKind::Context => " ",
    }
}

#[derive(Props, Clone, PartialEq)]
struct ReasoningPanelProps {
    reasoning: Reasoning,
}

/// VS Code-style collapsible chain of reasoning steps rendered as a timeline.
#[component]
fn ReasoningPanel(props: ReasoningPanelProps) -> Element {
    let reasoning = &props.reasoning;

    rsx! {
        details {
            class: "gc-reasoning",
            open: !reasoning.collapsed,
            summary {
                class: "gc-reasoning-summary",
                span { class: "gc-chevron" }
                span { class: "gc-reasoning-title", "{reasoning.summary}" }
            }
            ol {
                class: "gc-timeline",
                for (idx, step) in reasoning.steps.iter().enumerate() {
                    li {
                        key: "{idx}",
                        class: "gc-step gc-step-{step_slug(step.status)}",
                        span {
                            class: "gc-step-marker",
                            if step.status == StepStatus::Active {
                                span { class: "gc-spinner gc-spinner-sm" }
                            } else {
                                "{step_marker(step.status)}"
                            }
                        }
                        div {
                            class: "gc-step-content",
                            span { class: "gc-step-title", "{step.title}" }
                            if let Some(detail) = &step.detail {
                                span { class: "gc-step-detail", "{detail}" }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn context_kind_icon(kind: ContextKind) -> &'static str {
    match kind {
        ContextKind::File => "📄",
        ContextKind::Directory => "📁",
    }
}

fn role_label(role: &ChatRole) -> &'static str {
    match role {
        ChatRole::System => "System",
        ChatRole::User => "User",
        ChatRole::Assistant => "Assistant",
        ChatRole::Tool => "Tool",
    }
}

fn role_slug(role: &ChatRole) -> &'static str {
    match role {
        ChatRole::System => "system",
        ChatRole::User => "user",
        ChatRole::Assistant => "assistant",
        ChatRole::Tool => "tool",
    }
}

fn step_slug(status: StepStatus) -> &'static str {
    match status {
        StepStatus::Pending => "pending",
        StepStatus::Active => "active",
        StepStatus::Done => "done",
        StepStatus::Failed => "failed",
    }
}

fn step_marker(status: StepStatus) -> &'static str {
    match status {
        StepStatus::Pending => "○",
        StepStatus::Active => "●",
        StepStatus::Done => "✓",
        StepStatus::Failed => "✕",
    }
}

fn tool_step_status(status: &ToolCallStatus) -> StepStatus {
    match status {
        ToolCallStatus::Pending => StepStatus::Pending,
        ToolCallStatus::Running => StepStatus::Active,
        ToolCallStatus::Completed => StepStatus::Done,
        ToolCallStatus::Failed => StepStatus::Failed,
    }
}

pub fn sample_transcript() -> ChatTranscript {
    let mut transcript = ChatTranscript::default();

    transcript.push(
        ChatRole::System,
        ChatMessagePayload::Text("You are an expert Rust assistant.".to_string()),
    );
    transcript.push(
        ChatRole::User,
        ChatMessagePayload::Text("Summarize the latest telemetry report.".to_string()),
    );
    transcript.push(
        ChatRole::Assistant,
        ChatMessagePayload::Reasoning(Reasoning {
            summary: "Worked through 3 steps".to_string(),
            collapsed: false,
            steps: vec![
                ReasoningStep::new("Understood the request", StepStatus::Done)
                    .with_detail("Summarize the last 24h of telemetry"),
                ReasoningStep::new("Decided to fetch the report", StepStatus::Done)
                    .with_detail("Tool: fetch_report(source = telemetry)"),
                ReasoningStep::new("Waiting on tool output", StepStatus::Active),
            ],
        }),
    );
    transcript.push(
        ChatRole::Assistant,
        ChatMessagePayload::ToolCall(ToolCall {
            name: "fetch_report".to_string(),
            arguments: serde_json::json!({"source": "telemetry", "period": "24h"}),
            status: ToolCallStatus::Running,
        }),
    );
    transcript.push(
        ChatRole::Tool,
        ChatMessagePayload::Progress(ProgressState {
            label: "Fetching report data".to_string(),
            percent: 74.0,
        }),
    );
    transcript.push(
        ChatRole::Tool,
        ChatMessagePayload::ToolResult {
            name: "fetch_report".to_string(),
            content: "Error rate dropped by 14% while latency remained stable.".to_string(),
        },
    );
    transcript.push(
        ChatRole::Tool,
        ChatMessagePayload::Status("Generating summary…".to_string()),
    );
    transcript.push(
        ChatRole::Assistant,
        ChatMessagePayload::Markdown(
            "### Summary\n- Error rate improved by **14%**\n- Latency remained stable".to_string(),
        ),
    );
    transcript.push(
        ChatRole::Assistant,
        ChatMessagePayload::Diff(FileDiff {
            path: "src/alerts.rs".to_string(),
            animate: true,
            lines: vec![
                DiffLine::context("fn alert_threshold() -> f32 {"),
                DiffLine::removed("    0.20 // 20% error rate"),
                DiffLine::added("    0.14 // tightened after telemetry review"),
                DiffLine::context("}"),
            ],
        }),
    );
    transcript.push(
        ChatRole::Assistant,
        ChatMessagePayload::Controls(vec![
            InlineControl::Button {
                id: "accept".to_string(),
                label: "Keep summary".to_string(),
                style: ControlStyle::Primary,
                disabled: false,
            },
            InlineControl::Button {
                id: "retry".to_string(),
                label: "Regenerate".to_string(),
                style: ControlStyle::Neutral,
                disabled: false,
            },
            InlineControl::Select {
                id: "detail".to_string(),
                label: Some("Detail".to_string()),
                options: vec![
                    SelectOption::new("brief", "Brief"),
                    SelectOption::new("normal", "Normal"),
                    SelectOption::new("verbose", "Verbose"),
                ],
                selected: Some("normal".to_string()),
            },
            InlineControl::Toggle {
                id: "cite_sources".to_string(),
                label: "Cite sources".to_string(),
                value: true,
            },
        ]),
    );

    transcript
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "genai")]
    use genai::chat::ChatRole as GenAiRole;
    use pretty_assertions::assert_eq;

    #[test]
    fn chat_controls_default_to_enabled() {
        let controls = ChatControls::default();

        assert!(controls.show_input);
        assert!(controls.show_send_button);
        assert!(controls.show_stop_button);
        assert!(controls.show_retry_button);
        assert!(controls.show_clear_button);
        assert!(controls.input_enabled);
        assert_eq!(controls.placeholder, "Ask anything, or invoke a tool...");
    }

    #[cfg(feature = "genai")]
    #[test]
    fn transcript_to_genai_request_maps_roles_and_tool_events() {
        let mut transcript = ChatTranscript::default();
        transcript.push(
            ChatRole::System,
            ChatMessagePayload::Text("Be concise".to_string()),
        );
        transcript.push(
            ChatRole::User,
            ChatMessagePayload::Text("Hello".to_string()),
        );
        transcript.push(
            ChatRole::Assistant,
            ChatMessagePayload::Text("Hi".to_string()),
        );
        transcript.push(
            ChatRole::Assistant,
            ChatMessagePayload::ToolCall(ToolCall {
                name: "lookup".to_string(),
                arguments: serde_json::json!({"q": "status"}),
                status: ToolCallStatus::Completed,
            }),
        );
        transcript.push(
            ChatRole::Tool,
            ChatMessagePayload::ToolResult {
                name: "lookup".to_string(),
                content: "All systems healthy".to_string(),
            },
        );
        transcript.push(
            ChatRole::Tool,
            ChatMessagePayload::Progress(ProgressState {
                label: "Unused in request".to_string(),
                percent: 50.0,
            }),
        );

        let request = transcript.to_genai_request();

        assert_eq!(request.system, Some("Be concise".to_string()));
        assert_eq!(request.messages.len(), 4);
        assert!(matches!(request.messages[0].role, GenAiRole::User));
        assert!(matches!(request.messages[1].role, GenAiRole::Assistant));
        assert!(matches!(request.messages[2].role, GenAiRole::Assistant));
        assert!(matches!(request.messages[3].role, GenAiRole::Tool));
    }

    #[test]
    fn sample_transcript_includes_tooling_and_progress() {
        let transcript = sample_transcript();

        assert!(
            transcript
                .messages
                .iter()
                .any(|m| matches!(m.payload, ChatMessagePayload::ToolCall(_)))
        );
        assert!(
            transcript
                .messages
                .iter()
                .any(|m| matches!(m.payload, ChatMessagePayload::Progress(_)))
        );
        assert!(
            transcript
                .messages
                .iter()
                .any(|m| matches!(m.payload, ChatMessagePayload::ToolResult { .. }))
        );
    }
}
