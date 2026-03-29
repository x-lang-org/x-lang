# 搜索旋转排序数组 II (Search in Rotated Sorted Array II)

## 题目信息

- **难度**: MEDIUM
- **通过率**: 41.5%
- **标签**: `array`, `binary-search`
- **LeetCode**: https://leetcode.cn/problems/search-in-rotated-sorted-array-ii/

## 题目描述

There is an integer array `nums` sorted in non-decreasing order (not necessarily with **distinct** values).


Before being passed to your function, `nums` is **rotated** at an unknown pivot index `k` (`0 &lt;= k &lt; nums.length`) such that the resulting array is `[nums[k], nums[k+1], ..., nums[n-1], nums[0], nums[1], ..., nums[k-1]]` (**0-indexed**). For example, `[0,1,2,4,4,4,5,6,6,7]` might be rotated at pivot index `5` and become `[4,5,6,6,7,0,1,2,4,4]`.


Given the array `nums` **after** the rotation and an integer `target`, return `true`<em> if </em>`target`<em> is in </em>`nums`<em>, or </em>`false`<em> if it is not in </em>`nums`<em>.</em>


You must decrease the overall operation steps as much as possible.


 

**Example 1:**
```
**Input:** nums = [2,5,6,0,0,1,2], target = 0
**Output:** true

```
**Example 2:**
```
**Input:** nums = [2,5,6,0,0,1,2], target = 3
**Output:** false

```

 

**Constraints:**



	- `1 &lt;= nums.length &lt;= 5000`
	- `-10<sup>4</sup> &lt;= nums[i] &lt;= 10<sup>4</sup>`
	- `nums` is guaranteed to be rotated at some pivot.
	- `-10<sup>4</sup> &lt;= target &lt;= 10<sup>4</sup>`



 

**Follow up:** This problem is similar to <a href="/problems/search-in-rotated-sorted-array/description/" target="_blank">Search in Rotated Sorted Array</a>, but `nums` may contain **duplicates**. Would this affect the runtime complexity? How and why?
