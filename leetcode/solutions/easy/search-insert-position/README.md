# 搜索插入位置 (Search Insert Position)

## 题目信息

- **难度**: EASY
- **通过率**: 49.3%
- **标签**: `array`, `binary-search`
- **LeetCode**: https://leetcode.cn/problems/search-insert-position/

## 题目描述

Given a sorted array of distinct integers and a target value, return the index if the target is found. If not, return the index where it would be if it were inserted in order.


You must write an algorithm with `O(log n)` runtime complexity.


 

**Example 1:**

```

**Input:** nums = [1,3,5,6], target = 5
**Output:** 2

```


**Example 2:**

```

**Input:** nums = [1,3,5,6], target = 2
**Output:** 1

```


**Example 3:**

```

**Input:** nums = [1,3,5,6], target = 7
**Output:** 4

```


 

**Constraints:**



	- `1 &lt;= nums.length &lt;= 10<sup>4</sup>`
	- `-10<sup>4</sup> &lt;= nums[i] &lt;= 10<sup>4</sup>`
	- `nums` contains **distinct** values sorted in **ascending** order.
	- `-10<sup>4</sup> &lt;= target &lt;= 10<sup>4</sup>`
