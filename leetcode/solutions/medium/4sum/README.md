# 四数之和 (4Sum)

## 题目信息

- **难度**: MEDIUM
- **通过率**: 36.9%
- **标签**: `array`, `two-pointers`, `sorting`
- **LeetCode**: https://leetcode.cn/problems/4sum/

## 题目描述

Given an array `nums` of `n` integers, return <em>an array of all the **unique** quadruplets</em> `[nums[a], nums[b], nums[c], nums[d]]` such that:



	- `0 &lt;= a, b, c, d &lt; n`
	- `a`, `b`, `c`, and `d` are **distinct**.
	- `nums[a] + nums[b] + nums[c] + nums[d] == target`



You may return the answer in **any order**.


 

**Example 1:**

```

**Input:** nums = [1,0,-1,0,-2,2], target = 0
**Output:** [[-2,-1,1,2],[-2,0,0,2],[-1,0,0,1]]

```


**Example 2:**

```

**Input:** nums = [2,2,2,2,2], target = 8
**Output:** [[2,2,2,2]]

```


 

**Constraints:**



	- `1 &lt;= nums.length &lt;= 200`
	- `-10<sup>9</sup> &lt;= nums[i] &lt;= 10<sup>9</sup>`
	- `-10<sup>9</sup> &lt;= target &lt;= 10<sup>9</sup>`
