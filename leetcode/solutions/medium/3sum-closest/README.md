# 最接近的三数之和 (3Sum Closest)

## 题目信息

- **难度**: MEDIUM
- **通过率**: 44.8%
- **标签**: `array`, `two-pointers`, `sorting`
- **LeetCode**: https://leetcode.cn/problems/3sum-closest/

## 题目描述

Given an integer array `nums` of length `n` and an integer `target`, find three integers at **distinct indices** in `nums` such that the sum is closest to `target`.


Return <em>the sum of the three integers</em>.


You may assume that each input would have exactly one solution.


 

**Example 1:**

```

**Input:** nums = [-1,2,1,-4], target = 1
**Output:** 2
**Explanation:** The sum that is closest to the target is 2. (-1 + 2 + 1 = 2).

```


**Example 2:**

```

**Input:** nums = [0,0,0], target = 1
**Output:** 0
**Explanation:** The sum that is closest to the target is 0. (0 + 0 + 0 = 0).

```


 

**Constraints:**



	- `3 &lt;= nums.length &lt;= 500`
	- `-1000 &lt;= nums[i] &lt;= 1000`
	- `-10<sup>4</sup> &lt;= target &lt;= 10<sup>4</sup>`
