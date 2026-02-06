use std::fs;
use std::path::Path;

#[cfg(windows)]
use serde::Deserialize;
#[cfg(windows)]
use std::collections::HashMap;

fn main() {
    println!("cargo:rerun-if-changed=versioninfo.json");
    println!("cargo:rerun-if-changed=icon.ico");

    #[cfg(windows)]
    if let Err(err) = build_windows_resources() {
        panic!("resource embedding failed: {err}");
    }
}

#[cfg(windows)]
fn build_windows_resources() -> Result<(), Box<dyn std::error::Error>> {
    let content = fs::read_to_string("versioninfo.json")?;
    let config: VersionConfig = serde_json::from_str(&content)?;

    let mut res = winres::WindowsResource::new();

    if let Some(icon_path) = config.icon_path.as_deref() {
        if Path::new(icon_path).exists() {
            res.set_icon(icon_path);
        } else {
            println!("cargo:warning=icon file '{icon_path}' not found");
        }
    }

    if let Some(manifest_path) = config.manifest_path.as_deref() {
        if !manifest_path.is_empty() {
            res.set_manifest_file(manifest_path);
        }
    }

    if let Some(fixed) = config.fixed_file_info.as_ref() {
        if let Some(parts) = fixed.file_version.as_ref() {
            res.set_version_info(winres::VersionInfo::FILEVERSION, pack_version(parts));
        }
        if let Some(parts) = fixed.product_version.as_ref() {
            res.set_version_info(winres::VersionInfo::PRODUCTVERSION, pack_version(parts));
        }
        if let Some(value) = parse_hex_value(fixed.file_flags_mask.as_deref()) {
            res.set_version_info(winres::VersionInfo::FILEFLAGSMASK, value);
        }
        if let Some(value) = parse_hex_value(fixed.file_flags.as_deref()) {
            res.set_version_info(winres::VersionInfo::FILEFLAGS, value);
        }
        if let Some(value) = parse_hex_value(fixed.file_os.as_deref()) {
            res.set_version_info(winres::VersionInfo::FILEOS, value);
        }
        if let Some(value) = parse_hex_value(fixed.file_type.as_deref()) {
            res.set_version_info(winres::VersionInfo::FILETYPE, value);
        }
        if let Some(value) = parse_hex_value(fixed.file_sub_type.as_deref()) {
            res.set_version_info(winres::VersionInfo::FILESUBTYPE, value);
        }
    }

    if let Some(strings) = config.string_file_info.as_ref() {
        for (key, value) in strings {
            res.set(key.as_str(), value.as_str());
        }
    }

    if let Some(lang) = config
        .var_file_info
        .as_ref()
        .and_then(|var| var.translation.as_ref())
        .and_then(|translation| parse_hex_value(translation.lang_id.as_deref()))
        .and_then(|value| u16::try_from(value).ok())
    {
        res.set_language(lang);
    }

    res.compile()?;
    Ok(())
}

#[cfg(windows)]
fn pack_version(parts: &VersionParts) -> u64 {
    ((parts.major as u64) << 48)
        | ((parts.minor as u64) << 32)
        | ((parts.patch as u64) << 16)
        | parts.build as u64
}

#[cfg(windows)]
fn parse_hex_value(value: Option<&str>) -> Option<u64> {
    let raw = value?.trim();
    if raw.is_empty() {
        return None;
    }
    let trimmed = raw.trim_start_matches("0x");
    u64::from_str_radix(trimmed, 16).ok()
}

#[cfg(windows)]
#[derive(Deserialize)]
struct VersionConfig {
    #[serde(rename = "FixedFileInfo")]
    fixed_file_info: Option<FixedFileInfo>,
    #[serde(rename = "StringFileInfo")]
    string_file_info: Option<HashMap<String, String>>,
    #[serde(rename = "VarFileInfo")]
    var_file_info: Option<VarFileInfo>,
    #[serde(rename = "IconPath")]
    icon_path: Option<String>,
    #[serde(rename = "ManifestPath")]
    manifest_path: Option<String>,
}

#[cfg(windows)]
#[derive(Deserialize)]
struct FixedFileInfo {
    #[serde(rename = "FileVersion")]
    file_version: Option<VersionParts>,
    #[serde(rename = "ProductVersion")]
    product_version: Option<VersionParts>,
    #[serde(rename = "FileFlagsMask")]
    file_flags_mask: Option<String>,
    #[serde(rename = "FileFlags")]
    file_flags: Option<String>,
    #[serde(rename = "FileOS")]
    file_os: Option<String>,
    #[serde(rename = "FileType")]
    file_type: Option<String>,
    #[serde(rename = "FileSubType")]
    file_sub_type: Option<String>,
}

#[cfg(windows)]
#[derive(Deserialize)]
struct VersionParts {
    #[serde(rename = "Major")]
    major: u16,
    #[serde(rename = "Minor")]
    minor: u16,
    #[serde(rename = "Patch")]
    patch: u16,
    #[serde(rename = "Build")]
    build: u16,
}

#[cfg(windows)]
#[derive(Deserialize)]
struct VarFileInfo {
    #[serde(rename = "Translation")]
    translation: Option<Translation>,
}

#[cfg(windows)]
#[derive(Deserialize)]
struct Translation {
    #[serde(rename = "LangID")]
    lang_id: Option<String>,
}
