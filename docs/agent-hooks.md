# Agent Hooks

After an agent writes or edits a text file, the hook runs:

```sh
slop-lint hook --agent <agent>
```

Supported agents:

- Claude Code
- Codex
- Gemini CLI
- OpenCode
- Kimi Code

Install automatically:

```sh
slop-lint install-hooks
```

Install for one agent:

```sh
slop-lint install-hooks --agent codex
slop-lint install-hooks --agent claude-code
slop-lint install-hooks --agent gemini-cli
slop-lint install-hooks --agent opencode
slop-lint install-hooks --agent kimi-code --scope global
```

Recommended agent behavior:

- `warn`: revise the file directly.
- `ask_user`: ask whether to revise the text or mute the rule.
- mute: add the rule id under `[mute].rules` in `.slop-lint.toml`.
