pub mod commands;

use commands::{evaluate_download, evaluate_navigation, get_guardian_status, plan_workspace};
use guardian_service::{GuardianActor, GuardianPolicyStore, ParentPolicyConfig};
use secure_store::{ParentAuthorization, ParentAuthorizationResult, SecretStore};
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

pub fn save_parent_policy_with_authorization<S: SecretStore>(
    authorization: &mut ParentAuthorization<S>,
    guardian: &mut GuardianPolicyStore,
    pin: &str,
    now_seconds: u64,
    policy: ParentPolicyConfig,
) -> Result<(), String> {
    match authorization
        .verify(pin, now_seconds)
        .map_err(|_| "unable to verify parent PIN".to_string())?
    {
        ParentAuthorizationResult::Authorized => guardian
            .replace_parent_policy(GuardianActor::ParentAuthorized, policy)
            .map_err(|error| error.to_string()),
        ParentAuthorizationResult::Denied => Err("parent PIN was not accepted".into()),
        ParentAuthorizationResult::Locked => Err("parent PIN entry is temporarily locked".into()),
    }
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
        .invoke_handler(tauri::generate_handler![
            configure_parent_pin,
            plan_workspace,
            evaluate_navigation,
            evaluate_download,
            get_guardian_status
        ])
        .run(tauri::generate_context!())
        .expect("error while running KidOS");
}
