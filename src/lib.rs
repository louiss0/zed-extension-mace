use zed_extension_api as zed;

struct MaceExtension;

impl zed::Extension for MaceExtension {
    fn new() -> Self {
        Self
    }

    fn language_server_command(
        &mut self,
        _language_server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> zed::Result<zed::Command> {
        let command = worktree
            .which("go")
            .ok_or_else(|| "could not find `go` on PATH".to_string())?;

        let package_path = format!("{}/cmd", worktree.root_path());

        Ok(zed::Command {
            command,
            args: vec!["run".to_string(), package_path, "lsp".to_string()],
            env: worktree.shell_env(),
        })
    }
}

zed::register_extension!(MaceExtension);
