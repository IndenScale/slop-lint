# Configuration

`slop-lint` merges configuration in this order:

1. built-in defaults
2. user config at `~/.config/slop-lint/config.toml`
3. project config found in ancestors:
   `.slop-lint.toml`, `slop-lint.toml`, or `sloplint.toml`
4. explicit `--config`

Create a starter config:

```sh
slop-lint init
```

Example:

```toml
sensitivity = "high"
mode = "interactive"
uncertain_action = "ask_user"
file_extensions = ["md", "mdx", "txt", "rst", "adoc"]
extends = ["default", "zh"]
ruleset_paths = []

[mute]
rules = ["slop.generic-conclusion"]
```

Inline disables:

```md
<!-- slop-lint-disable-next-line slop.generic-conclusion -->
In conclusion, this sentence is intentionally formulaic.

This phrase is intentional. <!-- slop-lint-disable-line -->

<!-- slop-lint-disable-file slop.zh-generic-value -->
```
