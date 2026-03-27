# Claude Code 插件完全指南

本文档介绍 Claude Code 的 122 个插件及其使用方法。

## 目录

- [快速入门](#快速入门)
- [插件分类概览](#插件分类概览)
- [语言服务器 (LSP)](#语言服务器-lsp)
- [开发工具](#开发工具)
- [云平台与 DevOps](#云平台与-devops)
- [项目管理与协作](#项目管理与协作)
- [数据库服务](#数据库服务)
- [通信与通知](#通信与通知)
- [安全工具](#安全工具)
- [文档与内容](#文档与内容)
- [AI 与机器学习](#ai-与机器学习)
- [支付与商务](#支付与商务)
- [其他工具](#其他工具)
- [常见问题](#常见问题)

---

## 快速入门

### 什么是 Claude Code 插件？

Claude Code 插件是扩展 Claude 能力的模块化组件，可以：
- 提供**技能 (Skills)**：特定的任务能力，如生成文档、代码审查
- 提供**MCP 服务器**：连接外部服务，如 GitHub、Slack
- 提供**LSP 服务器**：代码智能，如跳转定义、查找引用

### 如何使用插件？

插件启用后会自动加载，使用方式：

1. **自动激活**：某些插件会在特定场景自动触发
2. **斜杠命令**：输入 `/插件名` 调用特定技能
3. **自然语言**：直接描述需求，Claude 会调用合适的插件

### 插件市场来源

| 市场名称 | 插件数量 | 来源 |
|---------|---------|------|
| claude-plugins-official | 119 | Anthropic 官方 |
| anthropic-agent-skills | 3 | Anthropic Agent Skills |

---

## 插件分类概览

```
┌─────────────────────────────────────────────────────────────┐
│                    Claude Code 插件生态                      │
├─────────────┬─────────────┬─────────────┬─────────────────────┤
│   LSP 服务器  │   开发工具   │  云平台服务  │   项目管理协作      │
│   (12个)     │   (15个)    │   (18个)    │     (10个)         │
├─────────────┼─────────────┼─────────────┼─────────────────────┤
│  数据库服务   │  通信通知    │   安全工具   │   文档内容         │
│   (8个)      │   (8个)     │    (6个)    │     (12个)         │
├─────────────┼─────────────┼─────────────┼─────────────────────┤
│  AI/ML 服务  │  支付商务    │   其他工具   │   输出样式         │
│   (5个)      │   (4个)     │    (10个)   │     (4个)          │
└─────────────┴─────────────┴─────────────┴─────────────────────┘
```

---

## 语言服务器 (LSP)

语言服务器提供代码智能功能：跳转定义、查找引用、自动补全、重构等。

### 已支持的编程语言

| 插件名 | 语言 | 启动命令 |
|-------|------|---------|
| rust-analyzer-lsp | Rust | `rust-analyzer` |
| clangd-lsp | C/C++ | `clangd --background-index` |
| pyright-lsp | Python | `pyright-langserver --stdio` |
| typescript-lsp | TypeScript/JavaScript | `typescript-language-server --stdio` |
| gopls-lsp | Go | `gopls` |
| jdtls-lsp | Java | `jdtls` |
| kotlin-lsp | Kotlin | `kotlin-language-server` |
| csharp-lsp | C# | `OmniSharp` |
| swift-lsp | Swift | `sourcekit-lsp` |
| ruby-lsp | Ruby | `ruby-lsp` |
| php-lsp | PHP | `intelephense --stdio` |
| lua-lsp | Lua | `lua-language-server` |
| elixir-ls-lsp | Elixir | `elixir-ls` |

### 使用示例

```
# 让 Claude 分析代码结构
> 用 LSP 分析这个函数的所有引用

# 跳转到定义
> 帮我跳转到 Foo 类的定义位置

# 重命名符号
> 用 LSP 把这个变量名重命名为更清晰的名字
```

### 安装依赖

使用 LSP 插件前需要先安装对应语言服务器：

```bash
# Rust
rustup component add rust-analyzer

# Python
pip install pyright

# TypeScript
npm install -g typescript-language-server typescript

# Go
go install golang.org/x/tools/gopls@latest

# C/C++
# 下载 LLVM/Clangd: https://clangd.llvm.org/

# Ruby
gem install ruby-lsp

# PHP
npm install -g intelephense

# Lua
npm install -g lua-language-server
```

---

## 开发工具

### 代码审查

| 插件名 | 功能 | 使用场景 |
|-------|------|---------|
| code-review | 多代理代码审查，置信度评分过滤误报 | PR 审查、代码合并前检查 |
| coderabbit | AI 代码审查，集成 40+ 静态分析器 | 安全漏洞、代码质量检查 |
| autofix-bot | 检测安全漏洞、代码质量问题、硬编码密钥 | CI/CD 集成、自动化修复 |

```
# 使用示例
> 用 coderabbit 审查我最近的代码更改
> 运行 autofix-bot 检查安全问题
```

### 代码简化

| 插件名 | 功能 |
|-------|------|
| code-simplifier | 简化代码，提高可读性和可维护性 |
| simplify | 检查代码复用、质量和效率 |

```
> 简化这个函数，保持功能不变
> 检查这段代码是否有优化空间
```

### 开发环境

| 插件名 | 功能 |
|-------|------|
| agent-sdk-dev | Claude Agent SDK 开发工具包 |
| mcp-server-dev | MCP 服务器开发指南 |
| plugin-dev | 插件开发工具 |
| skill-creator | 创建和修改技能 |

```
> 帮我创建一个新的 MCP 服务器
> 我想开发一个自定义技能
```

### GitHub 集成

```
# GitHub 插件使用示例
> 创建一个新仓库 my-project
> 列出我的所有 PR
> 查看 PR #123 的评论
> 创建一个 issue: "修复登录页面 bug"
```

### GitLab 集成

```
# GitLab 插件使用示例
> 列出 gitlab.com 上我的项目
> 查看 MR !45 的详细信息
> 在项目中创建一个新分支
```

### 版本控制

| 插件名 | 功能 |
|-------|------|
| commit-commands | Git 提交工作流命令 |
| pr-review-toolkit | PR 审查工具集 |
| hookify | Git hooks 管理 |

```
> 帮我提交代码，生成合适的 commit message
> 创建一个 PR 到 main 分支
```

---

## 云平台与 DevOps

### AWS 服务

| 插件名 | 功能 |
|-------|------|
| aws-serverless | 设计、构建、部署、测试无服务器应用 |
| deploy-on-aws | 部署应用到 AWS |
| migration-to-aws | 迁移应用到 AWS |
| amazon-location-service | 地图、地理编码、路径规划 |

```
# AWS 示例
> 帮我设计一个无服务器架构
> 把这个应用部署到 AWS Lambda
> 添加地图功能到应用中
```

### 云平台部署

| 插件名 | 平台 | 功能 |
|-------|------|------|
| vercel | Vercel | 部署前端应用、Serverless 函数 |
| netlify-skills | Netlify | 静态站点部署、边缘函数 |
| railway | Railway | 全栈应用部署 |
| firebase | Firebase | 后端服务、数据库、托管 |
| supabase | Supabase | PostgreSQL、认证、存储 |
| neon | Neon | 无服务器 PostgreSQL |

```
# Vercel 示例
> 部署这个 Next.js 项目到 Vercel
> 查看我的 Vercel 项目列表

# Supabase 示例
> 创建一个 Supabase 表结构
> 设置 Row Level Security 策略
```

### 基础设施即代码

| 插件名 | 功能 |
|-------|------|
| terraform | HashiCorp Terraform 开发指南 |
| fastly-agent-toolkit | Fastly CDN 配置 |

```
# Terraform 示例
> 生成一个 AWS EC2 实例的 Terraform 配置
> 检查这个 Terraform 模块的最佳实践
```

### 数据工程

| 插件名 | 功能 |
|-------|------|
| data-engineering | 数据管道、ETL、数据仓库 |
| astronomer-data-agents | Apache Airflow DAG 开发 |
| data | 通用数据处理工具 |

```
# Airflow 示例
> 创建一个 Airflow DAG 从 S3 提取数据
> 调试这个失败的 DAG 任务
```

---

## 项目管理与协作

### 项目跟踪

| 插件名 | 功能 |
|-------|------|
| linear | Linear 项目管理 |
| asana | Asana 任务管理 |
| atlassian | Jira 和 Confluence 集成 |
| notion | Notion 工作空间管理 |

```
# Linear 示例
> 创建一个新任务: 实现用户登录功能
> 列出我负责的所有任务
> 更新任务状态为进行中

# Jira 示例
> 创建一个 Jira issue
> 查看 Sprint 中的所有任务
> 更新 Jira 票状态
```

### 代码搜索

| 插件名 | 功能 |
|-------|------|
| sourcegraph | 代码搜索和智能导航 |
| greptile | 代码分析和搜索 |

```
> 在所有仓库中搜索 "authenticate" 函数
> 查找这个 API 的所有调用者
```

---

## 数据库服务

| 插件名 | 数据库 | 功能 |
|-------|--------|------|
| cockroachdb | CockroachDB | 分布式 SQL 数据库 |
| planetscale | PlanetScale | 无服务器 MySQL |
| prisma | Prisma | ORM 和数据库工具 |
| pinecone | Pinecone | 向量数据库 |

```
# Prisma 示例
> 生成 Prisma schema
> 创建一个数据库迁移
> 编写 Prisma 查询

# Pinecone 示例
> 创建一个 Pinecone 索引
> 插入向量数据
> 执行相似性搜索
```

---

## 通信与通知

### 团队沟通

| 插件名 | 平台 | 功能 |
|-------|------|------|
| slack | Slack | 发送消息、管理频道 |
| discord | Discord | 服务器管理、消息发送 |
| telegram | Telegram | 机器人消息 |
| intercom | Intercom | 客户支持 |

```
# Slack 示例
> 发送消息到 #general 频道
> 列出我的 Slack 频道
> 设置一个 Slack 提醒

# Discord 示例
> 发送嵌入消息到 Discord 频道
```

### 客户数据

| 插件名 | 功能 |
|-------|------|
| zoominfo | 商业情报和公司数据 |
| circleback | 会议、邮件、日历上下文 |

---

## 安全工具

| 插件名 | 功能 | 使用场景 |
|-------|------|---------|
| aikido | SAST、密钥检测、IaC 漏洞扫描 | 代码安全审计 |
| semgrep | 静态分析，检测 bug 和安全漏洞 | CI/CD 集成 |
| nightvision | API 安全测试 | 渗透测试 |
| security-guidance | 安全开发最佳实践 | 安全编码指导 |
| optibot | 代码优化和安全建议 | 代码质量提升 |
| opsera-devsecops | DevSecOps 工具链 | 安全运维 |

```
# 安全扫描示例
> 扫描这个仓库的安全漏洞
> 检查代码中是否有硬编码密钥
> 审查这个 API 的安全性
```

---

## 文档与内容

### 文档处理

| 插件名 | 功能 |
|-------|------|
| document-skills | Excel、Word、PowerPoint、PDF 处理 |
| mintlify | 文档生成和发布 |
| microsoft-docs | Microsoft 技术文档查询 |
| context7 | 实时文档查询 |

```
# 文档处理示例
> 创建一个 Excel 报表
> 生成一个 PowerPoint 演示文稿
> 提取 PDF 中的表格数据
```

### 内容管理

| 插件名 | 平台 | 功能 |
|-------|------|------|
| sanity-plugin | Sanity | 无头 CMS 内容管理 |
| wix | Wix | 网站构建 |
| wordpress.com | WordPress | 博客和网站管理 |
| legalzoom | LegalZoom | 法律文档 |

```
# WordPress 示例
> 创建一篇新博客文章
> 更新网站页面内容
```

### 设计工具

| 插件名 | 功能 |
|-------|------|
| figma | Figma 设计工具集成 |
| frontend-design | 前端界面设计 |
| cloudinary | 图片和视频管理 |

```
# Figma 示例
> 获取 Figma 设计稿
> 导出设计资源
> 生成设计对应的 CSS
```

---

## AI 与机器学习

| 插件名 | 功能 |
|-------|------|
| claude-api | Claude API 开发指南 |
| huggingface-skills | Hugging Face 模型使用 |
| fiftyone | 计算机视觉数据集管理 |
| goodmem | AI 记忆和上下文管理 |
| superpowers | AI 能力增强 |

```
# Claude API 示例
> 帮我写一个调用 Claude API 的 Python 脚本
> 解释 Claude API 的流式响应

# Hugging Face 示例
> 使用 transformers 库加载 BERT 模型
> 生成文本嵌入向量
```

---

## 支付与商务

| 插件名 | 功能 |
|-------|------|
| stripe | Stripe 支付集成 |
| sumup | SumUp 支付处理 |
| revenuecat | 订阅和内购管理 |
| posthog | 产品分析和用户追踪 |

```
# Stripe 示例
> 创建一个 Stripe 结账会话
> 处理 Webhook 事件
> 查询订阅状态
```

---

## 其他工具

### 测试与自动化

| 插件名 | 功能 |
|-------|------|
| playwright | 浏览器自动化测试 |
| stagehand | AI 驱动的网页操作 |
| chrome-devtools-mcp | Chrome 开发者工具控制 |
| postman | API 测试和文档 |

```
# Playwright 示例
> 编写一个登录流程的端到端测试
> 截取页面截图
> 测试响应式布局

# Chrome DevTools 示例
> 记录页面性能追踪
> 分析网络请求
> 检查控制台错误
```

### 监控与日志

| 插件名 | 功能 |
|-------|------|
| sentry | 错误监控和性能追踪 |
| pagerduty | 事件管理和告警 |
| posthog | 产品分析 |

```
# Sentry 示例
> 查看最近的错误报告
> 分析这个异常的堆栈跟踪
> 设置错误告警规则
```

### 网络与爬虫

| 插件名 | 功能 |
|-------|------|
| firecrawl | 网页爬取和数据提取 |
| brightdata-plugin | 网页抓取、Google 搜索、结构化数据 |
| nimble | 网页数据采集 |

```
# 爬虫示例
> 爬取这个网页的内容
> 提取页面中的所有链接
> 监控页面变化
```

### 社交媒体

| 插件名 | 功能 |
|-------|------|
| postiz | 社交媒体内容管理 |
| zapier | 自动化工作流 |
| followrabbit | 内容跟踪 |

### 搜索与发现

| 插件名 | 功能 |
|-------|------|
| adspirer-ads-agent | 跨平台广告管理 (Google/Meta/TikTok/LinkedIn) |
| searchfit-seo | SEO 优化工具 |

### 效率工具

| 插件名 | 功能 |
|-------|------|
| remember | 记忆和上下文保持 |
| feature-dev | 功能开发助手 |
| playground | 代码实验环境 |
| laravel-boost | Laravel 开发加速 |
| ai-firstify | AI-first 项目审计 |

---

## 输出样式

这些插件改变 Claude 的输出风格：

| 插件名 | 风格 |
|-------|------|
| explanatory-output-style | 详细解释，适合学习 |
| learning-output-style | 教学风格，循序渐进 |
| fakechat | 模拟聊天界面 |
| rc | 自定义输出格式 |

---

## 常见问题

### Q: 插件不生效怎么办？

1. 确认插件已在 `settings.json` 中启用
2. 重启 Claude Code
3. 检查插件依赖是否已安装（特别是 LSP 服务器）

### Q: 如何禁用某个插件？

编辑 `~/.claude/settings.json`，将对应插件设为 `false`：

```json
{
  "enabledPlugins": {
    "plugin-name@marketplace": false
  }
}
```

### Q: LSP 插件需要什么依赖？

每个 LSP 插件需要安装对应语言服务器，详见 [语言服务器 (LSP)](#语言服务器-lsp) 章节。

### Q: 如何查看已启用的插件？

```bash
# 查看配置文件
cat ~/.claude/settings.json | grep "true"
```

### Q: 插件会自动更新吗？

是的，大部分插件会自动更新。你可以在 `settings.json` 中设置：

```json
{
  "extraKnownMarketplaces": {
    "anthropic-agent-skills": {
      "autoUpdate": true
    }
  }
}
```

---

## 附录：完整插件列表

### anthropic-agent-skills 市场 (3个)

1. document-skills - Excel、Word、PowerPoint、PDF 处理
2. example-skills - 示例技能集合（算法艺术、品牌指南、画布设计等）
3. claude-api - Claude API 开发文档

### claude-plugins-official 市场 (119个)

详见上文各分类章节。

---

*文档版本: 2026-03-27*
*插件总数: 122*
