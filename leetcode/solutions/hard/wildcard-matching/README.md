# 通配符匹配 (Wildcard Matching)

## 题目信息

- **难度**: HARD
- **通过率**: 34.8%
- **标签**: `greedy`, `recursion`, `string`, `dynamic-programming`
- **LeetCode**: https://leetcode.cn/problems/wildcard-matching/

## 题目描述

Given an input string (`s`) and a pattern (`p`), implement wildcard pattern matching with support for `&#39;?&#39;` and `&#39;*&#39;` where:



	- `&#39;?&#39;` Matches any single character.
	- `&#39;*&#39;` Matches any sequence of characters (including the empty sequence).



The matching should cover the **entire** input string (not partial).


 

**Example 1:**

```

**Input:** s = &quot;aa&quot;, p = &quot;a&quot;
**Output:** false
**Explanation:** &quot;a&quot; does not match the entire string &quot;aa&quot;.

```


**Example 2:**

```

**Input:** s = &quot;aa&quot;, p = &quot;*&quot;
**Output:** true
**Explanation:** &#39;*&#39; matches any sequence.

```


**Example 3:**

```

**Input:** s = &quot;cb&quot;, p = &quot;?a&quot;
**Output:** false
**Explanation:** &#39;?&#39; matches &#39;c&#39;, but the second letter is &#39;a&#39;, which does not match &#39;b&#39;.

```


 

**Constraints:**



	- `0 &lt;= s.length, p.length &lt;= 2000`
	- `s` contains only lowercase English letters.
	- `p` contains only lowercase English letters, `&#39;?&#39;` or `&#39;*&#39;`.
