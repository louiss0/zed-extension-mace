use zed_extension_api::{self as zed, Architecture, Os};

struct MaceExtension;

impl MaceExtension {
    const SERVER_BINARY_NAME: &'static str = "mace";

    fn bundled_binary_path(&self) -> zed::Result<String> {
        let (os, architecture) = zed::current_platform();

        let path = match (os, architecture) {
            (Os::Mac, Architecture::Aarch64) => "bin/mace-darwin-arm64",
            (Os::Mac, Architecture::X8664) => "bin/mace-darwin-amd64",
            (Os::Linux, Architecture::Aarch64) => "bin/mace-linux-arm64",
            (Os::Linux, Architecture::X8664) => "bin/mace-linux-amd64",
            (Os::Windows, Architecture::Aarch64) => "bin/mace-windows-arm64.exe",
            (Os::Windows, Architecture::X8664) => "bin/mace-windows-amd64.exe",
            _ => return Err("current platform is not supported by the bundled Mace binary".to_string()),
        };

        Ok(path.to_string())
    }
}

impl zed::Extension for MaceExtension {
    fn new() -> Self {
        Self
    }

    fn language_server_command(
        &mut self,
        _language_server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> zed::Result<zed::Command> {
        let command = self
            .bundled_binary_path()
            .or_else(|_| {
                worktree
                    .which(Self::SERVER_BINARY_NAME)
                    .ok_or_else(|| "could not find a bundled `mace` binary or `mace` on PATH".to_string())
            })?;

        Ok(zed::Command {
            command,
            args: vec!["lsp".to_string()],
            env: worktree.shell_env(),
        })
    }
}

zed::register_extension!(MaceExtension);
