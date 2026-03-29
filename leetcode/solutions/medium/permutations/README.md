# 全排列 (Permutations)

## 题目信息

- **难度**: MEDIUM
- **通过率**: 80.3%
- **标签**: `array`, `backtracking`
- **LeetCode**: https://leetcode.cn/problems/permutations/

## 题目描述

Given an array `nums` of distinct integers, return all the possible <span data-keyword="permutation-array">permutations</span>. You can return the answer in **any order**.


 

**Example 1:**
```
**Input:** nums = [1,2,3]
**Output:** [[1,2,3],[1,3,2],[2,1,3],[2,3,1],[3,1,2],[3,2,1]]

```
**Example 2:**
```
**Input:** nums = [0,1]
**Output:** [[0,1],[1,0]]

```
**Example 3:**
```
**Input:** nums = [1]
**Output:** [[1]]

```

 

**Constraints:**



	- `1 &lt;= nums.length &lt;= 6`
	- `-10 &lt;= nums[i] &lt;= 10`
	- All the integers of `nums` are **unique**.
