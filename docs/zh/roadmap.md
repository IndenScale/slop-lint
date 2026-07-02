# 路线图

0.1 系列是面向 Agent 高频写作工作流的 public beta。

近期工作：

- 使用真实中英文 Agent 文档校准成对结构密度规则。
- 为 Claude Code、Codex、Gemini CLI、OpenCode 和 Kimi Code 的真实 hook
  event payload 增加 fixture 兼容性测试。
- 增加针对安装脚本和 `cargo install` 的发布 smoke 检查。
- 发布面向学术写作、技术文档、社交媒体和品牌语气的外部规则包。
- 增加 PostToolUse hook 的 diff-only 模式。

长期方向：

- 第一阶段优先支持 data-only 插件包。
- 只有当规则逻辑确实需要代码时，再考虑 WASM 插件。
