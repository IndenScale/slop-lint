<!-- markdownlint-disable-file MD033 -->

# slop-lint

<p align="center">
  <img src="docs/public/favicon.svg" alt="slop-lint logo" width="96" height="96">
</p>

Agent-facing lint for AI-generated text.

Website: <https://indenscale.github.io/slop-lint/>

`slop-lint` checks Markdown and plain text after an agent writes files. It does
not try to prove whether text is AI-written. It flags patterns that make text
feel vague, templated, inflated, or hard to trust.

Think of it as `markdownlint` for AI slop.

## Quick start

Choose one install method.

macOS/Linux release binary:

```sh
curl -fsSL https://raw.githubusercontent.com/IndenScale/slop-lint/main/install.sh | sh
```

Windows PowerShell release binary:

```powershell
iwr https://raw.githubusercontent.com/IndenScale/slop-lint/main/install.ps1 -useb | iex
```

Or install from crates.io:

```sh
cargo install slop-lint
```

Then attach the installed binary to your agent hooks:

```sh
slop-lint install-hooks
```

That is the intended happy path: install once, then run `slop-lint install-hooks`.
The hook installer auto-detects supported agents and writes the hook
configuration it can safely manage.

Supported targets:

- Claude Code
- Codex
- Gemini CLI
- OpenCode
- Kimi Code

Supported OS/arch release assets:

- `slop-lint-aarch64-apple-darwin.tar.gz`
- `slop-lint-x86_64-apple-darwin.tar.gz`
- `slop-lint-aarch64-unknown-linux-gnu.tar.gz`
- `slop-lint-x86_64-unknown-linux-gnu.tar.gz`
- `slop-lint-aarch64-pc-windows-msvc.zip`
- `slop-lint-x86_64-pc-windows-msvc.zip`

You can force a target:

```sh
slop-lint install-hooks --agent claude-code
slop-lint install-hooks --agent codex
slop-lint install-hooks --agent gemini-cli
slop-lint install-hooks --agent opencode
slop-lint install-hooks --agent kimi-code --scope global
```

Hook management:

```sh
slop-lint install-hooks --dry-run
slop-lint hook-status
slop-lint uninstall-hooks --agent claude-code
```

## What the hook does

When an agent writes or edits a text file, the hook runs:

```sh
slop-lint hook --agent <agent>
```

The hook reads the agent event JSON from stdin, extracts the touched file path,
runs `slop-lint check`, and sends structured feedback back to the agent.

Recommended agent behavior:

- `warn`: revise the file directly.
- `ask_user`: ask whether to revise the text or mute the rule.
- mute: add the rule id under `[mute].rules` in `.slop-lint.toml`.

## Manual use

```sh
slop-lint check README.md
slop-lint check docs/ --format json
slop-lint check docs/ --format sarif --fail-on-warning > slop-lint.sarif
slop-lint init
```

By default, `slop-lint check` scans Markdown and text-like files under the
current directory.

Output formats:

```sh
slop-lint check docs/ --format text
slop-lint check docs/ --format json
slop-lint check docs/ --format sarif
```

Use SARIF when wiring `slop-lint` into CI, code scanning, or editor diagnostics.

## Development checks

Run the full local check suite from anywhere inside the repository:

```sh
scripts/check-all.sh
```

## Product stance

- Sensitive by default. It is cheaper for an agent to revise a warning than for
  a human to review polished filler.
- Deterministic by default. Rules should be explainable, repeatable, and cheap
  enough to run after every file write.
- Mutable by project. If a warning is intentional, the agent can mute the rule
  in project config.
- Interactive when uncertain. Lower-confidence findings can tell the agent to
  ask the user before revising or muting.

## Configuration

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

Inline disables:

```md
<!-- slop-lint-disable-next-line slop.generic-conclusion -->
In conclusion, this sentence is intentionally formulaic.

This phrase is intentional. <!-- slop-lint-disable-line -->

<!-- slop-lint-disable-file slop.zh-generic-value -->
```

Prefer project-level `[mute].rules` when a whole project wants a style, and
inline disables when a single line or file has a local exception.

## Contribute Rules

Rule quality depends on real false positives, missed slop, and project-specific
writing habits. If you have a recurring pattern, open a GitHub issue with:

- the text that should be flagged
- the text that should not be flagged
- the rule action you expect: `warn` or `ask_user`

Project homepage: <https://indenscale.github.io/slop-lint/>
Author homepage: <https://indenscale.github.io/>

## Rule model

Rules are data-driven TOML, including the built-in rules. The Rust binary keeps
only the schema and loading logic; default rule content lives under `rulesets/`
and is embedded into the binary at build time.

The first rule engine supports:

- `phrase_presence`: warn when a unit contains configured phrases.
- `phrase_density`: warn when phrase density crosses a threshold.
- `phrase_pair_density`: warn when a paired construction is locally dense, such
  as repeated `不是...而是...` contrasts in one paragraph.

The analyzer segments text at document and paragraph levels, and skips Markdown
fenced and indented code blocks. Sentence-level and punctuation-window checks can
be added without changing the CLI contract.

The built-in rules cover English and Chinese patterns, including formulaic
transitions, empty intensifier density, generic benefit claims, evidence-free
importance claims, vague scale terms, and templated conclusions.

## Release quality

CI runs formatting, Clippy, and tests on pushes and pull requests. Tagged
releases build the install-script assets with GitHub Actions and publish
`SHA256SUMS` alongside the archives.

The install scripts verify downloaded archives against release checksums before
installing the binary.

The crate is also prepared for crates.io publication. See `RELEASE.md` for the
tag, GitHub Release, and crates.io release flow.

## Roadmap

- Calibrate paired-construction density rules on real Chinese and English
  agent-written documents.
- Add fixture-based hook compatibility tests for Claude Code, Codex, Gemini CLI,
  OpenCode, and Kimi Code event payloads.
- Add release smoke checks for install scripts and `cargo install`.
- External rule packs for academic writing, technical docs, social posts, and brand voice.
- Diff-only mode for PostToolUse hooks.
- Data-only plugin packages first, WASM plugins later if rule logic needs code.
