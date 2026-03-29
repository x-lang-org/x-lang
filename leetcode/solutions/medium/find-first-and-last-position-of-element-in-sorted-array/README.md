# 在排序数组中查找元素的第一个和最后一个位置 (Find First and Last Position of Element in Sorted Array)

## 题目信息

- **难度**: MEDIUM
- **通过率**: 46.5%
- **标签**: `array`, `binary-search`
- **LeetCode**: https://leetcode.cn/problems/find-first-and-last-position-of-element-in-sorted-array/

## 题目描述

Given an array of integers `nums` sorted in non-decreasing order, find the starting and ending position of a given `target` value.


If `target` is not found in the array, return `[-1, -1]`.


You must write an algorithm with `O(log n)` runtime complexity.


 

**Example 1:**
```
**Input:** nums = [5,7,7,8,8,10], target = 8
**Output:** [3,4]

```
**Example 2:**
```
**Input:** nums = [5,7,7,8,8,10], target = 6
**Output:** [-1,-1]

```
**Example 3:**
```
**Input:** nums = [], target = 0
**Output:** [-1,-1]

```

 

**Constraints:**



	- `0 &lt;= nums.length &lt;= 10<sup>5</sup>`
	- `-10<sup>9</sup> &lt;= nums[i] &lt;= 10<sup>9</sup>`
	- `nums` is a non-decreasing array.
	- `-10<sup>9</sup> &lt;= target &lt;= 10<sup>9</sup>`
