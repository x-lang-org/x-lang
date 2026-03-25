# Claude Code 官方插件目录

本文档整理了 Anthropic 官方提供的 Claude Code 插件和技能。

**参考链接：**
- [Claude Code 插件文档](https://code.claude.com/docs/en/plugins)
- [claude-plugins-official 仓库](https://github.com/anthropics/claude-plugins-official)
- [skills 仓库](https://github.com/anthropics/skills)

---

## Internal Plugins

来自 `anthropics/claude-plugins-official` 仓库，由 Anthropic 团队开发和维护。

| 插件名 | 描述 |
|--------|------|
| agent-sdk-dev | Claude Agent SDK 开发工具 |
| claude-code-setup | 分析代码库并推荐 Claude Code 自动化配置 |
| claude-md-management | CLAUDE.md 文件维护和质量审计工具 |
| code-review | 使用多代理进行自动化 PR 代码审查 |
| code-simplifier | 简化和重构代码 |
| commit-commands | 简化 git 提交工作流 |
| feature-dev | 功能开发工作流，含专门代理探索代码库 |
| frontend-design | 前端 UI/UX 设计实现 |
| hookify | 分析对话模式创建 hooks 防止不良行为 |
| mcp-server-dev | MCP 服务器开发技能 |
| playground | 创建交互式 HTML 演示环境 |
| plugin-dev | 插件开发工具包 |
| pr-review-toolkit | PR 审查代理工具 |
| ralph-loop | 自引用 AI 循环进行迭代开发 |
| security-guidance | 安全提醒 hook，编辑文件时警告潜在问题 |
| skill-creator | 创建新技能 |
| math-olympiad | 解决数学竞赛问题 (IMO级别) |
| explanatory-output-style | 教育性输出风格，解释实现选择 |
| learning-output-style | 交互式学习模式 |

---

## LSP Plugins

语言服务器协议 (Language Server Protocol) 插件，提供代码智能功能。

| 插件名 | 语言 |
|--------|------|
| clangd-lsp | C/C++ |
| csharp-lsp | C# |
| gopls-lsp | Go |
| jdtls-lsp | Java |
| kotlin-lsp | Kotlin |
| lua-lsp | Lua |
| php-lsp | PHP |
| pyright-lsp | Python |
| ruby-lsp | Ruby |
| rust-analyzer-lsp | Rust |
| swift-lsp | Swift |
| typescript-lsp | TypeScript |

---

## Document Skills

来自 `anthropics/skills` 仓库，提供文档处理和创作能力。

| 技能名 | 描述 |
|--------|------|
| algorithmic-art | p5.js 算法艺术创作 |
| brand-guidelines | Anthropic 品牌设计规范 |
| canvas-design | 海报/视觉设计创作 |
| claude-api | Claude API 应用开发 |
| doc-coauthoring | 协作文档撰写 |
| docx | Word 文档处理 |
| frontend-design | 高质量前端界面 |
| internal-comms | 内部沟通文档写作 |
| mcp-builder | MCP 服务器构建 |
| pdf | PDF 文件处理 |
| pptx | PowerPoint 演示文稿 |
| slack-gif-creator | Slack GIF 动画创建 |
| theme-factory | 主题样式工具 |
| web-artifacts-builder | React/Tailwind Web 应用构建 |
| webapp-testing | Playwright Web 测试 |
| xlsx | Excel 电子表格处理 |

---

## External Plugins

来自第三方和合作伙伴的插件。

| 插件名 | 描述 | 提供商 |
|--------|------|--------|
| github | GitHub 官方 MCP 服务器 | GitHub |
| gitlab | GitLab DevOps 集成 | GitLab |
| playwright | Microsoft 浏览器自动化测试 | Microsoft |
| supabase | Supabase 数据库操作 | Supabase |
| firebase | Google Firebase 集成 | Google |
| linear | Linear 任务跟踪 | Linear |
| asana | Asana 项目管理 | Asana |
| slack | Slack 工作区集成 | Slack |
| discord | Discord 频道集成 | Discord |
| telegram | Telegram 频道集成 | Telegram |
| imessage | iMessage 集成 | Apple |
| context7 | Upstash 文档查找 | Upstash |
| greptile | AI 代码审查代理 | Greptile |
| serena | 语义代码分析 | 社区 |
| laravel-boost | Laravel 开发工具包 | 社区 |

---

## 安装指南

### 添加官方市场

```bash
# 添加 Anthropic 插件市场
/plugin marketplace add anthropics/claude-plugins-official

# 添加 Anthropic 技能市场
/plugin marketplace add anthropics/skills
```

### 安装插件

```bash
# 安装插件
/plugin install <插件名>@claude-plugins-official

# 示例
/plugin install code-review@claude-plugins-official
/plugin install mcp-server-dev@claude-plugins-official
/plugin install rust-analyzer-lsp@claude-plugins-official
```

### 管理插件

```bash
# 列出已安装插件
/plugin list

# 启用/禁用插件
/plugin enable <插件名>
/plugin disable <插件名>

# 更新插件
/plugin update <插件名>
```

---

## 统计信息

| 类别 | 数量 |
|------|------|
| Internal Plugins | 19 |
| LSP Plugins | 12 |
| Document Skills | 16 |
| External Plugins | 15 |
| **总计** | **62** |
