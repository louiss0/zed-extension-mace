use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};
use zed_extension_api::serde_json;
use zed_extension_api::{self as zed};

const RELEASE_OWNER: &str = "louiss0";
const RELEASE_REPOSITORY: &str = "mace";
const DEVELOPMENT_ENV_FILE: &str = ".env.development";
const VERSION_FILE: &str = "mace-version.json";
const DOWNLOAD_DIR: &str = ".mace-bin";
const BINARY_NAME: &str = if cfg!(windows) { "mace.exe" } else { "mace" };

struct MaceExtension;

#[derive(Debug, Deserialize)]
struct ReleaseMetadata {
    version: String,
}

impl ReleaseMetadata {
    fn download_url(&self) -> zed::Result<String> {
        let (os, arch, extension) = current_platform_release_parts()?;
        let version = self.version.trim_start_matches('v');

        Ok(format!(
            "https://github.com/{RELEASE_OWNER}/{RELEASE_REPOSITORY}/releases/download/v{version}/mace_v{version}_{os}_{arch}.{extension}"
        ))
    }
}

impl zed::Extension for MaceExtension {
    fn new() -> Self {
        Self
    }

    fn language_server_command(
        &mut self,
        language_server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> zed::Result<zed::Command> {
        if is_development_mode() {
            return go_language_server_command(worktree);
        }

        let metadata = read_release_metadata().ok();

        if let Some(binary_path) = downloaded_binary_path(metadata.as_ref()) {
            return Ok(zed::Command {
                command: binary_path.to_string_lossy().into_owned(),
                args: Vec::new(),
                env: worktree.shell_env(),
            });
        }

        if metadata.is_none() {
            return go_language_server_command(worktree);
        }

        let metadata = metadata.expect("metadata checked above");
        let binary_path = binary_path_for(&metadata.version);

        ensure_binary_ready(language_server_id, worktree, &metadata, &binary_path)?;

        Ok(zed::Command {
            command: binary_path.to_string_lossy().into_owned(),
            args: Vec::new(),
            env: worktree.shell_env(),
        })
    }
}

fn read_release_metadata() -> zed::Result<ReleaseMetadata> {
    let data = fs::read_to_string(VERSION_FILE)
        .map_err(|error| format!("failed to read {VERSION_FILE}: {error}"))?;

    serde_json::from_str(&data).map_err(|error| format!("failed to parse {VERSION_FILE}: {error}"))
}

fn downloaded_binary_path(metadata: Option<&ReleaseMetadata>) -> Option<PathBuf> {
    let Some(metadata) = metadata else {
        return None;
    };

    let binary_path = binary_path_for(&metadata.version);
    binary_path.exists().then_some(binary_path)
}

fn binary_path_for(version: &str) -> PathBuf {
    Path::new(DOWNLOAD_DIR).join(version).join(BINARY_NAME)
}

fn ensure_binary_ready(
    language_server_id: &zed::LanguageServerId,
    worktree: &zed::Worktree,
    metadata: &ReleaseMetadata,
    binary_path: &Path,
) -> zed::Result<()> {
    if binary_path.exists() {
        return Ok(());
    }

    if let Some(parent) = binary_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("failed to create {}: {error}", parent.display()))?;
    }

    zed::set_language_server_installation_status(
        language_server_id,
        &zed::LanguageServerInstallationStatus::Downloading,
    );

    let download_url = metadata.download_url()?;
    let file_type = archive_type_for(&download_url);

    let download_path = binary_path
        .to_str()
        .ok_or_else(|| format!("binary path is not valid UTF-8: {}", binary_path.display()))?;

    if let Err(error) = zed::download_file(&download_url, download_path, file_type) {
        zed::set_language_server_installation_status(
            language_server_id,
            &zed::LanguageServerInstallationStatus::Failed(error.clone()),
        );
        return Err(format!(
            "failed to download Mace {version}: {error}",
            version = metadata.version
        ));
    }

    if let Err(error) = zed::make_file_executable(download_path) {
        zed::set_language_server_installation_status(
            language_server_id,
            &zed::LanguageServerInstallationStatus::Failed(error.clone()),
        );
        return Err(format!(
            "failed to mark {} executable: {error}",
            binary_path.display()
        ));
    }

    let _ = worktree;
    zed::set_language_server_installation_status(
        language_server_id,
        &zed::LanguageServerInstallationStatus::None,
    );
    Ok(())
}

fn go_language_server_command(worktree: &zed::Worktree) -> zed::Result<zed::Command> {
    let go = worktree
        .which("go")
        .ok_or_else(|| "could not find `go` on PATH".to_string())?;

    Ok(zed::Command {
        command: go,
        args: vec!["run".to_string(), "./cmd".to_string(), "lsp".to_string()],
        env: worktree.shell_env(),
    })
}

fn archive_type_for(url: &str) -> zed::DownloadedFileType {
    if url.ends_with(".zip") {
        zed::DownloadedFileType::Zip
    } else if url.ends_with(".tar.gz") {
        zed::DownloadedFileType::GzipTar
    } else if url.ends_with(".gz") {
        zed::DownloadedFileType::Gzip
    } else {
        zed::DownloadedFileType::Uncompressed
    }
}

fn current_platform_release_parts() -> zed::Result<(&'static str, &'static str, &'static str)> {
    match zed::current_platform() {
        (zed::Os::Linux, zed::Architecture::X8664) => Ok(("linux", "amd64", "tar.gz")),
        (zed::Os::Linux, zed::Architecture::Aarch64) => Ok(("linux", "arm64", "tar.gz")),
        (zed::Os::Mac, zed::Architecture::X8664) => Ok(("darwin", "amd64", "tar.gz")),
        (zed::Os::Mac, zed::Architecture::Aarch64) => Ok(("darwin", "arm64", "tar.gz")),
        (zed::Os::Windows, zed::Architecture::X8664) => Ok(("windows", "amd64", "zip")),
        _ => Err("unsupported platform for Mace release metadata".to_string()),
    }
}

fn is_development_mode() -> bool {
    let Ok(data) = fs::read_to_string(DEVELOPMENT_ENV_FILE) else {
        return false;
    };

    data.lines().any(|line| {
        let Some((key, value)) = line.split_once('=') else {
            return false;
        };

        key.trim() == "MACE_EXTENSION_MODE" && value.trim() == "development"
    })
}

zed::register_extension!(MaceExtension);
