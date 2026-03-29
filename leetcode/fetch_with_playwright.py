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

# When running from leetcode directory, use local paths
PROBLEMS_JSON = "problems_cn.json"
OUTPUT_JSON = "problems_cn.json"

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
    filename = f"{difficulty_folder}/{problem['id']}.{problem['slug']}.md"
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
