# dioxus-genai-chat

`dioxus-genai-chat` is a Rust workspace with:

- a reusable library crate that provides a configurable Dioxus + Bulma chat surface
- transcript/message models that support text, markdown, progress, tool calls, and tool results
- conversion helpers that turn transcript data into `genai::chat::ChatRequest`
- a small demo binary to exercise the transcript/configuration pipeline

## Workspace layout

- `/home/runner/work/dioxus-genai-chat/dioxus-genai-chat/rexlunae/dioxus-genai-chat/dioxus-genai-chat`: library crate
- `/home/runner/work/dioxus-genai-chat/dioxus-genai-chat/rexlunae/dioxus-genai-chat/demo`: demo binary crate

## Run tests

```bash
cargo test --workspace
```

## Run demo

```bash
cargo run -p demo
```
