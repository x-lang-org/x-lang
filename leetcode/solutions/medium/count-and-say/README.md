# 外观数列 (Count and Say)

## 题目信息

- **难度**: MEDIUM
- **通过率**: 61.7%
- **标签**: `string`
- **LeetCode**: https://leetcode.cn/problems/count-and-say/

## 题目描述

The **count-and-say** sequence is a sequence of digit strings defined by the recursive formula:



	- `countAndSay(1) = &quot;1&quot;`
	- `countAndSay(n)` is the run-length encoding of `countAndSay(n - 1)`.



<a href="http://en.wikipedia.org/wiki/Run-length_encoding" target="_blank">Run-length encoding</a> (RLE) is a string compression method that works by replacing consecutive identical characters (repeated 2 or more times) with the concatenation of the character and the number marking the count of the characters (length of the run). For example, to compress the string `&quot;3322251&quot;` we replace `&quot;33&quot;` with `&quot;23&quot;`, replace `&quot;222&quot;` with `&quot;32&quot;`, replace `&quot;5&quot;` with `&quot;15&quot;` and replace `&quot;1&quot;` with `&quot;11&quot;`. Thus the compressed string becomes `&quot;23321511&quot;`.


Given a positive integer `n`, return <em>the </em>`n<sup>th</sup>`<em> element of the **count-and-say** sequence</em>.


 

**Example 1:**

<div class="example-block">

**Input:** <span class="example-io">n = 4</span>


**Output:** <span class="example-io">&quot;1211&quot;</span>


**Explanation:**

```

countAndSay(1) = &quot;1&quot;
countAndSay(2) = RLE of &quot;1&quot; = &quot;11&quot;
countAndSay(3) = RLE of &quot;11&quot; = &quot;21&quot;
countAndSay(4) = RLE of &quot;21&quot; = &quot;1211&quot;

```
</div>


**Example 2:**

<div class="example-block">

**Input:** <span class="example-io">n = 1</span>


**Output:** <span class="example-io">&quot;1&quot;</span>


**Explanation:**


This is the base case.
</div>


 

**Constraints:**



	- `1 &lt;= n &lt;= 30`



 
**Follow up:** Could you solve it iteratively?
