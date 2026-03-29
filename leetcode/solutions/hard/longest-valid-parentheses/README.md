# 最长有效括号 (Longest Valid Parentheses)

## 题目信息

- **难度**: HARD
- **通过率**: 41.7%
- **标签**: `stack`, `string`, `dynamic-programming`
- **LeetCode**: https://leetcode.cn/problems/longest-valid-parentheses/

## 题目描述

Given a string containing just the characters `&#39;(&#39;` and `&#39;)&#39;`, return <em>the length of the longest valid (well-formed) parentheses </em><span data-keyword="substring-nonempty"><em>substring</em></span>.


 

**Example 1:**

```

**Input:** s = &quot;(()&quot;
**Output:** 2
**Explanation:** The longest valid parentheses substring is &quot;()&quot;.

```


**Example 2:**

```

**Input:** s = &quot;)()())&quot;
**Output:** 4
**Explanation:** The longest valid parentheses substring is &quot;()()&quot;.

```


**Example 3:**

```

**Input:** s = &quot;&quot;
**Output:** 0

```


 

**Constraints:**



	- `0 &lt;= s.length &lt;= 3 * 10<sup>4</sup>`
	- `s[i]` is `&#39;(&#39;`, or `&#39;)&#39;`.
