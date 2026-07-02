# 规则

`slop-lint` 使用确定性的 TOML 规则。内置规则位于 `rulesets/*.toml`，
项目可以在 `.slop-lint.toml` 中添加或覆盖规则。

当前规则类型：

- `phrase_presence`：文本单元包含指定短语时提示。
- `phrase_density`：短语密度超过阈值时提示。
- `phrase_pair_density`：成对结构在局部文本中过密时提示。

中文“不是……而是……”规则示例：

```toml
[[rules]]
id = "slop.zh-not-but-density"
name = "中文“不是……而是……”结构密度过高"
level = "warning"
kind = "phrase_pair_density"
first_phrases = ["不是"]
second_phrases = ["而是"]
threshold_per_1000_words = 24.0
min_occurrences = 3
message = "“不是……而是……”结构在局部文本中过于密集，容易形成模板化对照。"
suggestion = "保留真正有辨析价值的一处对照，其余改成直接判断、证据或边界条件。"
confidence = 0.74
```

这个规则关注局部密度：短文出现一次不报，同一段反复三次会报，长文里稀疏出现四五次不报。
