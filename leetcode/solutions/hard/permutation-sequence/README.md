# 排列序列 (Permutation Sequence)

## 题目信息

- **难度**: HARD
- **通过率**: 54.7%
- **标签**: `recursion`, `math`
- **LeetCode**: https://leetcode.cn/problems/permutation-sequence/

## 题目描述

The set `[1, 2, 3, ..., n]` contains a total of `n!` unique permutations.


By listing and labeling all of the permutations in order, we get the following sequence for `n = 3`:

<ol>
	- `&quot;123&quot;`
	- `&quot;132&quot;`
	- `&quot;213&quot;`
	- `&quot;231&quot;`
	- `&quot;312&quot;`
	- `&quot;321&quot;`
</ol>


Given `n` and `k`, return the `k<sup>th</sup>` permutation sequence.


 

**Example 1:**
```
**Input:** n = 3, k = 3
**Output:** "213"

```
**Example 2:**
```
**Input:** n = 4, k = 9
**Output:** "2314"

```
**Example 3:**
```
**Input:** n = 3, k = 1
**Output:** "123"

```

 

**Constraints:**



	- `1 &lt;= n &lt;= 9`
	- `1 &lt;= k &lt;= n!`
