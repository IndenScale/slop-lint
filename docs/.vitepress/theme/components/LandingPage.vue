<script setup lang="ts">
const props = defineProps<{
  locale: 'en' | 'zh'
}>()

const copy = {
  en: {
    kicker: 'deterministic text lint for agents',
    title: 'Catch AI slop before it lands in your repo.',
    subtitle:
      'slop-lint checks Markdown and plain text for vague, templated, inflated, or hard-to-trust writing. It is built for agent hooks, CI, and fast local review.',
    install: 'Install',
    installHref: '/slop-lint/installation',
    terminalLabel: 'slop-lint diagnostic example',
    command: 'slop-lint check draft.md',
    diagnostics: [
      {
        headline: 'docs/launch.md:3:34 warning Empty intensifiers are dense in this unit',
        matched: 'robust, seamless, comprehensive, innovative',
        rule: 'slop.empty-intensifier-density'
      },
      {
        headline: 'docs/launch.md:9:1 warning This benefit claim is generic',
        matched: 'unlock new opportunities',
        rule: 'slop.generic-benefit-claim'
      }
    ],
    bandTitle: 'One binary. Then one hook.',
    bandText:
      'Choose one install method, then attach the installed binary to your agent hooks.',
    installChoicesLabel: 'Choose one:',
    hookStepLabel: 'Then:',
    cards: [
      {
        title: 'Local density, not mere presence',
        text: 'A single phrase can be fine. Repeated contrast structures in one paragraph are different. Rules can model that difference.',
        id: 'phrase_pair_density'
      },
      {
        title: 'Data-driven rules',
        text: 'Built-in checks live in TOML. Rust handles schema, merging, loading, and execution instead of hiding judgment in opaque code.',
        id: 'rulesets/*.toml'
      },
      {
        title: 'Agent-friendly actions',
        text: 'High-confidence findings warn. Lower-confidence findings can ask the agent to confirm whether to revise or mute.',
        id: 'warn / ask_user'
      }
    ],
    hooks: [
      {
        title: 'Agent hooks',
        text: 'Claude Code, Codex, Gemini CLI, OpenCode, and Kimi Code are supported.'
      },
      {
        title: 'CI output',
        text: 'Use text, JSON, or SARIF output for local checks, scripts, and code scanning.'
      },
      {
        title: 'Project control',
        text: 'Mute rules in .slop-lint.toml or use inline disables for local exceptions.'
      }
    ]
  },
  zh: {
    kicker: '面向 agent 的确定性文本 lint',
    title: '在 AI 套话进仓库前，把它拦下来。',
    subtitle:
      'slop-lint 检查 Markdown 和纯文本里的空泛、模板化、膨胀、难以信任的表达。它为 Agent hook、CI 和本地快速审阅而设计。',
    install: '安装',
    installHref: '/slop-lint/zh/installation',
    terminalLabel: 'slop-lint 中文诊断示例',
    command: 'slop-lint check proposal.md',
    diagnostics: [
      {
        headline: 'proposal.md:1:2 warning “不是……而是……”结构在局部文本中过于密集',
        matched: '不是, 不是, 不是, 而是, 而是, 而是',
        rule: 'slop.zh-not-but-density'
      },
      {
        headline: 'proposal.md:6:1 warning 这个价值表达过于泛化，容易变成口号。',
        matched: '降本增效, 打造闭环',
        rule: 'slop.zh-generic-value'
      }
    ],
    bandTitle: '一个二进制。接一个 hook。',
    bandText: '任选一种方式安装 CLI，然后把已安装的二进制接入 Agent hook。',
    installChoicesLabel: '任选一种：',
    hookStepLabel: '然后：',
    cards: [
      {
        title: '控制局部密度，不是简单出现',
        text: '一个短文偶尔出现一次可以接受。同一段反复使用对照结构，就是另一回事。',
        id: 'phrase_pair_density'
      },
      {
        title: '规则数据化',
        text: '内置检查写在 TOML 里。Rust 负责 schema、加载、合并和执行，不把判断藏在代码里。',
        id: 'rulesets/*.toml'
      },
      {
        title: '面向 Agent 的动作',
        text: '高置信度结果直接 warn；低置信度结果可以让 Agent 先问用户要改写还是 mute。',
        id: 'warn / ask_user'
      }
    ],
    hooks: [
      {
        title: 'Agent hooks',
        text: '支持 Claude Code、Codex、Gemini CLI、OpenCode 和 Kimi Code。'
      },
      {
        title: 'CI 输出',
        text: '支持 text、JSON 和 SARIF，适合本地检查、脚本和代码扫描。'
      },
      {
        title: '项目可控',
        text: '在 .slop-lint.toml mute 规则，或用行内 disable 处理局部例外。'
      }
    ]
  }
}[props.locale]
</script>

<template>
  <div class="sl-landing" :class="props.locale === 'zh' ? 'sl-locale-zh' : 'sl-locale-en'">
    <section class="sl-hero">
      <div class="sl-hero-grid">
        <div>
          <div class="sl-kicker">{{ copy.kicker }}</div>
          <h1>{{ copy.title }}</h1>
          <p class="sl-subtitle">
            {{ copy.subtitle }}
          </p>
          <div class="sl-actions">
            <a class="sl-button primary" :href="copy.installHref">{{ copy.install }}</a>
            <a class="sl-button" href="https://github.com/IndenScale/slop-lint">GitHub</a>
            <a class="sl-button" href="https://crates.io/crates/slop-lint">crates.io</a>
          </div>
        </div>
        <div class="sl-terminal" :aria-label="copy.terminalLabel">
          <div class="sl-terminal-bar">
            <span>{{ copy.command }}</span>
            <span>v0.1 public beta</span>
          </div>
          <div class="sl-terminal-output">
            <template v-for="(diagnostic, index) in copy.diagnostics" :key="diagnostic.rule">
              <div :class="{ 'sl-gap': index > 0 }">{{ diagnostic.headline }}</div>
              <div class="sl-indent">matched: {{ diagnostic.matched }}</div>
              <div class="sl-indent">
                rule: <span class="sl-hit">{{ diagnostic.rule }}</span>
              </div>
              <div class="sl-indent">action: <span class="sl-ok">ask_user</span></div>
            </template>
          </div>
        </div>
      </div>
    </section>

    <section class="sl-section sl-band">
      <div class="sl-install">
        <div>
          <h2>{{ copy.bandTitle }}</h2>
          <p>{{ copy.bandText }}</p>
        </div>
        <div class="sl-install-commands">
          <div class="sl-step-label">{{ copy.installChoicesLabel }}</div>
          <pre><code>curl -fsSL https://raw.githubusercontent.com/IndenScale/slop-lint/main/install.sh | sh</code></pre>
          <div class="sl-or">or</div>
          <pre><code>cargo install slop-lint</code></pre>
          <div class="sl-step-label">{{ copy.hookStepLabel }}</div>
          <pre><code>slop-lint install-hooks</code></pre>
        </div>
      </div>
    </section>

    <section class="sl-section">
      <div class="sl-rule-grid">
        <article v-for="card in copy.cards" :key="card.id" class="sl-rule-card">
          <h2>{{ card.title }}</h2>
          <p>{{ card.text }}</p>
          <span class="sl-rule-id">{{ card.id }}</span>
        </article>
      </div>
    </section>

    <section class="sl-section">
      <div class="sl-hook-grid">
        <article v-for="hook in copy.hooks" :key="hook.title" class="sl-hook-card">
          <h2>{{ hook.title }}</h2>
          <p>{{ hook.text }}</p>
        </article>
      </div>
    </section>
  </div>
</template>
