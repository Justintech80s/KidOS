#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccountRole { Standard, Administrator, Unknown }

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApprovedApp {
    pub id: String,
    pub display_name: String,
    pub executable_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LockdownProfile {
    pub profile_id: String,
    pub account: String,
    pub account_role: AccountRole,
    pub apps: Vec<ApprovedApp>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LockdownConfigError {
    ManagedAccountMustBeStandard,
    KidOSRequired,
    AdministrativeExecutable(String),
}

fn escape_xml(value: &str) -> String {
    value.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;").replace('\'', "&apos;")
}

fn executable_name(path: &str) -> String {
    path.replace('/', "\\").rsplit('\\').next().unwrap_or("").to_ascii_lowercase()
}

pub fn build_assigned_access_config(profile: &LockdownProfile) -> Result<String, LockdownConfigError> {
    if profile.account_role != AccountRole::Standard {
        return Err(LockdownConfigError::ManagedAccountMustBeStandard);
    }
    if !profile.apps.iter().any(|app| app.id.eq_ignore_ascii_case("kidos")) {
        return Err(LockdownConfigError::KidOSRequired);
    }

    const ADMIN_TOOLS: &[&str] = &["cmd.exe", "powershell.exe", "pwsh.exe", "regedit.exe", "wt.exe", "wscript.exe", "cscript.exe", "mmc.exe"];
    for app in &profile.apps {
        let name = executable_name(&app.executable_path);
        if ADMIN_TOOLS.contains(&name.as_str()) {
            return Err(LockdownConfigError::AdministrativeExecutable(name));
        }
    }

    let apps = profile.apps.iter().map(|app| {
        format!("<App DesktopAppPath=\"{}\" />", escape_xml(&app.executable_path))
    }).collect::<Vec<_>>().join("");

    Ok(format!(
        "<?xml version=\"1.0\" encoding=\"utf-8\"?><AssignedAccessConfiguration xmlns=\"http://schemas.microsoft.com/AssignedAccess/2017/config\"><Profiles><Profile Id=\"{}\"><AllAppsList><AllowedApps>{}</AllowedApps></AllAppsList><Taskbar ShowTaskbar=\"false\" /></Profile></Profiles><Configs><Config><Account>{}</Account><DefaultProfile Id=\"{}\" /></Config></Configs></AssignedAccessConfiguration>",
        escape_xml(&profile.profile_id), apps, escape_xml(&profile.account), escape_xml(&profile.profile_id)
    ))
}
