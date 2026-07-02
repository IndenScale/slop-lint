import { defineConfig } from 'vitepress'

const base = '/slop-lint/'
const faviconHref = `${base}favicon.svg?v=20260702`

export default defineConfig({
  title: 'slop-lint',
  description: 'Deterministic lint for agent-written text.',
  base,
  cleanUrls: true,
  locales: {
    root: {
      label: 'English',
      lang: 'en-US'
    },
    zh: {
      label: '简体中文',
      lang: 'zh-CN',
      title: 'slop-lint',
      description: '面向 Agent 文本产物的确定性 lint 工具。',
      themeConfig: {
        nav: [
          { text: '安装', link: '/zh/installation' },
          { text: '规则', link: '/zh/rules' },
          { text: 'Hooks', link: '/zh/agent-hooks' },
          { text: '配置', link: '/zh/configuration' },
          { text: '路线图', link: '/zh/roadmap' }
        ],
        sidebar: [
          {
            text: '指南',
            items: [
              { text: '安装', link: '/zh/installation' },
              { text: '规则', link: '/zh/rules' },
              { text: 'Agent Hooks', link: '/zh/agent-hooks' },
              { text: '配置', link: '/zh/configuration' },
              { text: '路线图', link: '/zh/roadmap' }
            ]
          }
        ],
        langMenuLabel: '切换语言',
        returnToTopLabel: '返回顶部',
        sidebarMenuLabel: '菜单',
        darkModeSwitchLabel: '外观',
        lightModeSwitchTitle: '切换到浅色模式',
        darkModeSwitchTitle: '切换到深色模式',
        footer: {
          message:
            '欢迎 <a href="https://github.com/IndenScale/slop-lint/issues">贡献规则</a> 或访问 <a href="https://indenscale.github.io/">IndenScale 个人主页</a>。',
          copyright: 'Released under the MIT License.'
        }
      }
    }
  },
  head: [
    ['link', { rel: 'icon', href: faviconHref, type: 'image/svg+xml' }],
    ['link', { rel: 'shortcut icon', href: faviconHref, type: 'image/svg+xml' }],
    ['meta', { name: 'theme-color', content: '#101820' }],
    ['meta', { property: 'og:title', content: 'slop-lint' }],
    [
      'meta',
      {
        property: 'og:description',
        content: 'Deterministic lint for vague, templated, or inflated agent-written text.'
      }
    ]
  ],
  themeConfig: {
    i18nRouting: true,
    logo: { src: '/favicon.svg', alt: 'slop-lint' },
    nav: [
      { text: 'Install', link: '/installation' },
      { text: 'Rules', link: '/rules' },
      { text: 'Hooks', link: '/agent-hooks' },
      { text: 'Config', link: '/configuration' },
      { text: 'Roadmap', link: '/roadmap' }
    ],
    sidebar: [
      {
        text: 'Guide',
        items: [
          { text: 'Install', link: '/installation' },
          { text: 'Rules', link: '/rules' },
          { text: 'Agent Hooks', link: '/agent-hooks' },
          { text: 'Configuration', link: '/configuration' },
          { text: 'Roadmap', link: '/roadmap' }
        ]
      }
    ],
    socialLinks: [
      { icon: 'github', link: 'https://github.com/IndenScale/slop-lint' }
    ],
    search: {
      provider: 'local'
    },
    footer: {
      message:
        'Help tune the rulebook: <a href="https://github.com/IndenScale/slop-lint/issues">suggest a rule</a> or visit <a href="https://indenscale.github.io/">IndenScale</a>.',
      copyright: 'Released under the MIT License.'
    }
  }
})
