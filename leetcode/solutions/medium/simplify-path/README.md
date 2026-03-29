# 简化路径 (Simplify Path)

## 题目信息

- **难度**: MEDIUM
- **通过率**: 48.3%
- **标签**: `stack`, `string`
- **LeetCode**: https://leetcode.cn/problems/simplify-path/

## 题目描述

You are given an <em>absolute</em> path for a Unix-style file system, which always begins with a slash `&#39;/&#39;`. Your task is to transform this absolute path into its **simplified canonical path**.


The <em>rules</em> of a Unix-style file system are as follows:



	- A single period `&#39;.&#39;` represents the current directory.
	- A double period `&#39;..&#39;` represents the previous/parent directory.
	- Multiple consecutive slashes such as `&#39;//&#39;` and `&#39;///&#39;` are treated as a single slash `&#39;/&#39;`.
	- Any sequence of periods that does **not match** the rules above should be treated as a **valid directory or** **file ****name**. For example, `&#39;...&#39; `and `&#39;....&#39;` are valid directory or file names.



The simplified canonical path should follow these <em>rules</em>:



	- The path must start with a single slash `&#39;/&#39;`.
	- Directories within the path must be separated by exactly one slash `&#39;/&#39;`.
	- The path must not end with a slash `&#39;/&#39;`, unless it is the root directory.
	- The path must not have any single or double periods (`&#39;.&#39;` and `&#39;..&#39;`) used to denote current or parent directories.



Return the **simplified canonical path**.


 

**Example 1:**

<div class="example-block">

**Input:** <span class="example-io">path = &quot;/home/&quot;</span>


**Output:** <span class="example-io">&quot;/home&quot;</span>


**Explanation:**


The trailing slash should be removed.
</div>


**Example 2:**

<div class="example-block">

**Input:** <span class="example-io">path = &quot;/home//foo/&quot;</span>


**Output:** <span class="example-io">&quot;/home/foo&quot;</span>


**Explanation:**


Multiple consecutive slashes are replaced by a single one.
</div>


**Example 3:**

<div class="example-block">

**Input:** <span class="example-io">path = &quot;/home/user/Documents/../Pictures&quot;</span>


**Output:** <span class="example-io">&quot;/home/user/Pictures&quot;</span>


**Explanation:**


A double period `&quot;..&quot;` refers to the directory up a level (the parent directory).
</div>


**Example 4:**

<div class="example-block">

**Input:** <span class="example-io">path = &quot;/../&quot;</span>


**Output:** <span class="example-io">&quot;/&quot;</span>


**Explanation:**


Going one level up from the root directory is not possible.
</div>


**Example 5:**

<div class="example-block">

**Input:** <span class="example-io">path = &quot;/.../a/../b/c/../d/./&quot;</span>


**Output:** <span class="example-io">&quot;/.../b/d&quot;</span>


**Explanation:**


`&quot;...&quot;` is a valid name for a directory in this problem.
</div>


 

**Constraints:**



	- `1 &lt;= path.length &lt;= 3000`
	- `path` consists of English letters, digits, period `&#39;.&#39;`, slash `&#39;/&#39;` or `&#39;_&#39;`.
	- `path` is a valid absolute Unix path.
