# Changelog

All notable changes to `slop-lint` will be documented in this file.

## Unreleased

- Added `phrase_pair_density` rules for locally dense paired constructions.
- Added the built-in Chinese `slop.zh-not-but-density` rule for repeated
  `不是...而是...` contrast patterns.
- Added install smoke testing for GitHub Release install scripts and
  crates.io-based `cargo install`.
- Added a VitePress documentation and landing site with GitHub Pages deployment.
- Added English and Simplified Chinese landing pages, localized docs, and a
  VitePress language switcher.
- Improved the custom dark-mode palette for the landing page.
- Added rule-contribution links to the landing page footer and README files.

## 0.1.0 - 2026-07-02

Initial public release.

- Added deterministic TOML-backed rule engine for Markdown and text artifacts.
- Added built-in English and Chinese rulesets.
- Added text, JSON, and SARIF output formats.
- Added project/user config loading, project mutes, and inline disable comments.
- Added hook install/status/uninstall support for Claude Code, Codex, Gemini CLI,
  OpenCode, and Kimi Code.
- Added GitHub Release packaging for macOS, Linux, and Windows with SHA256
  checksums.
