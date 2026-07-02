# Rules

`slop-lint` uses deterministic TOML-backed rules. Built-in rules live in
`rulesets/*.toml`, and project-specific rules can be added in `.slop-lint.toml`.

The current rule kinds are:

- `phrase_presence`: warn when a unit contains configured phrases.
- `phrase_density`: warn when phrase density crosses a threshold.
- `phrase_pair_density`: warn when a paired construction is locally dense.

Example:

```toml
[[rules]]
id = "brand.not-just-but-density"
name = "Repeated not-just-but contrast"
level = "warning"
kind = "phrase_pair_density"
first_phrases = ["not just", "not merely"]
second_phrases = ["but also"]
threshold_per_1000_words = 18.0
min_occurrences = 3
message = "This contrast pattern is locally dense and may read as templated framing."
suggestion = "Keep the contrast only where it changes the argument; rewrite the rest as direct claims."
confidence = 0.74
```

The density rule is intentionally local: one occurrence in a short note should
not warn, while repeated patterns in a paragraph should.
