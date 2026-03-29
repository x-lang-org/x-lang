# 电话号码的字母组合 (Letter Combinations of a Phone Number)

## 题目信息

- **难度**: MEDIUM
- **通过率**: 63.4%
- **标签**: `hash-table`, `string`, `backtracking`
- **LeetCode**: https://leetcode.cn/problems/letter-combinations-of-a-phone-number/

## 题目描述

Given a string containing digits from `2-9` inclusive, return all possible letter combinations that the number could represent. Return the answer in **any order**.


A mapping of digits to letters (just like on the telephone buttons) is given below. Note that 1 does not map to any letters.
<img alt="" src="https://assets.leetcode.com/uploads/2022/03/15/1200px-telephone-keypad2svg.png" style="width: 300px; height: 243px;" />

 

**Example 1:**

```

**Input:** digits = &quot;23&quot;
**Output:** [&quot;ad&quot;,&quot;ae&quot;,&quot;af&quot;,&quot;bd&quot;,&quot;be&quot;,&quot;bf&quot;,&quot;cd&quot;,&quot;ce&quot;,&quot;cf&quot;]

```


**Example 2:**

```

**Input:** digits = &quot;2&quot;
**Output:** [&quot;a&quot;,&quot;b&quot;,&quot;c&quot;]

```


 

**Constraints:**



	- `1 &lt;= digits.length &lt;= 4`
	- `digits[i]` is a digit in the range `[&#39;2&#39;, &#39;9&#39;]`.
