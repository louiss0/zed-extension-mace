use zed_extension_api::{self as zed};

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
        let go = worktree
            .which("go")
            .ok_or_else(|| "could not find `go` on PATH".to_string())?;

        Ok(zed::Command {
            command: go,
            args: vec!["run".to_string(), "./cmd".to_string(), "lsp".to_string()],
            env: worktree.shell_env(),
        })
    }
}

zed::register_extension!(MaceExtension);
