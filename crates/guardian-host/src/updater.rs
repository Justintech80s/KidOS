#[cfg(target_os = "windows")]
use semver::Version;
#[cfg(target_os = "windows")]
use serde::{Deserialize, Serialize};
#[cfg(target_os = "windows")]
use sha2::{Digest, Sha256};
#[cfg(target_os = "windows")]
use std::{
    fs::{self, File},
    io::{Read, Write},
    path::{Path, PathBuf},
    process::Command,
    thread,
    time::Duration,
};

#[cfg(target_os = "windows")]
const RELEASE_PREFIX: &str = "https://github.com/Justintech80s/KidOS/releases/download/";
#[cfg(target_os = "windows")]
pub const DEFAULT_MANIFEST_URL: &str = "https://github.com/Justintech80s/KidOS/releases/latest/download/KidOS-update-manifest.json";
#[cfg(target_os = "windows")]
const MAX_INSTALLER_BYTES: u64 = 2 * 1024 * 1024 * 1024;
#[cfg(target_os = "windows")]
const UPDATE_CHECK_SECONDS: u64 = 6 * 60 * 60;

#[cfg(target_os = "windows")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateManifest {
    pub schema_version: u32,
    pub product: String,
    pub version: String,
    pub platform: String,
    pub installer: UpdateInstaller,
}

#[cfg(target_os = "windows")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInstaller {
    pub file_name: String,
    pub url: String,
    pub sha256: String,
    pub size_bytes: u64,
    pub authenticode: AuthenticodeMetadata,
}

#[cfg(target_os = "windows")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthenticodeMetadata {
    pub status: String,
    pub signer_subject: String,
    pub signer_thumbprint: String,
}

#[cfg(target_os = "windows")]
#[derive(Debug, Clone)]
pub struct UpdateAvailability {
    pub current_version: String,
    pub available_version: Option<String>,
    pub available: bool,
    pub installer_url: Option<String>,
    pub sha256: Option<String>,
    pub signer_thumbprint: Option<String>,
}

#[cfg(target_os = "windows")]
fn http_client() -> Result<reqwest::blocking::Client, String> {
    reqwest::blocking::Client::builder()
        .connect_timeout(Duration::from_secs(8))
        .timeout(Duration::from_secs(120))
        .user_agent("KidOS-Guardian-Updater/1")
        .build()
        .map_err(|_| "KidOS Guardian could not initialize its update client.".to_string())
}

#[cfg(target_os = "windows")]
fn validate_manifest_url(value: &str) -> Result<(), String> {
    if value != DEFAULT_MANIFEST_URL && !value.starts_with(RELEASE_PREFIX) {
        return Err("KidOS update manifests must come from the official KidOS GitHub Releases path.".into());
    }
    let parsed = reqwest::Url::parse(value)
        .map_err(|_| "KidOS rejected an invalid update manifest URL.".to_string())?;
    if parsed.scheme() != "https" || parsed.host_str() != Some("github.com") {
        return Err("KidOS update manifest URLs must use HTTPS on github.com.".into());
    }
    if !parsed.path().ends_with("/KidOS-update-manifest.json") {
        return Err("KidOS rejected an unexpected update manifest filename.".into());
    }
    Ok(())
}

#[cfg(target_os = "windows")]
fn validate_installer_url(value: &str, expected_name: &str) -> Result<(), String> {
    if !value.starts_with(RELEASE_PREFIX) {
        return Err("KidOS installers must come from the official versioned KidOS GitHub Releases path.".into());
    }
    let parsed = reqwest::Url::parse(value)
        .map_err(|_| "KidOS rejected an invalid update installer URL.".to_string())?;
    if parsed.scheme() != "https" || parsed.host_str() != Some("github.com") {
        return Err("KidOS update installers must use HTTPS on github.com.".into());
    }
    if !parsed.path().ends_with(&format!("/{expected_name}")) {
        return Err("KidOS update URL does not match the expected release artifact.".into());
    }
    Ok(())
}

#[cfg(target_os = "windows")]
fn normalize_thumbprint(value: &str) -> String {
    value
        .chars()
        .filter(|ch| ch.is_ascii_hexdigit())
        .collect::<String>()
        .to_ascii_uppercase()
}

#[cfg(target_os = "windows")]
fn validate_manifest(manifest: &UpdateManifest) -> Result<(), String> {
    if manifest.schema_version != 1 || manifest.product != "KidOS" || manifest.platform != "windows-x64" {
        return Err("KidOS rejected an incompatible update manifest.".into());
    }
    let _ = Version::parse(&manifest.version)
        .map_err(|_| "KidOS update manifest contains an invalid version.".to_string())?;
    if manifest.installer.file_name.is_empty()
        || manifest.installer.file_name.len() > 180
        || !manifest.installer.file_name.to_ascii_lowercase().ends_with(".exe")
        || manifest.installer.file_name.contains('/')
        || manifest.installer.file_name.contains('\\')
        || manifest.installer.file_name.contains("..")
    {
        return Err("KidOS update manifest contains an invalid installer filename.".into());
    }
    validate_installer_url(&manifest.installer.url, &manifest.installer.file_name)?;
    if manifest.installer.size_bytes == 0 || manifest.installer.size_bytes > MAX_INSTALLER_BYTES {
        return Err("KidOS update installer size is outside the allowed range.".into());
    }
    if manifest.installer.sha256.len() != 64
        || !manifest.installer.sha256.chars().all(|ch| ch.is_ascii_hexdigit())
    {
        return Err("KidOS update manifest contains an invalid SHA-256 digest.".into());
    }
    if !manifest.installer.authenticode.status.eq_ignore_ascii_case("valid") {
        return Err("KidOS update manifest does not describe a valid Authenticode signature.".into());
    }
    let thumbprint = normalize_thumbprint(&manifest.installer.authenticode.signer_thumbprint);
    if thumbprint.len() < 40 {
        return Err("KidOS update manifest contains an invalid signer thumbprint.".into());
    }
    if manifest.installer.authenticode.signer_subject.trim().is_empty() {
        return Err("KidOS update manifest is missing signer identity metadata.".into());
    }
    Ok(())
}

#[cfg(target_os = "windows")]
pub fn fetch_manifest(manifest_url: &str) -> Result<UpdateManifest, String> {
    validate_manifest_url(manifest_url)?;
    let response = http_client()?
        .get(manifest_url)
        .send()
        .map_err(|_| "KidOS could not reach the trusted update manifest.".to_string())?;
    if !response.status().is_success() {
        return Err(format!("KidOS update manifest returned HTTP {}.", response.status()));
    }
    if response.content_length().unwrap_or(0) > 256 * 1024 {
        return Err("KidOS rejected an oversized update manifest.".into());
    }
    let manifest: UpdateManifest = response
        .json()
        .map_err(|_| "KidOS received an invalid update manifest.".to_string())?;
    validate_manifest(&manifest)?;
    Ok(manifest)
}

#[cfg(target_os = "windows")]
pub fn check_update(manifest_url: &str) -> Result<UpdateAvailability, String> {
    let manifest = fetch_manifest(manifest_url)?;
    let current = Version::parse(env!("CARGO_PKG_VERSION"))
        .map_err(|_| "KidOS current version is invalid.".to_string())?;
    let candidate = Version::parse(&manifest.version)
        .map_err(|_| "KidOS update version is invalid.".to_string())?;
    let available = candidate > current;

    Ok(UpdateAvailability {
        current_version: current.to_string(),
        available_version: available.then(|| candidate.to_string()),
        available,
        installer_url: available.then(|| manifest.installer.url.clone()),
        sha256: available.then(|| manifest.installer.sha256.to_ascii_lowercase()),
        signer_thumbprint: available.then(|| normalize_thumbprint(&manifest.installer.authenticode.signer_thumbprint)),
    })
}

#[cfg(target_os = "windows")]
fn update_root() -> Result<PathBuf, String> {
    let base = std::env::var_os("PROGRAMDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(r"C:\ProgramData"));
    let dir = base.join("KidOS").join("Guardian").join("Updates");
    fs::create_dir_all(&dir)
        .map_err(|_| "KidOS could not create its protected update directory.".to_string())?;
    Ok(dir)
}

#[cfg(target_os = "windows")]
fn staging_dir() -> Result<PathBuf, String> {
    let dir = update_root()?.join("Staging");
    fs::create_dir_all(&dir)
        .map_err(|_| "KidOS could not create its protected update staging directory.".to_string())?;
    Ok(dir)
}

#[cfg(target_os = "windows")]
fn pending_manifest_path() -> Result<PathBuf, String> {
    Ok(update_root()?.join("pending-update.json"))
}

#[cfg(target_os = "windows")]
fn download_installer(manifest: &UpdateManifest) -> Result<PathBuf, String> {
    let destination = staging_dir()?.join(&manifest.installer.file_name);
    let partial = destination.with_extension("exe.partial");
    let _ = fs::remove_file(&partial);
    let _ = fs::remove_file(&destination);

    let mut response = http_client()?
        .get(&manifest.installer.url)
        .send()
        .map_err(|_| "KidOS could not download the signed update installer.".to_string())?;
    if !response.status().is_success() {
        return Err(format!("KidOS update installer returned HTTP {}.", response.status()));
    }
    if let Some(length) = response.content_length() {
        if length == 0 || length > MAX_INSTALLER_BYTES || length != manifest.installer.size_bytes {
            return Err("KidOS rejected an installer with an unexpected size.".into());
        }
    }

    let mut file = File::create(&partial)
        .map_err(|_| "KidOS could not create its temporary update file.".to_string())?;
    let mut hasher = Sha256::new();
    let mut total = 0u64;
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let count = response
            .read(&mut buffer)
            .map_err(|_| "KidOS update download was interrupted.".to_string())?;
        if count == 0 {
            break;
        }
        total = total.saturating_add(count as u64);
        if total > MAX_INSTALLER_BYTES || total > manifest.installer.size_bytes {
            let _ = fs::remove_file(&partial);
            return Err("KidOS stopped an oversized update download.".into());
        }
        hasher.update(&buffer[..count]);
        file.write_all(&buffer[..count])
            .map_err(|_| "KidOS could not write its update staging file.".to_string())?;
    }
    file.sync_all().map_err(|_| "KidOS could not finalize its update staging file.".to_string())?;

    if total != manifest.installer.size_bytes {
        let _ = fs::remove_file(&partial);
        return Err("KidOS update download size did not match the signed manifest.".into());
    }
    let digest = format!("{:x}", hasher.finalize());
    if !digest.eq_ignore_ascii_case(&manifest.installer.sha256) {
        let _ = fs::remove_file(&partial);
        return Err("KidOS blocked an update whose SHA-256 digest did not match the manifest.".into());
    }

    fs::rename(&partial, &destination)
        .map_err(|_| "KidOS could not finalize its staged update installer.".to_string())?;
    Ok(destination)
}

#[cfg(target_os = "windows")]
fn hash_file(path: &Path) -> Result<(String, u64), String> {
    let mut file = File::open(path)
        .map_err(|_| "KidOS could not open its staged update installer.".to_string())?;
    let mut hasher = Sha256::new();
    let mut total = 0u64;
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let count = file.read(&mut buffer)
            .map_err(|_| "KidOS could not verify its staged update installer.".to_string())?;
        if count == 0 { break; }
        total = total.saturating_add(count as u64);
        if total > MAX_INSTALLER_BYTES {
            return Err("KidOS blocked an oversized staged update installer.".into());
        }
        hasher.update(&buffer[..count]);
    }
    Ok((format!("{:x}", hasher.finalize()), total))
}

#[cfg(target_os = "windows")]
fn verify_authenticode(path: &Path, expected_thumbprint: &str) -> Result<(), String> {
    let script = "$s=Get-AuthenticodeSignature -LiteralPath $args[0]; if($s.Status -ne 'Valid' -or -not $s.SignerCertificate){exit 20}; [Console]::Out.Write($s.SignerCertificate.Thumbprint)";
    let output = Command::new("powershell.exe")
        .args(["-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-Command", script, "--"])
        .arg(path)
        .output()
        .map_err(|_| "KidOS could not invoke Windows signature verification.".to_string())?;
    if !output.status.success() {
        return Err("KidOS blocked an installer whose Windows Authenticode signature is not valid.".into());
    }
    let actual = normalize_thumbprint(&String::from_utf8_lossy(&output.stdout));
    let expected = normalize_thumbprint(expected_thumbprint);
    if actual.is_empty() || actual != expected {
        return Err("KidOS blocked an installer signed by an unexpected publisher certificate.".into());
    }
    Ok(())
}

#[cfg(target_os = "windows")]
fn verify_staged_installer(manifest: &UpdateManifest, installer: &Path) -> Result<(), String> {
    let canonical_root = fs::canonicalize(staging_dir()?)
        .map_err(|_| "KidOS update staging directory is unavailable.".to_string())?;
    let canonical_installer = fs::canonicalize(installer)
        .map_err(|_| "KidOS staged update installer is missing.".to_string())?;
    if !canonical_installer.starts_with(&canonical_root) || !canonical_installer.is_file() {
        return Err("KidOS rejected an update installer outside its protected staging directory.".into());
    }
    let (digest, size) = hash_file(&canonical_installer)?;
    if size != manifest.installer.size_bytes || !digest.eq_ignore_ascii_case(&manifest.installer.sha256) {
        return Err("KidOS blocked a staged update that no longer matches its trusted manifest.".into());
    }
    verify_authenticode(&canonical_installer, &manifest.installer.authenticode.signer_thumbprint)
}

#[cfg(target_os = "windows")]
pub fn stage_verified_update(manifest_url: &str) -> Result<(UpdateManifest, PathBuf), String> {
    let manifest = fetch_manifest(manifest_url)?;
    let current = Version::parse(env!("CARGO_PKG_VERSION"))
        .map_err(|_| "KidOS current version is invalid.".to_string())?;
    let candidate = Version::parse(&manifest.version)
        .map_err(|_| "KidOS update version is invalid.".to_string())?;
    if candidate <= current {
        return Err("KidOS refused an update that is not newer than the installed version.".into());
    }

    let installer = download_installer(&manifest)?;
    if let Err(error) = verify_staged_installer(&manifest, &installer) {
        let _ = fs::remove_file(&installer);
        return Err(error);
    }
    Ok((manifest, installer))
}

#[cfg(target_os = "windows")]
fn save_pending_update(manifest: &UpdateManifest) -> Result<(), String> {
    let bytes = serde_json::to_vec_pretty(manifest)
        .map_err(|_| "KidOS could not encode its pending update record.".to_string())?;
    let path = pending_manifest_path()?;
    let temporary = path.with_extension("json.tmp");
    fs::write(&temporary, bytes)
        .map_err(|_| "KidOS could not save its pending update record.".to_string())?;
    fs::rename(temporary, path)
        .map_err(|_| "KidOS could not finalize its pending update record.".to_string())
}

#[cfg(target_os = "windows")]
pub fn stage_latest_update() -> Result<Option<String>, String> {
    let availability = check_update(DEFAULT_MANIFEST_URL)?;
    if !availability.available {
        return Ok(None);
    }
    let (manifest, _installer) = stage_verified_update(DEFAULT_MANIFEST_URL)?;
    save_pending_update(&manifest)?;
    Ok(Some(manifest.version))
}

#[cfg(target_os = "windows")]
pub fn launch_pending_update_if_verified() -> Result<Option<String>, String> {
    let pending_path = pending_manifest_path()?;
    if !pending_path.exists() {
        return Ok(None);
    }
    let bytes = fs::read(&pending_path)
        .map_err(|_| "KidOS could not read its pending update record.".to_string())?;
    let manifest: UpdateManifest = serde_json::from_slice(&bytes)
        .map_err(|_| "KidOS rejected a corrupted pending update record.".to_string())?;
    validate_manifest(&manifest)?;

    let current = Version::parse(env!("CARGO_PKG_VERSION"))
        .map_err(|_| "KidOS current version is invalid.".to_string())?;
    let candidate = Version::parse(&manifest.version)
        .map_err(|_| "KidOS pending update version is invalid.".to_string())?;
    if candidate <= current {
        let _ = fs::remove_file(&pending_path);
        let _ = fs::remove_file(staging_dir()?.join(&manifest.installer.file_name));
        return Ok(None);
    }

    let installer = staging_dir()?.join(&manifest.installer.file_name);
    verify_staged_installer(&manifest, &installer)?;
    Command::new(&installer)
        .arg("/S")
        .spawn()
        .map_err(|_| "KidOS could not launch its verified pending update installer.".to_string())?;
    let _ = fs::remove_file(&pending_path);
    Ok(Some(manifest.version))
}

#[cfg(target_os = "windows")]
pub fn run_update_monitor() {
    thread::sleep(Duration::from_secs(60));
    loop {
        match stage_latest_update() {
            Ok(Some(version)) => {
                eprintln!("KidOS Guardian staged trusted update {version}; it will install after the next Guardian restart/reboot.");
            }
            Ok(None) => {}
            Err(error) => {
                eprintln!("KidOS Guardian update check failed safely: {error}");
            }
        }
        thread::sleep(Duration::from_secs(UPDATE_CHECK_SECONDS));
    }
}
