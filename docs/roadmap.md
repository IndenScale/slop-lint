# Roadmap

The 0.1 line is a public beta for agent-heavy writing workflows.

Near-term work:

- Calibrate paired-construction density rules on real Chinese and English
  agent-written documents.
- Add fixture-based hook compatibility tests for Claude Code, Codex, Gemini CLI,
  OpenCode, and Kimi Code event payloads.
- Add release smoke checks for install scripts and `cargo install`.
- Ship external rule packs for academic writing, technical docs, social posts,
  and brand voice.
- Add diff-only mode for PostToolUse hooks.

Longer-term direction:

- Data-only plugin packages first.
- WASM plugins only if rule logic clearly needs executable code.
