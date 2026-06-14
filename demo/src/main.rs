use dioxus_genai_chat::{ChatControls, sample_transcript};

fn main() {
    let transcript = sample_transcript();
    let request = transcript.to_genai_request();

    println!("dioxus-genai-chat demo");
    println!("----------------------");
    println!("messages in transcript: {}", transcript.messages.len());
    println!("messages sent to genai: {}", request.messages.len());
    println!(
        "system prompt: {}",
        request.system.unwrap_or_else(|| "<none>".to_string())
    );

    let controls = ChatControls::default();
    println!("send control enabled: {}", controls.show_send_button);
    println!("retry control enabled: {}", controls.show_retry_button);
    println!("placeholder: {}", controls.placeholder);

    println!("\nUse ChatSurface in a Dioxus app to render the full Bulma UI.");
}
