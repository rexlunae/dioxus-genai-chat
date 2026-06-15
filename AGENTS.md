# AGENTS.md

Guidance for AI agents (and humans) working in this repository.

## What this is

A Rust workspace providing a configurable Dioxus + Bulma chat UI.

- `dioxus-genai-chat/` — the library crate. Data model + the `ChatSurface` component and friends.
- `demo/` — a small binary that renders `ChatSurface` with a sample transcript.

Edition 2024. Built/tested on a recent Rust nightly.

## Build, run, test

```bash
# Run the demo (desktop renderer; no dioxus-cli required)
cargo run -p demo

# Run all tests
cargo test --workspace

# Hot reload (requires dioxus-cli — see gotchas below)
dx serve --platform desktop
dx serve --platform web --no-default-features --features web
```

Always run `cargo test --workspace` before committing. When touching rendering,
also confirm both targets compile:

```bash
cargo check -p demo
cargo check -p demo --target wasm32-unknown-unknown --no-default-features --features web
```

To smoke-test the desktop app without a human, launch it under a timeout — a
clean run keeps running until killed (exit code 124):

```bash
timeout 6 ./target/debug/demo; echo "exit: $?"   # 124 == launched OK
```

## Gotchas (these have already bitten us)

- **`cargo install dioxus-cli` fails** on a fresh resolve: it picks an
  incompatible `git2`/`auth-git2` pair (`Cred::credential_helper` not found).
  Install with `cargo install dioxus-cli --locked`.
- **The web/wasm target cannot include `genai`.** `genai` pulls in `tokio`
  networking features that don't build on `wasm32-unknown-unknown`. It is an
  optional cargo feature (`dioxus-genai-chat/genai`, on by default). The web
  build must pass `--no-default-features --features web`; the demo's `desktop`
  feature re-enables `genai`.
- Because `to_genai_request` lives behind `#[cfg(feature = "genai")]`, it exists
  on desktop/native but not in web builds.

## Conventions

### Adding a `ChatMessagePayload` variant

The match arms are exhaustive (no catch-all), so a new variant requires updating
**both** sites or the build breaks:

1. `ChatTranscript::to_genai_request` (in `lib.rs`) — map it to a genai message,
   or add it to the ignore arm if it is ephemeral UI (progress / reasoning /
   status / controls / diff / typing are ignored — they are not sent to the model).
2. `ChatBubble` (in `lib.rs`) — render it.

### Controlled components (no internal state)

Interactive widgets do not own their *data* state — they render from props and
emit events; the consumer mutates the source data and passes it back. Keep it
that way; don't put transcript/attachment data in internal signals.

Purely *ephemeral view state* (a lightbox being open, a `<details>` collapse) is
fine to keep in a local signal — it is not conversation data. Example:
`DocumentGallery` holds the expanded-thumbnail index in a `use_signal`.

- `InlineControl` (Button/Select/Toggle) → `on_action: EventHandler<ControlEvent>`;
  consumer updates the transcript.
- Context attachments (`ContextItem`, files/directories) → `attachments` prop +
  `on_context: EventHandler<ContextEvent>`; consumer owns the pending list and
  does the actual file/dir picking (native dialog, browser input, typed path —
  the library deliberately has no file-dialog dependency). Enable via
  `ChatControls::allow_file_attachments` / `allow_directory_context`.

### Styling

All custom CSS lives in one scoped `<style>` block, `CHAT_SURFACE_CSS` in
`lib.rs`. Class names are prefixed `gc-`. Use Bulma CSS variables
(`var(--bulma-text)`, `var(--bulma-primary)`, …) with a hardcoded fallback so
the UI adapts to light/dark themes. Animations are pure CSS (`@keyframes
gc-spin`, `gc-pulse`) — no JS/state for spinners or collapse (use native
`<details>` for collapsible regions).

## Git / PR workflow

- `main` is the default branch and is protected by review — never commit
  directly to it. Branch first, then open a PR with `gh`.
- Only commit or push when the user explicitly asks.
- Merged PRs have their remote branch auto-deleted. Reusing a merged branch name
  recreates the branch but does NOT reopen the PR — start a fresh branch for new
  work.
