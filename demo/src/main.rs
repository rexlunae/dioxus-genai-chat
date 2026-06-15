use dioxus::prelude::*;
use dioxus_genai_chat::{ChatControls, ChatSurface, ControlValue, sample_transcript};

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let mut last_action = use_signal(|| "No control used yet.".to_string());

    rsx! {
        ChatSurface {
            transcript: sample_transcript(),
            controls: ChatControls::default(),
            title: "Dioxus GenAI Chat Demo".to_string(),
            on_action: move |event: dioxus_genai_chat::ControlEvent| {
                let summary = match event.value {
                    ControlValue::Clicked => format!("clicked `{}`", event.id),
                    ControlValue::Selected(value) => format!("`{}` set to `{value}`", event.id),
                    ControlValue::Toggled(on) => format!("`{}` toggled {}", event.id, if on { "on" } else { "off" }),
                };
                last_action.set(format!("Last action: {summary}"));
            },
        }
        // Echo the most recent inline-control interaction so the demo is self-explanatory.
        p {
            style: "max-width: 40rem; margin: 0 auto 2rem; text-align: center; color: #7a7a7a; font-size: 0.85rem;",
            "{last_action}"
        }
    }
}
