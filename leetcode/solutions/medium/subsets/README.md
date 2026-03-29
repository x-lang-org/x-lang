# 子集 (Subsets)

## 题目信息

- **难度**: MEDIUM
- **通过率**: 82.0%
- **标签**: `bit-manipulation`, `array`, `backtracking`
- **LeetCode**: https://leetcode.cn/problems/subsets/

## 题目描述

Given an integer array `nums` of **unique** elements, return <em>all possible</em> <span data-keyword="subset"><em>subsets</em></span> <em>(the power set)</em>.


The solution set **must not** contain duplicate subsets. Return the solution in **any order**.


 

**Example 1:**

```

**Input:** nums = [1,2,3]
**Output:** [[],[1],[2],[1,2],[3],[1,3],[2,3],[1,2,3]]

```


**Example 2:**

```

**Input:** nums = [0]
**Output:** [[],[0]]

```


 

**Constraints:**



	- `1 &lt;= nums.length &lt;= 10`
	- `-10 &lt;= nums[i] &lt;= 10`
	- All the numbers of `nums` are **unique**.
