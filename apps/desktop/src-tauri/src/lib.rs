mod dto;
mod profiles;
mod service;

use dto::DeviceDto;
use profiles::ProfileDto;
use button_engine::Action;
use service::{DeviceService, EngineStatusDto};
use tauri::State;

#[tauri::command]
async fn list_devices(svc: State<'_, DeviceService>) -> Result<Vec<DeviceDto>, String> {
    let started = std::time::Instant::now();
    let result = svc.list();
    match &result {
        Ok(devices) => tracing::info!(
            count = devices.len(),
            ms = started.elapsed().as_millis(),
            names = ?devices.iter().map(|d| &d.name).collect::<Vec<_>>(),
            "list_devices"
        ),
        Err(e) => tracing::error!("list_devices failed: {e}"),
    }
    result
}

/// Registered because the frontend calls it on startup. Returning defaults
/// keeps the device list usable until SQLite-backed profiles land.
#[tauri::command]
async fn list_profiles() -> Result<Vec<ProfileDto>, String> {
    Ok(profiles::defaults())
}

#[tauri::command]
async fn set_dpi(
    svc: State<'_, DeviceService>,
    device_id: String,
    dpi: u16,
) -> Result<u16, String> {
    // The frontend uses the lowercased unit id as the device id.
    let result = svc.set_dpi(device_id.to_uppercase(), dpi);
    tracing::info!(?result, dpi, "set_dpi");
    result
}

#[tauri::command]
async fn set_diverted(
    svc: State<'_, DeviceService>,
    device_id: String,
    cid: u16,
    diverted: bool,
) -> Result<(), String> {
    svc.set_diverted(device_id.to_uppercase(), cid, diverted)
}

/// Assign an action to a control. The engine and the device's diversion flag
/// are updated together — see `Cmd::SetMapping`.
#[tauri::command]
async fn set_mapping(
    svc: State<'_, DeviceService>,
    device_id: String,
    cid: u16,
    action: Action,
) -> Result<(), String> {
    let result = svc.set_mapping(device_id.to_uppercase(), cid, action.clone());
    tracing::info!(cid = format!("0x{cid:04x}"), ?action, ?result, "set_mapping");
    result
}

/// Whether input synthesis is working. The UI uses this to explain that
/// remapping needs a re-login, rather than letting buttons silently do nothing.
#[tauri::command]
async fn engine_status(svc: State<'_, DeviceService>) -> Result<EngineStatusDto, String> {
    svc.engine_status()
}

/// Report a frontend error into the Rust log.
///
/// A webview that hangs or throws leaves nothing in the terminal, so a UI
/// fault is invisible to anyone reading logs — which is exactly the situation
/// where you most need one. The frontend installs global `error` and
/// `unhandledrejection` handlers that call this.
#[tauri::command]
async fn log_frontend(level: String, message: String, detail: Option<String>) {
    match level.as_str() {
        "error" => tracing::error!(target: "frontend", detail = ?detail, "{message}"),
        "warn" => tracing::warn!(target: "frontend", detail = ?detail, "{message}"),
        _ => tracing::info!(target: "frontend", detail = ?detail, "{message}"),
    }
}

/// Restore a device to factory behaviour.
#[tauri::command]
async fn reset_device(svc: State<'_, DeviceService>, device_id: String) -> Result<(), String> {
    let result = svc.reset_device(device_id.to_uppercase());
    tracing::info!(?result, "reset_device");
    result
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                // `frontend` is its own target so webview diagnostics are not
                // filtered out by a crate-name filter.
                .unwrap_or_else(|_| "openlogi_lib=info,frontend=info,hidpp=warn".into()),
        )
        .init();

    tauri::Builder::default()
        .setup(|app| {
            // Debug builds get devtools so a webview fault can be inspected
            // rather than guessed at.
            #[cfg(debug_assertions)]
            {
                use tauri::Manager;
                if let Some(window) = app.get_webview_window("main") {
                    window.open_devtools();
                }
            }
            let _ = app;
            Ok(())
        })
        .manage(DeviceService::spawn())
        .invoke_handler(tauri::generate_handler![
            list_devices,
            list_profiles,
            set_dpi,
            set_diverted,
            set_mapping,
            engine_status,
            reset_device,
            log_frontend
        ])
        .run(tauri::generate_context!())
        .expect("error while running OpenLogi");
}
