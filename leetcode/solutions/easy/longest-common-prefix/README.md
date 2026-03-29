# 最长公共前缀 (Longest Common Prefix)

## 题目信息

- **难度**: EASY
- **通过率**: 45.1%
- **标签**: `trie`, `array`, `string`
- **LeetCode**: https://leetcode.cn/problems/longest-common-prefix/

## 题目描述

Write a function to find the longest common prefix string amongst an array of strings.


If there is no common prefix, return an empty string `&quot;&quot;`.


 

**Example 1:**

```

**Input:** strs = [&quot;flower&quot;,&quot;flow&quot;,&quot;flight&quot;]
**Output:** &quot;fl&quot;

```


**Example 2:**

```

**Input:** strs = [&quot;dog&quot;,&quot;racecar&quot;,&quot;car&quot;]
**Output:** &quot;&quot;
**Explanation:** There is no common prefix among the input strings.

```


 

**Constraints:**



	- `1 &lt;= strs.length &lt;= 200`
	- `0 &lt;= strs[i].length &lt;= 200`
	- `strs[i]` consists of only lowercase English letters if it is non-empty.
