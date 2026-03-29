# 正则表达式匹配 (Regular Expression Matching)

## 题目信息

- **难度**: HARD
- **通过率**: 31.2%
- **标签**: `recursion`, `string`, `dynamic-programming`
- **LeetCode**: https://leetcode.cn/problems/regular-expression-matching/

## 题目描述

Given an input string `s` and a pattern `p`, implement regular expression matching with support for `&#39;.&#39;` and `&#39;*&#39;` where:



	- `&#39;.&#39;` Matches any single character.​​​​
	- `&#39;*&#39;` Matches zero or more of the preceding element.



Return a boolean indicating whether the matching covers the entire input string (not partial).


 

**Example 1:**

```

**Input:** s = &quot;aa&quot;, p = &quot;a&quot;
**Output:** false
**Explanation:** &quot;a&quot; does not match the entire string &quot;aa&quot;.

```


**Example 2:**

```

**Input:** s = &quot;aa&quot;, p = &quot;a*&quot;
**Output:** true
**Explanation:** &#39;*&#39; means zero or more of the preceding element, &#39;a&#39;. Therefore, by repeating &#39;a&#39; once, it becomes &quot;aa&quot;.

```


**Example 3:**

```

**Input:** s = &quot;ab&quot;, p = &quot;.*&quot;
**Output:** true
**Explanation:** &quot;.*&quot; means &quot;zero or more (*) of any character (.)&quot;.

```


 

**Constraints:**



	- `1 &lt;= s.length &lt;= 20`
	- `1 &lt;= p.length &lt;= 20`
	- `s` contains only lowercase English letters.
	- `p` contains only lowercase English letters, `&#39;.&#39;`, and `&#39;*&#39;`.
	- It is guaranteed for each appearance of the character `&#39;*&#39;`, there will be a previous valid character to match.
