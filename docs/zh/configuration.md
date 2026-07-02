# 配置

`slop-lint` 按以下顺序合并配置：

1. 内置默认值
2. 用户配置：`~/.config/slop-lint/config.toml`
3. 从当前目录向上查找项目配置：
   `.slop-lint.toml`、`slop-lint.toml` 或 `sloplint.toml`
4. 显式传入的 `--config`

创建初始配置：

```sh
slop-lint init
```

示例：

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

行内 disable：

```md
<!-- slop-lint-disable-next-line slop.generic-conclusion -->
In conclusion, this sentence is intentionally formulaic.

This phrase is intentional. <!-- slop-lint-disable-line -->

<!-- slop-lint-disable-file slop.zh-generic-value -->
```
