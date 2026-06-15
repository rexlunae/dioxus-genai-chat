use dioxus::prelude::*;
use dioxus_genai_chat::{
    ChatControls, ChatSurface, ContextEvent, ContextItem, ControlEvent, ControlValue,
    DocumentEvent, sample_transcript,
};

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let mut last_action = use_signal(|| "No control used yet.".to_string());
    // The caller owns the list of attached context — the chat surface is controlled.
    let mut attachments = use_signal(Vec::<ContextItem>::new);
    let mut next_id = use_signal(|| 0u32);

    // Enable the file/directory affordances and document selection (off by default).
    let controls = ChatControls {
        allow_file_attachments: true,
        allow_directory_context: true,
        allow_document_selection: true,
        ..ChatControls::default()
    };

    rsx! {
        ChatSurface {
            transcript: sample_transcript(),
            controls,
            title: "Dioxus GenAI Chat Demo".to_string(),
            attachments: attachments(),
            on_action: move |event: ControlEvent| {
                let summary = match event.value {
                    ControlValue::Clicked => format!("clicked `{}`", event.id),
                    ControlValue::Selected(value) => format!("`{}` set to `{value}`", event.id),
                    ControlValue::Toggled(on) => format!("`{}` toggled {}", event.id, if on { "on" } else { "off" }),
                };
                last_action.set(format!("Last action: {summary}"));
            },
            // Custom handler: render the expanded view of `DocumentKind::Custom`
            // documents however the app likes, using the document's `data` payload.
            render_document: move |doc: dioxus_genai_chat::Document| {
                let coords = doc
                    .data
                    .as_ref()
                    .and_then(|d| d.get("coords"))
                    .and_then(|c| c.as_str())
                    .unwrap_or("unknown");
                rsx! {
                    div {
                        style: "padding: 1rem; min-width: 18rem;",
                        p { style: "font-weight: 600; margin-bottom: 0.5rem;", "📍 Location handler" }
                        p { "Coordinates: {coords}" }
                        div {
                            style: "margin-top: 0.75rem; height: 8rem; border-radius: 0.5rem; background: linear-gradient(135deg, #6dd5ed, #2193b0); display: flex; align-items: center; justify-content: center; color: white;",
                            "[ custom map widget ]"
                        }
                    }
                }
            },
            // Selected documents added to context become attachments.
            on_document: move |event: DocumentEvent| match event {
                DocumentEvent::AddToContext(docs) => {
                    let n = docs.len();
                    for doc in &docs {
                        let item = ContextItem::from_document(doc);
                        if !attachments.read().iter().any(|a| a.id == item.id) {
                            attachments.write().push(item);
                        }
                    }
                    last_action.set(format!("Last action: added {n} document(s) to context"));
                }
            },
            on_context: move |event: ContextEvent| {
                // A real app would open a native file dialog (e.g. `rfd`) on desktop
                // or an <input type="file"> on web, then add the chosen paths. Here we
                // just simulate a selection so the controlled flow is visible.
                match event {
                    ContextEvent::AddFilesRequested => {
                        let id = next_id();
                        next_id.set(id + 1);
                        attachments.write().push(ContextItem::file(
                            format!("file-{id}"),
                            format!("example_{id}.rs"),
                        ));
                        last_action.set("Last action: attached a file".to_string());
                    }
                    ContextEvent::AddDirectoryRequested => {
                        let id = next_id();
                        next_id.set(id + 1);
                        attachments.write().push(ContextItem::directory(
                            format!("dir-{id}"),
                            "src/".to_string(),
                        ));
                        last_action.set("Last action: added a directory".to_string());
                    }
                    ContextEvent::Remove(id) => {
                        attachments.write().retain(|item| item.id != id);
                        last_action.set("Last action: removed context".to_string());
                    }
                }
            },
        }
        // Echo the most recent interaction so the demo is self-explanatory.
        p {
            style: "max-width: 40rem; margin: 0 auto 2rem; text-align: center; color: #7a7a7a; font-size: 0.85rem;",
            "{last_action}"
        }
    }
}
