use std::path::{Path, PathBuf};

use regex::Regex;
use serde::Serialize;

use crate::config::Config;
use crate::rules::{Level, Rule, RuleKind};

#[derive(Debug, Serialize)]
pub struct Diagnostic {
    pub path: PathBuf,
    pub line: usize,
    pub column: usize,
    pub level: Level,
    pub rule_id: String,
    pub message: String,
    pub suggestion: Option<String>,
    pub action: String,
    pub confidence: f32,
    pub matched: Vec<String>,
}

#[derive(Debug)]
struct Unit<'a> {
    text: &'a str,
    offset: usize,
    word_count: usize,
    scope: UnitScope,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum UnitScope {
    Document,
    Paragraph,
}

pub fn analyze_text(path: &Path, text: &str, config: &Config, rules: &[Rule]) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let units = segment_units(text);
    for rule in rules {
        match rule.kind {
            RuleKind::PhrasePresence => {
                for unit in &units {
                    if unit.scope == UnitScope::Document {
                        continue;
                    }
                    let matches = find_phrases(unit.text, &rule.phrases);
                    if matches.len() >= rule.min_occurrences.unwrap_or(1) {
                        diagnostics.push(to_diagnostic(
                            path,
                            text,
                            unit.offset,
                            rule,
                            matches,
                            config,
                        ));
                    }
                }
            }
            RuleKind::PhraseDensity => {
                for unit in &units {
                    if unit.word_count == 0 {
                        continue;
                    }
                    let matches = find_phrases(unit.text, &rule.phrases);
                    let density = matches.len() as f32 * 1000.0 / unit.word_count as f32;
                    let threshold = rule.threshold_per_1000_words.unwrap_or(0.0)
                        * config.threshold_multiplier();
                    let min_occurrences = rule.min_occurrences.unwrap_or(1);
                    if matches.len() >= min_occurrences && density >= threshold {
                        diagnostics.push(to_diagnostic(
                            path,
                            text,
                            unit.offset,
                            rule,
                            matches,
                            config,
                        ));
                    }
                }
            }
        }
    }
    diagnostics.sort_by(|left, right| {
        left.path
            .cmp(&right.path)
            .then(left.line.cmp(&right.line))
            .then(left.column.cmp(&right.column))
            .then(left.rule_id.cmp(&right.rule_id))
    });
    diagnostics.dedup_by(|left, right| {
        left.path == right.path
            && left.line == right.line
            && left.column == right.column
            && left.rule_id == right.rule_id
    });
    diagnostics
}

fn to_diagnostic(
    path: &Path,
    full_text: &str,
    offset: usize,
    rule: &Rule,
    matched: Vec<String>,
    config: &Config,
) -> Diagnostic {
    let (line, column) = line_column(full_text, offset);
    Diagnostic {
        path: path.to_path_buf(),
        line,
        column,
        level: rule.level,
        rule_id: rule.id.clone(),
        message: rule.message.clone(),
        suggestion: rule.suggestion.clone(),
        action: config.diagnostic_action(rule.confidence).to_string(),
        confidence: rule.confidence,
        matched,
    }
}

fn segment_units(text: &str) -> Vec<Unit<'_>> {
    let plain_ranges = markdown_plain_ranges(text);
    let mut units = Vec::new();
    for (start, end) in &plain_ranges {
        let slice = &text[*start..*end];
        if slice.trim().is_empty() {
            continue;
        }
        units.push(Unit {
            text: slice,
            offset: *start,
            word_count: count_words(slice),
            scope: UnitScope::Document,
        });
    }

    let mut offset = 0;
    for paragraph in text.split("\n\n") {
        let trimmed = paragraph.trim();
        if !trimmed.is_empty() {
            let leading_ws = paragraph.len() - paragraph.trim_start().len();
            let start = offset + leading_ws;
            let end = start + trimmed.len();
            if plain_ranges
                .iter()
                .any(|(plain_start, plain_end)| start >= *plain_start && end <= *plain_end)
            {
                units.push(Unit {
                    text: trimmed,
                    offset: start,
                    word_count: count_words(trimmed),
                    scope: UnitScope::Paragraph,
                });
            }
        }
        offset += paragraph.len() + 2;
    }
    units
}

fn find_phrases(text: &str, phrases: &[String]) -> Vec<String> {
    let lower = text.to_lowercase();
    let mut matches = Vec::new();
    for phrase in phrases {
        let escaped = regex::escape(&phrase.to_lowercase());
        let phrase_pattern = escaped.replace(r"\ ", r"\s+");
        let pattern = if phrase.chars().any(|ch| ch.is_ascii_alphabetic()) {
            format!(r"\b{}\b", phrase_pattern)
        } else {
            phrase_pattern
        };
        if let Ok(regex) = Regex::new(&pattern) {
            for _ in regex.find_iter(&lower) {
                matches.push(phrase.clone());
            }
        }
    }
    matches
}

fn count_words(text: &str) -> usize {
    let whitespace_words = text
        .split_whitespace()
        .filter(|part| part.chars().any(|ch| ch.is_alphanumeric() && !is_cjk(ch)))
        .count();
    let cjk_chars = text.chars().filter(|ch| is_cjk(*ch)).count();
    whitespace_words + cjk_chars
}

fn is_cjk(ch: char) -> bool {
    matches!(
        ch as u32,
        0x3400..=0x4DBF
            | 0x4E00..=0x9FFF
            | 0xF900..=0xFAFF
            | 0x3040..=0x30FF
            | 0xAC00..=0xD7AF
            | 0x20000..=0x2A6DF
            | 0x2A700..=0x2B73F
            | 0x2B740..=0x2B81F
            | 0x2B820..=0x2CEAF
            | 0x2CEB0..=0x2EBEF
            | 0x30000..=0x3134F
    )
}

fn markdown_plain_ranges(text: &str) -> Vec<(usize, usize)> {
    let mut ranges = Vec::new();
    let mut plain_start = 0;
    let mut offset = 0;
    let mut in_fence = false;

    for line in text.split_inclusive('\n') {
        let trimmed_start = line.trim_start();
        let is_fence = trimmed_start.starts_with("```") || trimmed_start.starts_with("~~~");
        let is_indented_code = !in_fence && (line.starts_with("    ") || line.starts_with('\t'));
        let line_end = offset + line.len();

        if is_fence {
            if !in_fence && plain_start < offset {
                ranges.push((plain_start, offset));
            }
            in_fence = !in_fence;
            plain_start = line_end;
        } else if is_indented_code {
            if plain_start < offset {
                ranges.push((plain_start, offset));
            }
            plain_start = line_end;
        } else if !in_fence && plain_start > offset {
            plain_start = offset;
        }

        offset = line_end;
    }

    if !in_fence && plain_start < text.len() {
        ranges.push((plain_start, text.len()));
    }
    ranges
}

fn line_column(text: &str, offset: usize) -> (usize, usize) {
    let mut line = 1;
    let mut column = 1;
    for (index, ch) in text.char_indices() {
        if index >= offset {
            break;
        }
        if ch == '\n' {
            line += 1;
            column = 1;
        } else {
            column += 1;
        }
    }
    (line, column)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::rules::builtin_rules;

    #[test]
    fn flags_builtin_phrase() {
        let diagnostics = analyze_text(
            Path::new("demo.md"),
            "It is important to note that this is robust and seamless.",
            &Config::default(),
            &builtin_rules().unwrap(),
        );
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.rule_id == "slop.overconfident-transition")
        );
    }

    #[test]
    fn low_confidence_rules_ask_in_interactive_mode() {
        let diagnostics = analyze_text(
            Path::new("demo.md"),
            "This is not merely a tool but also a workflow. It is not just useful but also flexible.",
            &Config::default(),
            &builtin_rules().unwrap(),
        );
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.action == "ask_user")
        );
    }

    #[test]
    fn skips_markdown_code_blocks() {
        let diagnostics = analyze_text(
            Path::new("demo.md"),
            "```text\nIt is important to note that this is robust and seamless.\n```\n",
            &Config::default(),
            &builtin_rules().unwrap(),
        );
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn matches_cjk_phrases() {
        let diagnostics = analyze_text(
            Path::new("demo.md"),
            "值得注意的是，这是一套全面、强大、高效、创新、无缝的方案。",
            &Config::default(),
            &builtin_rules().unwrap(),
        );
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.rule_id == "slop.zh-formulaic-transition")
        );
    }

    #[test]
    fn counts_cjk_characters_for_density_units() {
        assert_eq!(count_words("全面强大高效创新"), 8);
        assert_eq!(count_words("robust 工具"), 3);
    }

    #[test]
    fn cjk_density_uses_character_units_not_single_word_fallback() {
        let long_specific_text = format!(
            "{}全面强大高效创新",
            "这个段落描述具体机制和边界条件".repeat(40)
        );
        let diagnostics = analyze_text(
            Path::new("demo.md"),
            &long_specific_text,
            &Config::default(),
            &builtin_rules().unwrap(),
        );
        assert!(
            !diagnostics
                .iter()
                .any(|diagnostic| diagnostic.rule_id == "slop.zh-empty-intensifier-density")
        );
    }
}
