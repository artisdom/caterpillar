use capi_runtime::Function;
use leptos::SignalGet;

fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug)
        .expect("Failed to initialize logging to console");

    let code = leptos::create_local_resource(|| (), fetch_code);
    let code = move || code.get();

    leptos::mount_to_body(move || {
        leptos::view! {
            <pre>{code}</pre>
        }
    });

    log::info!("Capi Debug initialized.");
}

async fn fetch_code((): ()) -> Vec<String> {
    let code = reqwest::get("http://127.0.0.1:8080/code")
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    let code: Vec<Function> = serde_json::from_str(&code).unwrap();

    let mut s = Vec::new();

    for (i, function) in code.into_iter().enumerate() {
        if i > 0 {
            s.push("\n".to_string());
        }

        s.push(format!("{function}"));
    }

    s
}
