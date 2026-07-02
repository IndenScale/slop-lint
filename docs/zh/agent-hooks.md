# Agent Hooks

Agent 写入或编辑文本文件后，hook 会运行：

```sh
slop-lint hook --agent <agent>
```

支持的 Agent：

- Claude Code
- Codex
- Gemini CLI
- OpenCode
- Kimi Code

自动安装：

```sh
slop-lint install-hooks
```

指定 Agent 安装：

```sh
slop-lint install-hooks --agent codex
slop-lint install-hooks --agent claude-code
slop-lint install-hooks --agent gemini-cli
slop-lint install-hooks --agent opencode
slop-lint install-hooks --agent kimi-code --scope global
```

建议 Agent 行为：

- `warn`：直接改写文件。
- `ask_user`：询问用户是改写文本还是 mute 规则。
- mute：把规则 ID 加到 `.slop-lint.toml` 的 `[mute].rules`。
