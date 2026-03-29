# 无重复字符的最长子串 (Longest Substring Without Repeating Characters)

## 题目信息

- **难度**: MEDIUM
- **通过率**: 42.3%
- **标签**: `hash-table`, `string`, `sliding-window`
- **LeetCode**: https://leetcode.cn/problems/longest-substring-without-repeating-characters/

## 题目描述

Given a string `s`, find the length of the **longest** <span data-keyword="substring-nonempty">**substring**</span> without duplicate characters.


 

**Example 1:**

```

**Input:** s = &quot;abcabcbb&quot;
**Output:** 3
**Explanation:** The answer is &quot;abc&quot;, with the length of 3. Note that `&quot;bca&quot;` and `&quot;cab&quot;` are also correct answers.

```


**Example 2:**

```

**Input:** s = &quot;bbbbb&quot;
**Output:** 1
**Explanation:** The answer is &quot;b&quot;, with the length of 1.

```


**Example 3:**

```

**Input:** s = &quot;pwwkew&quot;
**Output:** 3
**Explanation:** The answer is &quot;wke&quot;, with the length of 3.
Notice that the answer must be a substring, &quot;pwke&quot; is a subsequence and not a substring.

```


 

**Constraints:**



	- `0 &lt;= s.length &lt;= 5 * 10<sup>4</sup>`
	- `s` consists of English letters, digits, symbols and spaces.
