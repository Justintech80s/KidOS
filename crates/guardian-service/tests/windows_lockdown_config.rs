use guardian_service::windows_lockdown::{build_assigned_access_config, AccountRole, ApprovedApp, LockdownConfigError, LockdownProfile};

fn profile(role: AccountRole, apps: Vec<ApprovedApp>) -> LockdownProfile {
    LockdownProfile {
        profile_id: "{9A2A490F-10F6-4764-974A-43B19E722C23}".into(),
        account: "KidOSChild".into(),
        account_role: role,
        apps,
    }
}

fn kidos() -> ApprovedApp {
    ApprovedApp { id: "kidos".into(), display_name: "KidOS".into(), executable_path: r"C:\Program Files\KidOS\KidOS.exe".into() }
}

#[test]
fn standard_child_profile_is_generated_deterministically() {
    let p = profile(AccountRole::Standard, vec![kidos()]);
    let a = build_assigned_access_config(&p).unwrap();
    let b = build_assigned_access_config(&p).unwrap();
    assert_eq!(a, b);
    assert!(a.contains("<Account>KidOSChild</Account>"));
    assert!(a.contains("DesktopAppPath=\"C:\\Program Files\\KidOS\\KidOS.exe\""));
}

#[test]
fn administrator_and_unknown_accounts_are_rejected() {
    for role in [AccountRole::Administrator, AccountRole::Unknown] {
        assert_eq!(build_assigned_access_config(&profile(role, vec![kidos()])), Err(LockdownConfigError::ManagedAccountMustBeStandard));
    }
}

#[test]
fn kidos_is_mandatory() {
    let app = ApprovedApp { id: "paint".into(), display_name: "Paint".into(), executable_path: r"C:\Windows\System32\mspaint.exe".into() };
    assert_eq!(build_assigned_access_config(&profile(AccountRole::Standard, vec![app])), Err(LockdownConfigError::KidOSRequired));
}

#[test]
fn xml_sensitive_values_are_escaped() {
    let mut p = profile(AccountRole::Standard, vec![kidos()]);
    p.account = "Kid&<Child>".into();
    let xml = build_assigned_access_config(&p).unwrap();
    assert!(xml.contains("Kid&amp;&lt;Child&gt;"));
}

#[test]
fn administrative_executables_are_rejected() {
    for executable in ["cmd.exe", "powershell.exe", "pwsh.exe", "regedit.exe", "wt.exe", "wscript.exe", "cscript.exe", "mmc.exe"] {
        let app = ApprovedApp { id: "bad".into(), display_name: "Bad".into(), executable_path: format!(r"C:\Windows\System32\{executable}") };
        assert_eq!(build_assigned_access_config(&profile(AccountRole::Standard, vec![kidos(), app])), Err(LockdownConfigError::AdministrativeExecutable(executable.into())));
    }
}
