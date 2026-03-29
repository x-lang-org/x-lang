# 最后一个单词的长度 (Length of Last Word)

## 题目信息

- **难度**: EASY
- **通过率**: 48.7%
- **标签**: `string`
- **LeetCode**: https://leetcode.cn/problems/length-of-last-word/

## 题目描述

Given a string `s` consisting of words and spaces, return <em>the length of the **last** word in the string.</em>


A **word** is a maximal <span data-keyword="substring-nonempty">substring</span> consisting of non-space characters only.


 

**Example 1:**

```

**Input:** s = &quot;Hello World&quot;
**Output:** 5
**Explanation:** The last word is &quot;World&quot; with length 5.

```


**Example 2:**

```

**Input:** s = &quot;   fly me   to   the moon  &quot;
**Output:** 4
**Explanation:** The last word is &quot;moon&quot; with length 4.

```


**Example 3:**

```

**Input:** s = &quot;luffy is still joyboy&quot;
**Output:** 6
**Explanation:** The last word is &quot;joyboy&quot; with length 6.

```


 

**Constraints:**



	- `1 &lt;= s.length &lt;= 10<sup>4</sup>`
	- `s` consists of only English letters and spaces `&#39; &#39;`.
	- There will be at least one word in `s`.
