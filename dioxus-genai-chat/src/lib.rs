use dioxus::prelude::*;
use dioxus_bulma::prelude::*;
use genai::chat::{ChatMessage as GenAiChatMessage, ChatRequest};
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ChatMessagePayload {
    Text(String),
    Markdown(String),
    ToolCall(ToolCall),
    ToolResult { name: String, content: String },
    Progress(ProgressState),
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
                    messages.push(GenAiChatMessage::assistant(content.clone()));
                }
                (_, ChatMessagePayload::ToolResult { name, content }) => {
                    messages.push(GenAiChatMessage::assistant(format!(
                        "Tool result ({name}): {content}"
                    )));
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
                (_, ChatMessagePayload::Progress(_)) | (_, ChatMessagePayload::Typing) => {}
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
}

#[component]
pub fn ChatSurface(props: ChatSurfaceProps) -> Element {
    let title = props
        .title
        .clone()
        .unwrap_or_else(|| "Dioxus GenAI Chat".to_string());

    rsx! {
        BulmaProvider {
            theme: props.theme,
            load_bulma_css: true,
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
                            }
                        }

                        if props.controls.show_input {
                            Field {
                                Control {
                                    Textarea {
                                        value: String::new(),
                                        placeholder: props.controls.placeholder.clone(),
                                        rows: 3,
                                        disabled: !props.controls.input_enabled,
                                        readonly: true,
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
}

#[component]
fn ChatBubble(props: ChatBubbleProps) -> Element {
    let color = role_color(&props.message.role);

    rsx! {
        Message {
            color: color,
            class: "mb-4",
            MessageHeader {
                "{role_label(&props.message.role)}"
            }
            MessageBody {
                {match &props.message.payload {
                    ChatMessagePayload::Text(content) | ChatMessagePayload::Markdown(content) => rsx! {
                        p { "{content}" }
                    },
                    ChatMessagePayload::Typing => rsx! {
                        p { "Thinking…" }
                    },
                    ChatMessagePayload::Error(content) => rsx! {
                        p { class: "has-text-weight-semibold", "{content}" }
                    },
                    ChatMessagePayload::Progress(progress) => rsx! {
                        div {
                            p { class: "mb-2", "{progress.label}" }
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
                        rsx! {
                            div {
                                p { class: "mb-2", "Tool call requested." }
                                Tags {
                                    Tag {
                                        color: BulmaColor::Info,
                                        "{call.name}"
                                    }
                                    Tag {
                                        color: tool_status_color(&call.status),
                                        light: true,
                                        "{call.status.as_label()}"
                                    }
                                }
                                pre {
                                    style: "padding: 0.75rem; border-radius: 0.5rem; background: var(--bulma-scheme-main-bis, #f5f7fa);",
                                    "{args}"
                                }
                            }
                        }
                    }
                    ChatMessagePayload::ToolResult { name, content } => rsx! {
                        div {
                            Tags {
                                Tag {
                                    color: BulmaColor::Success,
                                    "Tool result: {name}"
                                }
                            }
                            pre {
                                style: "padding: 0.75rem; border-radius: 0.5rem; background: var(--bulma-scheme-main-bis, #f5f7fa);",
                                "{content}"
                            }
                        }
                    },
                }}
            }
        }
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

fn role_color(role: &ChatRole) -> BulmaColor {
    match role {
        ChatRole::System => BulmaColor::Dark,
        ChatRole::User => BulmaColor::Primary,
        ChatRole::Assistant => BulmaColor::Link,
        ChatRole::Tool => BulmaColor::Info,
    }
}

fn tool_status_color(status: &ToolCallStatus) -> BulmaColor {
    match status {
        ToolCallStatus::Pending => BulmaColor::Warning,
        ToolCallStatus::Running => BulmaColor::Info,
        ToolCallStatus::Completed => BulmaColor::Success,
        ToolCallStatus::Failed => BulmaColor::Danger,
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
        ChatRole::Assistant,
        ChatMessagePayload::Markdown(
            "### Summary\n- Error rate improved by **14%**\n- Latency remained stable".to_string(),
        ),
    );

    transcript
}

#[cfg(test)]
mod tests {
    use super::*;
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
        assert!(matches!(request.messages[3].role, GenAiRole::Assistant));
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
