#[tauri::command]
fn runtime_label() -> String {
    format!(
        "Lyreniq {} · {}",
        env!("CARGO_PKG_VERSION"),
        std::env::consts::OS
    )
}

fn register_commands<R: tauri::Runtime>(builder: tauri::Builder<R>) -> tauri::Builder<R> {
    builder.invoke_handler(tauri::generate_handler![runtime_label])
}

pub fn run() {
    register_commands(tauri::Builder::default())
        .run(tauri::generate_context!())
        .expect("failed to run Lyreniq desktop application");
}

#[cfg(test)]
mod tests {
    use super::register_commands;
    use tauri::test::{get_ipc_response, mock_builder, mock_context, noop_assets, INVOKE_KEY};

    fn invoke(command: &str) -> Result<String, String> {
        let app = register_commands(mock_builder())
            .build(mock_context(noop_assets()))
            .expect("mock app must build");
        let window = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
            .build()
            .expect("mock window must build");
        get_ipc_response(
            &window,
            tauri::webview::InvokeRequest {
                cmd: command.into(),
                callback: tauri::ipc::CallbackFn(0),
                error: tauri::ipc::CallbackFn(1),
                url: "http://tauri.localhost".parse().unwrap(),
                body: Default::default(),
                headers: Default::default(),
                invoke_key: INVOKE_KEY.to_string(),
            },
        )
        .map(|response| {
            response
                .deserialize::<String>()
                .expect("IPC result must be a string")
        })
        .map_err(|error| format!("{error:?}"))
    }

    #[test]
    fn registered_runtime_command_is_callable_through_ipc() {
        let label = invoke("runtime_label").expect("registered IPC command must succeed");
        assert!(label.starts_with("Lyreniq "));
        assert!(label.contains(env!("CARGO_PKG_VERSION")));
        assert!(label.ends_with(std::env::consts::OS));
    }

    #[test]
    fn unregistered_commands_are_rejected() {
        assert!(invoke("unregistered_command").is_err());
    }
}
