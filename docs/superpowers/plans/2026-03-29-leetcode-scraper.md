# LeetCode 题目爬取器实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 使用 Playwright 爬取 LeetCode 中国站前 100 题（编号 1-100）的完整题目内容，更新 JSON 并生成 markdown 文件。

**Architecture:** 创建一个独立的 Python 异步脚本 `fetch_with_playwright.py`，使用 Playwright 异步 API 并发爬取网页。先筛选出编号 1-100 的题目，跳过已有完整内容的题目，爬取完成后更新 JSON 并生成 markdown 文件。

**Tech Stack:** Python 3, Playwright (async API), asyncio, JSON。

---

## 文件结构

| 文件 | 操作 | 职责 |
|------|------|------|
| `leetcode/fetch_with_playwright.py` | 创建 | 主爬取脚本 |

---

### Task 1: 创建基础脚本框架

**Files:**
- Create: `leetcode/fetch_with_playwright.py`

- [ ] **Step 1: 创建基础脚本结构**

```python
"""
Use Playwright to scrape LeetCode problem content (problems 1-100 by ID).
"""
import asyncio
import json
import os
from typing import List, Dict, Optional
from playwright.async_api import async_playwright, Page, Browser

# Configuration
START_ID = 1
END_ID = 100
MAX_CONCURRENT = 5
RETRY_MAX = 2
RETRY_DELAY_SEC = 1
MAX_CONTENT_LENGTH_THRESHOLD = 100  # Content longer than this is considered complete

PROBLEMS_JSON = "leetcode/problems_cn.json"
OUTPUT_JSON = "leetcode/problems_cn.json"

def map_difficulty(difficulty: int) -> str:
    """Map difficulty level to Chinese name."""
    return {1: "简单", 2: "中等", 3: "困难"}.get(difficulty, "未知")

def is_numeric_id(id_str: str) -> bool:
    """Check if problem ID is numeric (skip LCP/LCR etc)."""
    try:
        int(id_str)
        return True
    except ValueError:
        return False

def get_numeric_id(id_str: str) -> int:
    """Convert ID string to integer."""
    return int(id_str)

def filter_problems(problems: List[Dict]) -> List[Dict]:
    """Filter problems: numeric ID between START_ID and END_ID, skip if content already complete."""
    filtered = []
    for p in problems:
        id_str = p["id"]
        if not is_numeric_id(id_str):
            continue
        nid = get_numeric_id(id_str)
        if nid < START_ID or nid > END_ID:
            continue
        # Check if content already exists (skip if complete)
        content = p.get("content", "")
        if len(content) > MAX_CONTENT_LENGTH_THRESHOLD and "English description" not in content:
            print(f"跳过 {id_str} {p['title']} - 已有内容")
            continue
        filtered.append(p)
    # Sort by numeric ID ascending
    filtered.sort(key=lambda x: get_numeric_id(x["id"]))
    return filtered

async def fetch_content(page: Page, slug: str) -> Optional[str]:
    """Fetch problem content from webpage."""
    url = f"https://leetcode.cn/problems/{slug}/"
    try:
        await page.goto(url, wait_until="domcontentloaded", timeout=30000)
        # Wait for content to load
        content_selectors = [".content__1Y2H", "[data-track-id=\"solution-content\"]"]
        content_element = None
        for selector in content_selectors:
            try:
                content_element = await page.wait_for_selector(selector, timeout=10000)
                if content_element:
                    break
            except asyncio.TimeoutError:
                continue
        if not content_element:
            return None
        content_html = await content_element.inner_html()
        return content_html
    except Exception as e:
        print(f"Error fetching {slug}: {e}")
        return None

async def fetch_with_retry(page: Page, slug: str) -> Optional[str]:
    """Fetch with retries."""
    for attempt in range(RETRY_MAX + 1):
        content = await fetch_content(page, slug)
        if content is not None and len(content.strip()) > 0:
            return content
        if attempt < RETRY_MAX:
            print(f"重试 {slug} (尝试 {attempt + 1}/{RETRY_MAX})")
            await asyncio.sleep(RETRY_DELAY_SEC)
    return None

def generate_markdown_file(problem: Dict) -> None:
    """Generate markdown file for a problem."""
    difficulty_cn = map_difficulty(problem["difficulty"])
    difficulty_folder = {1: "easy", 2: "medium", 3: "hard"}[problem["difficulty"]]
    filename = f"leetcode/{difficulty_folder}/{problem['id']}.{problem['slug']}.md"
    content = f"""# {problem['id']} {problem['title']}

**难度**: {difficulty_cn}

**Slug**: {problem['slug']}

**来源**: [LeetCode 链接](https://leetcode.cn/problems/{problem['slug']}/)

## 题目描述

{problem['content']}

## 解题思路



## 代码

```x
// Solution goes here
```
"""
    os.makedirs(os.path.dirname(filename), exist_ok=True)
    with open(filename, "w", encoding="utf-8") as f:
        f.write(content)

async def worker(queue: asyncio.Queue, browser: Browser, results: List[Dict], failed: List[str]) -> None:
    """Worker coroutine that processes problems from queue."""
    page = await browser.new_page()
    page.set_default_timeout(30000)
    while not queue.empty():
        problem = await queue.get()
        print(f"正在爬取: {problem['id']} {problem['title']}")
        content = await fetch_with_retry(page, problem["slug"])
        if content is not None:
            problem["content"] = content
            results.append(problem)
            generate_markdown_file(problem)
        else:
            failed.append(problem["slug"])
            print(f"爬取失败: {problem['id']} {problem['title']}")
        queue.task_done()
    await page.close()

async def main():
    """Main entry point."""
    # Read problems
    with open(PROBLEMS_JSON, "r", encoding="utf-8") as f:
        data = json.load(f)
    all_problems = data["problems"]

    # Filter problems
    target_problems = filter_problems(all_problems)
    if not target_problems:
        print("没有需要爬取的题目")
        return

    print(f"需要爬取: {len(target_problems)} 道题目")
    print(f"题目列表: {', '.join(p['id'] for p in target_problems[:10])}{'...' if len(target_problems) > 10 else ''}")

    # Create queue
    queue: asyncio.Queue = asyncio.Queue()
    for p in target_problems:
        await queue.put(p)

    results: List[Dict] = []
    failed: List[str] = []

    # Launch browser and run workers
    async with async_playwright() as p:
        browser = await p.chromium.launch(headless=True)
        tasks = []
        for _ in range(min(MAX_CONCURRENT, queue.qsize())):
            task = asyncio.create_task(worker(queue, browser, results, failed))
            tasks.append(task)
        # Wait for all workers to finish
        await queue.join()
        # Cancel worker tasks
        for t in tasks:
            t.cancel()
        await browser.close()

    # Update the original problems list
    updated_count = 0
    for result in results:
        for i, p in enumerate(all_problems):
            if p["id"] == result["id"] and p["slug"] == result["slug"]:
                all_problems[i] = result
                updated_count += 1
                break

    # Write back to JSON
    with open(OUTPUT_JSON, "w", encoding="utf-8") as f:
        json.dump({"problems": all_problems}, f, ensure_ascii=False, indent=2)

    # Summary
    print("\n" + "=" * 50)
    print(f"爬取完成!")
    print(f"成功: {len(results)} 道")
    print(f"失败: {len(failed)} 道")
    print(f"更新 JSON: {updated_count} 项")
    if failed:
        print(f"失败列表: {failed[:10]}{'...' if len(failed) > 10 else ''}")

if __name__ == "__main__":
    asyncio.run(main())
```

- [ ] **Step 2: 测试脚本语法**

```bash
cd leetcode && python -m py_compile fetch_with_playwright.py
```

Expected: 无语法错误，生成 `__pycache__/fetch_with_playwright.cpython-*.pyc`

- [ ] **Step 3: Commit**

```bash
git add leetcode/fetch_with_playwright.py
git commit -m "feat: add fetch_with_playwright.py script for leetcode scraping"
```

---

### Task 2: 安装依赖测试运行

**Files:**
- Modify: `leetcode/fetch_with_playwright.py` (if needed for fixes)

- [ ] **Step 1: Check if playwright is installed**

```bash
pip show playwright
```

If not installed:

```bash
pip install playwright
playwright install chromium
```

- [ ] **Step 2: Dry run to test script loading**

```bash
cd leetcode && python fetch_with_playwright.py
```

Expected: Script loads, filters problems, shows how many need to be scraped, then exits if all already complete.

- [ ] **Step 3: Fix any issues found during test run**

- [ ] **Step 4: Commit any fixes**

```bash
git add leetcode/fetch_with_playwright.py
git commit -m "fix: adjust script based on dry run"
```

---

### Task 3: 运行爬取

**Files:**
- Reads: `leetcode/problems_cn.json`
- Writes: `leetcode/problems_cn.json` (updated), `leetcode/{easy|medium|hard}/*.md` (generated)

- [ ] **Step 1: Run the full scrape**

```bash
cd leetcode && python fetch_with_playwright.py
```

- [ ] **Step 2: Verify output**

Check that at least 90% of problems were successfully scraped:

```bash
# Count how many markdown files were updated
ls -la leetcode/easy/ | grep -E "^[0-9]+\." | wc -l
ls -la leetcode/medium/ | grep -E "^[0-9]+\." | wc -l
ls -la leetcode/hard/ | grep -E "^[0-9]+\." | wc -l
```

Check one file content:

```bash
head -50 leetcode/easy/1.two-sum.md
```

- [ ] **Step 3: Commit changes**

```bash
git add leetcode/problems_cn.json
git add leetcode/easy/ leetcode/medium/ leetcode/hard/
git commit -m "feat: scrape 1-100 leetcode problems with playwright"
```

---

## Self-Review

1. **Spec coverage:** All requirements covered: filtering 1-100, skip complete content, async concurrent crawling, retry, update JSON, generate markdown, error handling ✓
2. **No placeholders:** All steps have exact code and exact commands ✓
3. **Consistency:** File paths, variable names are consistent throughout ✓
