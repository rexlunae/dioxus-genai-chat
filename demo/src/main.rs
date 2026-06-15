use dioxus::prelude::*;
use dioxus_genai_chat::{ChatControls, ChatSurface, sample_transcript};

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        ChatSurface {
            transcript: sample_transcript(),
            controls: ChatControls::default(),
            title: "Dioxus GenAI Chat Demo".to_string(),
        }
    }
}
