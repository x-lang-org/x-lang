# 组合总和 II (Combination Sum II)

## 题目信息

- **难度**: MEDIUM
- **通过率**: 60.1%
- **标签**: `array`, `backtracking`
- **LeetCode**: https://leetcode.cn/problems/combination-sum-ii/

## 题目描述

Given a collection of candidate numbers (`candidates`) and a target number (`target`), find all unique combinations in `candidates` where the candidate numbers sum to `target`.


Each number in `candidates` may only be used **once** in the combination.


**Note:** The solution set must not contain duplicate combinations.


 

**Example 1:**

```

**Input:** candidates = [10,1,2,7,6,1,5], target = 8
**Output:** 
[
[1,1,6],
[1,2,5],
[1,7],
[2,6]
]

```


**Example 2:**

```

**Input:** candidates = [2,5,2,1,2], target = 5
**Output:** 
[
[1,2,2],
[5]
]

```


 

**Constraints:**



	- `1 &lt;= candidates.length &lt;= 100`
	- `1 &lt;= candidates[i] &lt;= 50`
	- `1 &lt;= target &lt;= 30`
