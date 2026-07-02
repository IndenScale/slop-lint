# slop-lint Agent Notes

## Project intent

`slop-lint` is a Rust CLI for detecting low-quality AI text patterns in
Markdown and plain text artifacts. It is meant to run as a fast global binary in
agent hooks, especially after file writes.

## Product rules

- Prefer deterministic, explainable checks over model-based judging.
- Keep default sensitivity high.
- Diagnostics should be actionable for agents.
- Low-confidence findings in interactive mode should use `action = "ask_user"`.
- A project may mute rules explicitly in `.slop-lint.toml`.
- Rule content must stay data-driven. Put built-in rules in `rulesets/*.toml`;
  keep Rust focused on schema, loading, merging, and execution.
- Data-only rulesets are preferred for plugins until executable plugin logic is clearly necessary.

## Tooling

- Use Cargo for Rust tasks.
- Do not introduce Node or Python runtime requirements for the core CLI.
