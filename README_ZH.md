# slop-lint

<p align="center">
  <img src="docs/public/favicon.svg" alt="slop-lint logo" width="96" height="96">
</p>

面向 Agent 的 AI 文本产物质量 Linter。

网站：<https://indenscale.github.io/slop-lint/>

`slop-lint` 会在 Agent 写入 Markdown 或纯文本后检查文件。它不试图证明一段文本
是不是 AI 写的，而是发现那些会让文本显得空泛、模板化、膨胀、不可信的模式。

可以把它理解成：面向 AI slop 的 `markdownlint`。

## 快速开始

任选一种安装方式。

macOS/Linux release 二进制：

```sh
curl -fsSL https://raw.githubusercontent.com/IndenScale/slop-lint/main/install.sh | sh
```

Windows PowerShell release 二进制：

```powershell
iwr https://raw.githubusercontent.com/IndenScale/slop-lint/main/install.ps1 -useb | iex
```

也可以从 crates.io 安装：

```sh
cargo install slop-lint
```

然后把已安装的二进制接入 Agent Hooks：

```sh
slop-lint install-hooks
```

这是预期的 happy path：先安装一次，再运行 `slop-lint install-hooks`。hook 安装器
会自动检测支持的 Agent，并写入它能安全管理的 hook 配置。

当前目标：

- Claude Code
- Codex
- Gemini CLI
- OpenCode
- Kimi Code

支持的 OS/arch release 资产：

- `slop-lint-aarch64-apple-darwin.tar.gz`
- `slop-lint-x86_64-apple-darwin.tar.gz`
- `slop-lint-aarch64-unknown-linux-gnu.tar.gz`
- `slop-lint-x86_64-unknown-linux-gnu.tar.gz`
- `slop-lint-aarch64-pc-windows-msvc.zip`
- `slop-lint-x86_64-pc-windows-msvc.zip`

也可以指定目标：

```sh
slop-lint install-hooks --agent claude-code
slop-lint install-hooks --agent codex
slop-lint install-hooks --agent gemini-cli
slop-lint install-hooks --agent opencode
slop-lint install-hooks --agent kimi-code --scope global
```

Hook 管理：

```sh
slop-lint install-hooks --dry-run
slop-lint hook-status
slop-lint uninstall-hooks --agent claude-code
```

## Hook 做什么

当 Agent 写入或编辑文本文件时，hook 会运行：

```sh
slop-lint hook --agent <agent>
```

它会从 Agent 传入的事件 JSON 中读取文件路径，执行 `slop-lint check`，再把结构化
反馈送回 Agent。

推荐的 Agent 行为：

- `warn`：直接修改文件。
- `ask_user`：询问用户是修改文本，还是 mute 这条规则。
- mute：把规则 ID 加到 `.slop-lint.toml` 的 `[mute].rules` 下。

## 手动使用

```sh
slop-lint check README.md
slop-lint check docs/ --format json
slop-lint check docs/ --format sarif --fail-on-warning > slop-lint.sarif
slop-lint init
```

默认情况下，`slop-lint check` 会扫描当前目录下的 Markdown 和文本类文件。

输出格式：

```sh
slop-lint check docs/ --format text
slop-lint check docs/ --format json
slop-lint check docs/ --format sarif
```

接入 CI、代码扫描或编辑器诊断时，优先使用 SARIF。

## 开发检查

在仓库任意目录运行完整本地检查：

```sh
scripts/check-all.sh
```

## 产品立场

- 默认高敏感。让 Agent 多修一次 warning，通常比让人审一段精致废话更便宜。
- 默认确定性。规则应该可解释、可复现，并且足够便宜，可以在每次文件写入后运行。
- 项目可 mute。如果某条 warning 在项目语境中是有意为之，Agent 可以把规则 ID
  加入项目配置。
- 不确定就询问。交互模式下，低置信度诊断可以要求 Agent 先问用户：应该修改文本，
  还是 mute 规则。

## 配置

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

[[rules]]
id = "brand.empty-superlative"
name = "Empty brand superlative"
level = "warning"
kind = "phrase_presence"
phrases = ["best-in-class", "world-class"]
message = "Avoid empty brand superlatives unless there is concrete evidence nearby."
suggestion = "Replace the superlative with a concrete property, benchmark, or proof point."
confidence = 0.7
```

行内 disable：

```md
<!-- slop-lint-disable-next-line slop.generic-conclusion -->
In conclusion, this sentence is intentionally formulaic.

This phrase is intentional. <!-- slop-lint-disable-line -->

<!-- slop-lint-disable-file slop.zh-generic-value -->
```

如果整个项目都接受某种风格，优先用项目级 `[mute].rules`；如果只是单行或单文件例外，
使用行内 disable。

## 贡献规则

规则质量来自真实误报、漏报和项目里的写作习惯。如果你遇到反复出现的模式，欢迎开
GitHub issue，并附上：

- 应该被标记的文本
- 不应该被标记的文本
- 期望动作：`warn` 或 `ask_user`

项目主页：<https://indenscale.github.io/slop-lint/>
作者主页：<https://indenscale.github.io/>

## 规则模型

规则是 TOML 数据驱动的，包括内置规则。Rust 二进制只保留 schema 和加载逻辑；
默认规则内容放在 `rulesets/` 下，并在构建时嵌入二进制。

第一版规则引擎支持：

- `phrase_presence`：当文本单元包含指定短语时产生诊断。
- `phrase_density`：当短语密度超过阈值时产生诊断。
- `phrase_pair_density`：当成对结构在局部文本中过密时产生诊断，例如同一段里反复使用
  `不是……而是……`。

分析器会按全文和段落两个层次切分文本，并跳过 Markdown fenced code block 和缩进
代码块。后续可以加入句子级、标点窗口、字数窗口等检查，而不改变 CLI 契约。

内置规则已经覆盖中英文模式，包括模板化提示语、空泛修饰词密度、泛化价值表达、
无证据重要性判断、模糊规模表达和模板化总结。

## 发布质量

CI 会在 push 和 pull request 上运行格式检查、Clippy 和测试。带 tag 的 release 会
通过 GitHub Actions 构建安装脚本所需资产，并随归档文件发布 `SHA256SUMS`。

安装脚本会在安装前用 release 中的 `SHA256SUMS` 校验下载归档。

crate 也已经按 crates.io 发布要求准备。tag、GitHub Release 和 crates.io 发布流程见
`RELEASE.md`。

## 路线图

- 使用真实中英文 Agent 文档校准成对结构密度规则。
- 为 Claude Code、Codex、Gemini CLI、OpenCode 和 Kimi Code 的真实 hook
  event payload 增加 fixture 兼容性测试。
- 增加针对安装脚本和 `cargo install` 的发布 smoke 检查。
- 针对学术写作、技术文档、社交媒体、品牌语气的外部规则包。
- 面向 PostToolUse Hook 的 diff-only 模式。
- 第一阶段优先支持 data-only 插件包；只有当规则逻辑确实需要代码时，再考虑 WASM
  插件。
