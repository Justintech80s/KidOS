#[cfg(target_os = "windows")]
use semver::Version;
#[cfg(target_os = "windows")]
use serde::Deserialize;
#[cfg(target_os = "windows")]
use sha2::{Digest, Sha256};
#[cfg(target_os = "windows")]
use std::{
    fs::{self, File},
    io::{Read, Write},
    path::{Path, PathBuf},
    process::Command,
    time::Duration,
};

#[cfg(target_os = "windows")]
const RELEASE_PREFIX: &str = "https://github.com/Justintech80s/KidOS/releases/download/";
#[cfg(target_os = "windows")]
const MAX_INSTALLER_BYTES: u64 = 2 * 1024 * 1024 * 1024;

#[cfg(target_os = "windows")]
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateManifest {
    pub schema_version: u32,
    pub product: String,
    pub version: String,
    pub platform: String,
    pub installer: UpdateInstaller,
}

#[cfg(target_os = "windows")]
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInstaller {
    pub file_name: String,
    pub url: String,
    pub sha256: String,
    pub size_bytes: u64,
    pub authenticode: AuthenticodeMetadata,
}

#[cfg(target_os = "windows")]
#[derive(Debug, Clone, Deserialize)]
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
fn validate_release_url(value: &str, expected_suffix: Option<&str>) -> Result<(), String> {
    if !value.starts_with(RELEASE_PREFIX) {
        return Err("KidOS updates must come from the official KidOS GitHub Releases path.".into());
    }
    let parsed = reqwest::Url::parse(value)
        .map_err(|_| "KidOS rejected an invalid update URL.".to_string())?;
    if parsed.scheme() != "https" || parsed.host_str() != Some("github.com") {
        return Err("KidOS update URLs must use HTTPS on github.com.".into());
    }
    if let Some(suffix) = expected_suffix {
        if !parsed.path().ends_with(suffix) {
            return Err("KidOS update URL does not match the expected release artifact.".into());
        }
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
    validate_release_url(&manifest.installer.url, Some(&manifest.installer.file_name))?;
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
    validate_release_url(manifest_url, Some("KidOS-update-manifest.json"))?;
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
fn staging_dir() -> Result<PathBuf, String> {
    let base = std::env::var_os("PROGRAMDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(r"C:\ProgramData"));
    let dir = base.join("KidOS").join("Updates").join("Staging");
    fs::create_dir_all(&dir)
        .map_err(|_| "KidOS could not create its protected update staging directory.".to_string())?;
    Ok(dir)
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
    if let Err(error) = verify_authenticode(&installer, &manifest.installer.authenticode.signer_thumbprint) {
        let _ = fs::remove_file(&installer);
        return Err(error);
    }
    Ok((manifest, installer))
}

#[cfg(target_os = "windows")]
pub fn launch_verified_update(installer: &Path) -> Result<(), String> {
    Command::new(installer)
        .arg("/S")
        .spawn()
        .map_err(|_| "KidOS could not launch the verified update installer.".to_string())?;
    Ok(())
}
