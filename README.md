# dioxus-genai-chat

`dioxus-genai-chat` is a Rust workspace with:

- a reusable library crate that provides a configurable Dioxus + Bulma chat surface
- transcript/message models that support text, markdown, progress, tool calls, and tool results
- conversion helpers that turn transcript data into `genai::chat::ChatRequest`
- a small demo binary to exercise the transcript/configuration pipeline

## Workspace layout

- `dioxus-genai-chat/`: library crate
- `demo/`: demo binary crate

## Run tests

```bash
cargo test --workspace
```

## Run demo

The demo launches the full Bulma chat UI (`ChatSurface`) with a sample transcript.

### Desktop (no extra tooling)

```bash
cargo run -p demo
```

This builds the desktop renderer and opens a native window. No `dioxus-cli` required.

### Hot reload (dioxus-cli)

```bash
# Install the CLI with --locked to avoid a transitive dependency build failure
# (a plain `cargo install dioxus-cli` resolves an incompatible git2/auth-git2 pair)
cargo install dioxus-cli --locked

# Desktop, with hot reload
dx serve --platform desktop

# Web, in a browser at http://127.0.0.1:8080
dx serve --platform web --no-default-features --features web
```

> **About the web target:** `genai` depends on `tokio` networking features that
> do not build on `wasm32-unknown-unknown`, so it is an optional feature
> (`dioxus-genai-chat/genai`, on by default). The web build disables it via
> `--no-default-features --features web`; the chat UI renders the same, but the
> `ChatTranscript::to_genai_request` helper is only available with `genai`
> enabled (i.e. on desktop/native).
