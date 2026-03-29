# 颜色分类 (Sort Colors)

## 题目信息

- **难度**: MEDIUM
- **通过率**: 63.6%
- **标签**: `array`, `two-pointers`, `sorting`
- **LeetCode**: https://leetcode.cn/problems/sort-colors/

## 题目描述

Given an array `nums` with `n` objects colored red, white, or blue, sort them **<a href="https://en.wikipedia.org/wiki/In-place_algorithm" target="_blank">in-place</a> **so that objects of the same color are adjacent, with the colors in the order red, white, and blue.


We will use the integers `0`, `1`, and `2` to represent the color red, white, and blue, respectively.


You must solve this problem without using the library&#39;s sort function.


 

**Example 1:**

```

**Input:** nums = [2,0,2,1,1,0]
**Output:** [0,0,1,1,2,2]

```


**Example 2:**

```

**Input:** nums = [2,0,1]
**Output:** [0,1,2]

```


 

**Constraints:**



	- `n == nums.length`
	- `1 &lt;= n &lt;= 300`
	- `nums[i]` is either `0`, `1`, or `2`.



 

**Follow up:** Could you come up with a one-pass algorithm using only constant extra space?
