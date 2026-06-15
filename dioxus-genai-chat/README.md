# dioxus-genai-chat

[![crates.io](https://img.shields.io/crates/v/dioxus-genai-chat.svg)](https://crates.io/crates/dioxus-genai-chat)
[![docs.rs](https://docs.rs/dioxus-genai-chat/badge.svg)](https://docs.rs/dioxus-genai-chat)

A configurable [Dioxus](https://dioxuslabs.com/) + [Bulma](https://bulma.io/)
chat UI: a `ChatSurface` component plus a small transcript data model.

## Features

- **Chained reasoning timelines** — collapsible, VS Code-style "thinking" steps
  with status markers and a connecting line.
- **Inline controls** — buttons, selectors, and toggles rendered inside messages,
  surfaced through a single `on_action` event handler (controlled-component style).
- **Spinning status indicators** — pure-CSS spinners for in-progress work; tool
  calls, progress bars, and error states.
- **`genai` integration** — convert a transcript into a `genai` chat request
  (optional, on by default).
- **Theme-aware styling** — scoped CSS driven by Bulma CSS variables.

## Install

```toml
[dependencies]
dioxus-genai-chat = "0.1"
```

For a `wasm32-unknown-unknown` (web) build, disable default features — `genai`
pulls in `tokio` networking that does not compile on wasm:

```toml
[dependencies]
dioxus-genai-chat = { version = "0.1", default-features = false }
```

## Usage

```rust
use dioxus::prelude::*;
use dioxus_genai_chat::{ChatSurface, ControlValue, sample_transcript};

#[component]
fn App() -> Element {
    rsx! {
        ChatSurface {
            transcript: sample_transcript(),
            title: "My Assistant".to_string(),
            on_action: move |event: dioxus_genai_chat::ControlEvent| {
                match event.value {
                    ControlValue::Clicked => { /* button `event.id` */ }
                    ControlValue::Selected(value) => { /* select -> value */ }
                    ControlValue::Toggled(on) => { /* toggle -> on */ }
                }
            },
        }
    }
}
```

Inline controls are **controlled**: handle `on_action` and update your
transcript in response.

With the default `genai` feature, `ChatTranscript::to_genai_request()` turns a
transcript into a `genai::chat::ChatRequest`.

## License

Licensed under the [MIT license](LICENSE).
