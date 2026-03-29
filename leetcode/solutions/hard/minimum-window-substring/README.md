# 最小覆盖子串 (Minimum Window Substring)

## 题目信息

- **难度**: HARD
- **通过率**: 49.1%
- **标签**: `hash-table`, `string`, `sliding-window`
- **LeetCode**: https://leetcode.cn/problems/minimum-window-substring/

## 题目描述

Given two strings `s` and `t` of lengths `m` and `n` respectively, return <em>the **minimum window**</em> <span data-keyword="substring-nonempty">**<em>substring</em>**</span><em> of </em>`s`<em> such that every character in </em>`t`<em> (**including duplicates**) is included in the window</em>. If there is no such substring, return <em>the empty string </em>`&quot;&quot;`.


The testcases will be generated such that the answer is **unique**.


 

**Example 1:**

```

**Input:** s = &quot;ADOBECODEBANC&quot;, t = &quot;ABC&quot;
**Output:** &quot;BANC&quot;
**Explanation:** The minimum window substring &quot;BANC&quot; includes &#39;A&#39;, &#39;B&#39;, and &#39;C&#39; from string t.

```


**Example 2:**

```

**Input:** s = &quot;a&quot;, t = &quot;a&quot;
**Output:** &quot;a&quot;
**Explanation:** The entire string s is the minimum window.

```


**Example 3:**

```

**Input:** s = &quot;a&quot;, t = &quot;aa&quot;
**Output:** &quot;&quot;
**Explanation:** Both &#39;a&#39;s from t must be included in the window.
Since the largest window of s only has one &#39;a&#39;, return empty string.

```


 

**Constraints:**



	- `m == s.length`
	- `n == t.length`
	- `1 &lt;= m, n &lt;= 10<sup>5</sup>`
	- `s` and `t` consist of uppercase and lowercase English letters.



 

**Follow up:** Could you find an algorithm that runs in `O(m + n)` time?
