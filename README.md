# Mace Zed Extension

This extension provides Mace language support for Zed, including the grammar,
language server wiring, and editor snippets. Mace is a new deterministic,
strongly typed configuration language created in 2026; this is its first stable
Zed extension release.

The language server prefers a downloaded Mace binary when one is already
present in the extension cache. Otherwise it reads `mace-version.json` from the
extension root, selects the GoReleaser asset for the current platform, and
downloads that release before starting the server.

## Snippets

Zed extension snippets live in the top-level `snippets/` directory as one JSON
file per language scope. Mace snippets are defined in `snippets/mace.json` and
registered through the `snippets` field in `extension.toml`.
