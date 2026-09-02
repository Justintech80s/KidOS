use secure_store::SecretStore;
#[cfg(target_os = "windows")]
use secure_store::WindowsSecretStore;

const PARENT_PIN_KEY: &str = "parent-pin";

pub fn configure_parent_pin_with_store(
    store: &dyn SecretStore,
    pin: &str,
) -> Result<(), String> {
    if !(4..=8).contains(&pin.len()) || !pin.chars().all(|character| character.is_ascii_digit()) {
        return Err("invalid parent PIN".into());
    }

    store
        .put_secret(PARENT_PIN_KEY, pin)
        .map_err(|_| "unable to protect parent PIN".into())
}

#[cfg(target_os = "windows")]
#[tauri::command]
fn configure_parent_pin(pin: String) -> Result<(), String> {
    let store = WindowsSecretStore::new("KidOS");
    configure_parent_pin_with_store(&store, &pin)
}

#[cfg(not(target_os = "windows"))]
#[tauri::command]
fn configure_parent_pin(_pin: String) -> Result<(), String> {
    Err("parent PIN storage is available in the Windows KidOS build".into())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![configure_parent_pin])
        .run(tauri::generate_context!())
        .expect("error while running KidOS");
}
