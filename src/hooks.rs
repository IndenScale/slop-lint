use std::env;
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use clap::ValueEnum;
use serde_json::{Value, json};

use crate::analyze::analyze_text;
use crate::config::Config;

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum HookAgent {
    Auto,
    ClaudeCode,
    Codex,
    GeminiCli,
    Opencode,
    KimiCode,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum HookScope {
    Project,
    Global,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum Agent {
    Generic,
    ClaudeCode,
    Codex,
    GeminiCli,
    Opencode,
    KimiCode,
}

#[derive(Debug)]
struct InstallResult {
    agent: &'static str,
    path: PathBuf,
    changed: bool,
}

pub fn install_hooks(agent: HookAgent, scope: HookScope, dry_run: bool) -> Result<()> {
    let agents = match agent {
        HookAgent::Auto => detect_agents(),
        agent => vec![agent],
    };

    if agents.is_empty() {
        println!("slop-lint: no supported agent config detected");
        println!("Try one of:");
        println!("  slop-lint install-hooks --agent claude-code");
        println!("  slop-lint install-hooks --agent codex");
        println!("  slop-lint install-hooks --agent gemini-cli");
        println!("  slop-lint install-hooks --agent opencode");
        println!("  slop-lint install-hooks --agent kimi-code --scope global");
        return Ok(());
    }

    let mut results = Vec::new();
    for agent in agents {
        match agent {
            HookAgent::Auto => {}
            HookAgent::ClaudeCode => results.push(install_claude(scope, dry_run)?),
            HookAgent::Codex => results.push(install_codex(scope, dry_run)?),
            HookAgent::GeminiCli => results.push(install_gemini(scope, dry_run)?),
            HookAgent::Opencode => results.push(install_opencode(scope, dry_run)?),
            HookAgent::KimiCode => results.push(install_kimi(scope, dry_run)?),
        }
    }

    for result in results {
        let verb = if dry_run {
            if result.changed {
                "would install"
            } else {
                "already present"
            }
        } else if result.changed {
            "installed"
        } else {
            "already present"
        };
        println!(
            "slop-lint: {verb} {} hook at {}",
            result.agent,
            result.path.display()
        );
    }
    Ok(())
}

pub fn hook_status(agent: HookAgent, scope: HookScope) -> Result<()> {
    for agent in resolve_agents(agent) {
        let (name, path) = hook_path(agent, scope)?;
        let installed = hook_installed(agent, &path)?;
        let status = if installed { "installed" } else { "missing" };
        println!("slop-lint: {name}: {status} at {}", path.display());
    }
    Ok(())
}

pub fn uninstall_hooks(agent: HookAgent, scope: HookScope) -> Result<()> {
    for agent in resolve_agents(agent) {
        let (name, path) = hook_path(agent, scope)?;
        let changed = match agent {
            HookAgent::ClaudeCode => remove_json_hook(&path, "PostToolUse")?,
            HookAgent::Codex => remove_json_hook(&path, "PostToolUse")?,
            HookAgent::GeminiCli => remove_json_hook(&path, "AfterTool")?,
            HookAgent::Opencode => remove_file_if_sloplint(&path)?,
            HookAgent::KimiCode => remove_kimi_hook(&path)?,
            HookAgent::Auto => false,
        };
        let status = if changed { "removed" } else { "not present" };
        println!("slop-lint: {status} {name} hook at {}", path.display());
    }
    Ok(())
}

pub fn run_hook(agent: Agent) -> Result<()> {
    let mut raw = String::new();
    io::stdin().read_to_string(&mut raw)?;
    let input: Value = serde_json::from_str(&raw).unwrap_or_else(|_| json!({}));
    let cwd = input
        .get("cwd")
        .or_else(|| input.get("GEMINI_CWD"))
        .and_then(Value::as_str)
        .map(PathBuf::from)
        .unwrap_or(env::current_dir()?);

    let config = Config::load_hierarchy(cwd.clone())?;
    let rules = config.rules()?;
    let mut diagnostics = Vec::new();
    let files = extract_files(&input, &cwd, &config);

    for file in files {
        if let Ok(text) = fs::read_to_string(&file) {
            diagnostics.extend(analyze_text(&file, &text, &config, &rules));
        }
    }

    if diagnostics.is_empty() {
        print_hook_ok(agent)?;
        return Ok(());
    }

    let feedback = format_hook_feedback(&diagnostics);
    print_hook_feedback(agent, &feedback)?;
    Ok(())
}

fn detect_agents() -> Vec<HookAgent> {
    let mut agents = Vec::new();
    let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    if cwd.join(".claude").exists() || home_join(".claude").exists() || command_exists("claude") {
        agents.push(HookAgent::ClaudeCode);
    }
    if cwd.join(".codex").exists() || home_join(".codex").exists() || command_exists("codex") {
        agents.push(HookAgent::Codex);
    }
    if cwd.join(".gemini").exists() || home_join(".gemini").exists() || command_exists("gemini") {
        agents.push(HookAgent::GeminiCli);
    }
    if cwd.join(".opencode").exists()
        || cwd.join("opencode.json").exists()
        || home_join(".config/opencode").exists()
        || command_exists("opencode")
    {
        agents.push(HookAgent::Opencode);
    }
    if home_join(".kimi").exists() || command_exists("kimi") {
        agents.push(HookAgent::KimiCode);
    }
    agents
}

fn install_claude(scope: HookScope, dry_run: bool) -> Result<InstallResult> {
    let (_, path) = hook_path(HookAgent::ClaudeCode, scope)?;
    let hook = json!({
        "matcher": "Write|Edit|MultiEdit",
        "hooks": [{
            "type": "command",
            "command": "slop-lint hook --agent claude-code",
            "timeout": 30
        }]
    });
    let changed = merge_json_hook(&path, "PostToolUse", hook, dry_run)?;
    Ok(InstallResult {
        agent: "Claude Code",
        path,
        changed,
    })
}

fn install_codex(scope: HookScope, dry_run: bool) -> Result<InstallResult> {
    let (_, path) = hook_path(HookAgent::Codex, scope)?;
    let hook = json!({
        "matcher": "Write|Edit|apply_patch",
        "hooks": [{
            "type": "command",
            "command": "slop-lint hook --agent codex",
            "timeout": 30,
            "statusMessage": "Checking text slop"
        }]
    });
    let changed = merge_json_hook(&path, "PostToolUse", hook, dry_run)?;
    Ok(InstallResult {
        agent: "Codex",
        path,
        changed,
    })
}

fn install_gemini(scope: HookScope, dry_run: bool) -> Result<InstallResult> {
    let (_, path) = hook_path(HookAgent::GeminiCli, scope)?;
    let hook = json!({
        "matcher": "write_file|replace|edit|write.*",
        "hooks": [{
            "name": "slop-lint",
            "type": "command",
            "command": "slop-lint hook --agent gemini-cli",
            "timeout": 30000,
            "description": "Check AI text slop after file writes."
        }]
    });
    let changed = merge_json_hook(&path, "AfterTool", hook, dry_run)?;
    Ok(InstallResult {
        agent: "Gemini CLI",
        path,
        changed,
    })
}

fn install_opencode(scope: HookScope, dry_run: bool) -> Result<InstallResult> {
    let (_, path) = hook_path(HookAgent::Opencode, scope)?;
    let content = opencode_plugin();
    let changed = read_to_string_optional(&path)? != Some(content.to_string());
    if changed && !dry_run {
        ensure_parent(&path)?;
        fs::write(&path, content).with_context(|| format!("failed to write {}", path.display()))?;
    }
    Ok(InstallResult {
        agent: "OpenCode",
        path,
        changed,
    })
}

fn install_kimi(scope: HookScope, dry_run: bool) -> Result<InstallResult> {
    if scope == HookScope::Project {
        println!("slop-lint: Kimi Code hooks are user-level today; writing ~/.kimi/config.toml");
    }
    let (_, path) = hook_path(HookAgent::KimiCode, HookScope::Global)?;
    let hook = r#"

[[hooks]]
event = "PostToolUse"
matcher = "WriteFile|StrReplaceFile|Edit|Write"
command = "slop-lint hook --agent kimi-code"
timeout = 30
"#;
    let old = read_to_string_optional(&path)?.unwrap_or_default();
    let changed = !old.contains("slop-lint hook --agent kimi-code");
    if changed && !dry_run {
        ensure_parent(&path)?;
        fs::write(&path, format!("{old}{hook}"))
            .with_context(|| format!("failed to write {}", path.display()))?;
    }
    Ok(InstallResult {
        agent: "Kimi Code",
        path,
        changed,
    })
}

fn merge_json_hook(path: &Path, event: &str, hook: Value, dry_run: bool) -> Result<bool> {
    let mut root = match read_to_string_optional(path)? {
        Some(raw) if !raw.trim().is_empty() => serde_json::from_str::<Value>(&raw)
            .with_context(|| format!("failed to parse {}", path.display()))?,
        _ => json!({}),
    };

    if !root.is_object() {
        bail!("{} must contain a JSON object", path.display());
    }

    let hooks = root
        .as_object_mut()
        .unwrap()
        .entry("hooks")
        .or_insert_with(|| json!({}));
    if !hooks.is_object() {
        bail!("{}.hooks must contain a JSON object", path.display());
    }
    let event_hooks = hooks
        .as_object_mut()
        .unwrap()
        .entry(event)
        .or_insert_with(|| json!([]));
    if !event_hooks.is_array() {
        bail!("{}.hooks.{event} must contain an array", path.display());
    }

    let exists = event_hooks.as_array().unwrap().iter().any(|entry| {
        serde_json::to_string(entry)
            .map(|raw| raw.contains("slop-lint hook"))
            .unwrap_or(false)
    });
    if exists {
        return Ok(false);
    }

    event_hooks.as_array_mut().unwrap().push(hook);
    if !dry_run {
        ensure_parent(path)?;
        fs::write(path, format!("{}\n", serde_json::to_string_pretty(&root)?))
            .with_context(|| format!("failed to write {}", path.display()))?;
    }
    Ok(true)
}

fn remove_json_hook(path: &Path, event: &str) -> Result<bool> {
    let Some(raw) = read_to_string_optional(path)? else {
        return Ok(false);
    };
    let mut root: Value = serde_json::from_str(&raw)
        .with_context(|| format!("failed to parse {}", path.display()))?;
    let Some(event_hooks) = root
        .get_mut("hooks")
        .and_then(Value::as_object_mut)
        .and_then(|hooks| hooks.get_mut(event))
        .and_then(Value::as_array_mut)
    else {
        return Ok(false);
    };
    let old_len = event_hooks.len();
    event_hooks.retain(|entry| {
        serde_json::to_string(entry)
            .map(|raw| !raw.contains("slop-lint hook"))
            .unwrap_or(true)
    });
    let changed = old_len != event_hooks.len();
    if changed {
        fs::write(path, format!("{}\n", serde_json::to_string_pretty(&root)?))
            .with_context(|| format!("failed to write {}", path.display()))?;
    }
    Ok(changed)
}

fn hook_installed(agent: HookAgent, path: &Path) -> Result<bool> {
    match agent {
        HookAgent::ClaudeCode | HookAgent::Codex => json_hook_installed(path, "PostToolUse"),
        HookAgent::GeminiCli => json_hook_installed(path, "AfterTool"),
        HookAgent::Opencode => Ok(read_to_string_optional(path)?
            .map(|raw| raw.contains("slop-lint check"))
            .unwrap_or(false)),
        HookAgent::KimiCode => Ok(read_to_string_optional(path)?
            .map(|raw| raw.contains("slop-lint hook --agent kimi-code"))
            .unwrap_or(false)),
        HookAgent::Auto => Ok(false),
    }
}

fn json_hook_installed(path: &Path, event: &str) -> Result<bool> {
    let Some(raw) = read_to_string_optional(path)? else {
        return Ok(false);
    };
    let root: Value = serde_json::from_str(&raw)
        .with_context(|| format!("failed to parse {}", path.display()))?;
    Ok(root
        .get("hooks")
        .and_then(Value::as_object)
        .and_then(|hooks| hooks.get(event))
        .and_then(Value::as_array)
        .map(|hooks| {
            hooks.iter().any(|entry| {
                serde_json::to_string(entry)
                    .map(|raw| raw.contains("slop-lint hook"))
                    .unwrap_or(false)
            })
        })
        .unwrap_or(false))
}

fn remove_file_if_sloplint(path: &Path) -> Result<bool> {
    let Some(raw) = read_to_string_optional(path)? else {
        return Ok(false);
    };
    if !raw.contains("slop-lint") {
        return Ok(false);
    }
    fs::remove_file(path).with_context(|| format!("failed to remove {}", path.display()))?;
    Ok(true)
}

fn remove_kimi_hook(path: &Path) -> Result<bool> {
    let Some(raw) = read_to_string_optional(path)? else {
        return Ok(false);
    };
    if !raw.contains("slop-lint hook --agent kimi-code") {
        return Ok(false);
    }
    let mut next = Vec::new();
    let mut block = Vec::new();
    let mut in_hook_block = false;

    for line in raw.lines() {
        if line.trim() == "[[hooks]]" {
            flush_kimi_block(&mut next, &mut block);
            in_hook_block = true;
        }
        if in_hook_block {
            block.push(line.to_string());
        } else {
            next.push(line.to_string());
        }
    }
    flush_kimi_block(&mut next, &mut block);

    let mut output = next.join("\n");
    if !output.is_empty() {
        output.push('\n');
    }
    fs::write(path, output).with_context(|| format!("failed to write {}", path.display()))?;
    Ok(true)
}

fn flush_kimi_block(next: &mut Vec<String>, block: &mut Vec<String>) {
    if block.is_empty() {
        return;
    }
    let raw = block.join("\n");
    if !raw.contains("slop-lint hook --agent kimi-code") {
        next.append(block);
    } else {
        block.clear();
    }
}

fn resolve_agents(agent: HookAgent) -> Vec<HookAgent> {
    match agent {
        HookAgent::Auto => detect_agents(),
        agent => vec![agent],
    }
}

fn hook_path(agent: HookAgent, scope: HookScope) -> Result<(&'static str, PathBuf)> {
    match agent {
        HookAgent::ClaudeCode => Ok((
            "Claude Code",
            match scope {
                HookScope::Project => env::current_dir()?.join(".claude/settings.json"),
                HookScope::Global => home_join(".claude/settings.json"),
            },
        )),
        HookAgent::Codex => Ok((
            "Codex",
            match scope {
                HookScope::Project => env::current_dir()?.join(".codex/hooks.json"),
                HookScope::Global => home_join(".codex/hooks.json"),
            },
        )),
        HookAgent::GeminiCli => Ok((
            "Gemini CLI",
            match scope {
                HookScope::Project => env::current_dir()?.join(".gemini/settings.json"),
                HookScope::Global => home_join(".gemini/settings.json"),
            },
        )),
        HookAgent::Opencode => Ok((
            "OpenCode",
            match scope {
                HookScope::Project => env::current_dir()?.join(".opencode/plugins/slop-lint.js"),
                HookScope::Global => home_join(".config/opencode/plugins/slop-lint.js"),
            },
        )),
        HookAgent::KimiCode => Ok(("Kimi Code", home_join(".kimi/config.toml"))),
        HookAgent::Auto => bail!("auto does not have a single hook path"),
    }
}

fn extract_files(input: &Value, cwd: &Path, config: &Config) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_paths(input, &mut files);
    let mut files = files
        .into_iter()
        .map(|path| {
            if path.is_absolute() {
                path
            } else {
                cwd.join(path)
            }
        })
        .filter(|path| path.is_file() && should_check_hook_path(path, config))
        .collect::<Vec<_>>();
    files.sort();
    files.dedup();
    files
}

fn collect_paths(value: &Value, files: &mut Vec<PathBuf>) {
    match value {
        Value::Object(map) => {
            for (key, value) in map {
                if is_path_key(key)
                    && let Some(path) = value.as_str()
                {
                    files.push(PathBuf::from(path));
                }
                collect_paths(value, files);
            }
        }
        Value::Array(values) => {
            for value in values {
                collect_paths(value, files);
            }
        }
        _ => {}
    }
}

fn is_path_key(key: &str) -> bool {
    matches!(
        key,
        "file_path" | "filePath" | "filepath" | "path" | "absolutePath" | "target_file"
    )
}

fn should_check_hook_path(path: &Path, config: &Config) -> bool {
    let Some(ext) = path.extension().and_then(|value| value.to_str()) else {
        return false;
    };
    config
        .file_extensions
        .iter()
        .any(|allowed| allowed.eq_ignore_ascii_case(ext))
}

fn format_hook_feedback(diagnostics: &[crate::analyze::Diagnostic]) -> String {
    let mut lines = vec![format!(
        "slop-lint found {} text quality diagnostic(s).",
        diagnostics.len()
    )];
    for diagnostic in diagnostics.iter().take(8) {
        lines.push(format!(
            "- {}:{}:{} [{}] {} action={}",
            diagnostic.path.display(),
            diagnostic.line,
            diagnostic.column,
            diagnostic.rule_id,
            diagnostic.message,
            diagnostic.action
        ));
        if let Some(suggestion) = diagnostic.suggestion.as_ref() {
            lines.push(format!("  suggestion: {suggestion}"));
        }
    }
    if diagnostics.len() > 8 {
        lines.push(format!("- ... {} more", diagnostics.len() - 8));
    }
    lines.join("\n")
}

fn print_hook_ok(agent: Agent) -> Result<()> {
    if agent == Agent::GeminiCli {
        println!("{}", json!({ "suppressOutput": true }));
    }
    Ok(())
}

fn print_hook_feedback(agent: Agent, feedback: &str) -> Result<()> {
    let output = match agent {
        Agent::GeminiCli => json!({
            "systemMessage": feedback,
            "hookSpecificOutput": {
                "hookEventName": "AfterTool",
                "additionalContext": feedback
            }
        }),
        Agent::ClaudeCode => json!({
            "hookSpecificOutput": {
                "hookEventName": "PostToolUse",
                "additionalContext": feedback
            }
        }),
        Agent::Codex => json!({
            "hookSpecificOutput": {
                "hookEventName": "PostToolUse",
                "additionalContext": feedback
            }
        }),
        Agent::KimiCode => json!({
            "hookSpecificOutput": {
                "hookEventName": "PostToolUse",
                "additionalContext": feedback
            }
        }),
        Agent::Opencode | Agent::Generic => json!({
            "hookSpecificOutput": {
                "additionalContext": feedback
            }
        }),
    };
    println!("{}", serde_json::to_string(&output)?);
    Ok(())
}

fn opencode_plugin() -> &'static str {
    r#"export const SlopLintPlugin = async ({ $, client }) => {
  const collectPaths = (value, out = []) => {
    if (!value || typeof value !== "object") return out
    if (Array.isArray(value)) {
      for (const item of value) collectPaths(item, out)
      return out
    }
    for (const [key, child] of Object.entries(value)) {
      if (["file_path", "filePath", "filepath", "path", "absolutePath", "target_file"].includes(key) && typeof child === "string") {
        out.push(child)
      }
      collectPaths(child, out)
    }
    return out
  }

  return {
    "tool.execute.after": async (input, output) => {
      const paths = collectPaths({ input, output })
      for (const path of paths) {
        const result = await $`slop-lint check ${path} --format json`.quiet().nothrow()
        if (result.stdout && !result.stdout.includes('"diagnostic_count": 0')) {
          await client.app.log({
            body: {
              service: "slop-lint",
              level: "warn",
              message: result.stdout,
            },
          })
        }
      }
    },
  }
}
"#
}

fn ensure_parent(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    Ok(())
}

fn read_to_string_optional(path: &Path) -> Result<Option<String>> {
    match fs::read_to_string(path) {
        Ok(raw) => Ok(Some(raw)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error).with_context(|| format!("failed to read {}", path.display())),
    }
}

fn home_join(path: &str) -> PathBuf {
    env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
        .join(path)
}

fn command_exists(command: &str) -> bool {
    let Some(path) = env::var_os("PATH") else {
        return false;
    };
    env::split_paths(&path).any(|dir| dir.join(command).is_file())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use serde_json::json;
    use tempfile::tempdir;

    use super::*;

    #[test]
    fn extracts_supported_paths_from_nested_hook_events() {
        let dir = tempdir().unwrap();
        let docs = dir.path().join("docs");
        fs::create_dir(&docs).unwrap();
        fs::write(docs.join("note.md"), "It is important to note this.").unwrap();
        fs::write(docs.join("skip.rs"), "fn main() {}").unwrap();

        let event = json!({
            "cwd": dir.path(),
            "tool_input": {
                "file_path": "docs/note.md",
                "edits": [
                    { "path": docs.join("note.md") },
                    { "target_file": "docs/skip.rs" }
                ]
            }
        });

        let files = extract_files(&event, dir.path(), &Config::default());

        assert_eq!(files, vec![docs.join("note.md")]);
    }

    #[test]
    fn ignores_missing_and_unsupported_hook_paths() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("notes.md"), "plain").unwrap();
        fs::write(dir.path().join("code.rs"), "fn main() {}").unwrap();

        let event = json!({
            "tool_input": {
                "file_path": "missing.md",
                "path": "code.rs",
                "absolutePath": dir.path().join("notes.md")
            }
        });

        let files = extract_files(&event, dir.path(), &Config::default());

        assert_eq!(files, vec![dir.path().join("notes.md")]);
    }
}
