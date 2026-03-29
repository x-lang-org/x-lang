# LeetCode 题目爬取器设计

## 概述

使用 Playwright 爬取 LeetCode 中国站题目内容，按题目编号从小到大爬取前 100 题，更新 JSON 数据并生成完整的 markdown 文件。

## 背景

现有的 `fetch_cn_fast.py` 使用 GraphQL API 获取题目内容，但许多题目返回的是占位符 `"English description is not available..."`，缺少实际题目内容。本项目通过爬取网页解决这个问题。

## 需求

- 爬取范围：题目编号 **1 ~ 100**（按数字编号排序，跳过 LCP/LCR 等特殊编号）
- 输出：
  1. 更新 `leetcode/problems_cn.json` 中对应题目的 `content` 字段
  2. 在 `leetcode/{easy|medium|hard}/` 目录生成包含完整题目的 markdown 文件
- 去重：不重复爬取已有完整内容的题目
- 限速：控制并发请求避免被限流

## 架构设计

```
leetcode/
├── fetch_with_playwright.py     # 新增：主爬取脚本
├── problems.json                # 原始题目列表
├── problems_cn.json             # 输入输出：带 content 的题目数据
├── easy/                        # 输出：简单题 markdown
├── medium/                      # 输出：中等题 markdown
└── hard/                        # 输出：困难题 markdown
```

## 核心流程

1. **读取输入**：加载 `problems_cn.json` 中的所有题目
2. **筛选题目**：
   - 提取可转换为整数编号的题目
   - 按编号从小到大排序
   - 选取编号 1 ~ 100
   - 跳过 content 已是完整内容（长度 > 100）的题目
3. **并发爬取**：
   - 使用 `asyncio` + `playwright.async_api` 异步并发
   - 控制并发数 `max_concurrent = 5`
   - 每个题目：
     - 导航到 `https://leetcode.cn/problems/<slug>/`
     - 等待题目内容元素加载（`[data-track-id="solution-content"]` 或 `.content__1Y2H`）
     - 提取 HTML 内容
     - 添加重试机制（失败重试 2 次）
4. **保存结果**：
   - 更新内存中题目对象的 `content` 字段
   - 全部完成后写回 `problems_cn.json`
   - 为每个题目生成 markdown 文件

## 数据提取

选择器（优先尝试第一个，失败尝试第二个）：

| 选择器 | 位置 | 说明 |
|--------|------|------|
| `.content__1Y2H` | 题目描述容器 | 当前网页结构 |
| `[data-track-id="solution-content"]` | 备选 | 旧结构 |

提取：`element.inner_html()` 直接得到 HTML 格式的题目内容。

## markdown 文件格式

```markdown
# {id} {title}

**难度**: {难度中文}

**Slug**: {slug}

**来源**: [LeetCode 链接](https://leetcode.cn/problems/{slug}/)

## 题目描述

{content}

## 解题思路


## 代码

```x
// Solution goes here
```
```

## 错误处理与重试

- 单个题目爬取失败：记录失败列表，继续爬取其他题目
- 重试：每个题目最多重试 2 次，间隔 1 秒
- 限流检测：如果连续多个题目失败，自动降低并发数并暂停 5 秒

## 依赖

- Python 3.8+
- `playwright` package：`pip install playwright`
- 浏览器：`playwright install chromium`（只需安装一次）

## 成功标准

- 前 100 题中有完整内容的题目数 ≥ 90
- `problems_cn.json` 更新正确
- markdown 文件生成正确
- 脚本可重复运行（重复运行不会破坏已有数据）
