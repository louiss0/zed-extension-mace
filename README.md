# Mace Zed Extension

This extension provides Mace language support for Zed, including the grammar,
language server wiring, and editor snippets.

It tracks the current Mace type syntax, including both `variant[...]` closed
alternatives and `union[...]` schema composition.

## Snippets

Zed extension snippets live in the top-level `snippets/` directory as one JSON
file per language scope. Mace snippets are defined in `snippets/mace.json`.
