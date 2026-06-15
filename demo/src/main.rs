use dioxus::prelude::*;
use dioxus_genai_chat::{
    ChatControls, ChatSurface, ContextEvent, ContextItem, ControlEvent, ControlValue,
    sample_transcript,
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

    // Enable the file/directory affordances (off by default).
    let controls = ChatControls {
        allow_file_attachments: true,
        allow_directory_context: true,
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
