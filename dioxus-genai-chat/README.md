# dioxus-genai-chat

[![crates.io](https://img.shields.io/crates/v/dioxus-genai-chat.svg)](https://crates.io/crates/dioxus-genai-chat)
[![docs.rs](https://docs.rs/dioxus-genai-chat/badge.svg)](https://docs.rs/dioxus-genai-chat)

A configurable [Dioxus](https://dioxuslabs.com/) + [Bulma](https://bulma.io/)
chat UI: a `ChatSurface` component plus a small transcript data model.

## Features

- **Chained reasoning timelines** — collapsible, VS Code-style "thinking" steps
  with status markers and a connecting line.
- **Controlled composer** — bind `input` and handle `on_send` / `on_stop` /
  `on_retry` / `on_clear`; Enter sends, Shift+Enter inserts a newline.
- **Inline controls** — buttons, selectors, and toggles rendered inside messages,
  surfaced through a single `on_action` event handler (controlled-component style).
- **Markdown rendering** — `Markdown` payloads render to formatted HTML
  (`markdown` feature, on by default).
- **Embeddable** — `embedded: true` drops the standalone page chrome (no
  self-loaded Bulma, no `Section`/`Box`/title) so the surface fits inside a host
  app that already provides Bulma and a theme.
- **Composer accessory slot** — inject app-specific controls (a model picker, a
  directory selector, …) into the input area via `input_accessory`.
- **Spinning status indicators** — pure-CSS spinners for in-progress work; tool
  calls, progress bars, and error states.
- **`genai` integration** — convert a transcript into a `genai` chat request
  (optional, on by default).
- **Theme-aware styling** — scoped CSS driven by Bulma CSS variables.

## Install

```toml
[dependencies]
dioxus-genai-chat = "0.2"
```

For a `wasm32-unknown-unknown` (web) build, disable default features — `genai`
pulls in `tokio` networking that does not compile on wasm (the `markdown`
feature is pure Rust and safe to keep):

```toml
[dependencies]
dioxus-genai-chat = { version = "0.2", default-features = false, features = ["markdown"] }
```

## Usage

```rust
use dioxus::prelude::*;
use dioxus_genai_chat::{ChatMessagePayload, ChatRole, ChatSurface, ChatTranscript};

#[component]
fn App() -> Element {
    let mut transcript = use_signal(ChatTranscript::default);
    let mut input = use_signal(String::new);

    rsx! {
        ChatSurface {
            // `embedded: true` when hosting inside an app that already loads Bulma.
            transcript: transcript(),
            title: "My Assistant".to_string(),
            input: input(),
            on_input: move |v: String| input.set(v),
            on_send: move |text: String| {
                transcript.write().push(ChatRole::User, ChatMessagePayload::Markdown(text));
                input.set(String::new());
                // ...kick off your backend / model call here.
            },
        }
    }
}
```

The composer, transcript, attachments, and inline controls are all
**controlled**: the surface renders what you pass and emits events; you own the
state and update it in response.

With the default `genai` feature, `ChatTranscript::to_genai_request()` turns a
transcript into a `genai::chat::ChatRequest`.

## License

Licensed under the [MIT license](LICENSE).
