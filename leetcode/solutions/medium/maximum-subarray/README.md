# 最大子数组和 (Maximum Subarray)

## 题目信息

- **难度**: MEDIUM
- **通过率**: 56.6%
- **标签**: `array`, `divide-and-conquer`, `dynamic-programming`
- **LeetCode**: https://leetcode.cn/problems/maximum-subarray/

## 题目描述

Given an integer array `nums`, find the <span data-keyword="subarray-nonempty">subarray</span> with the largest sum, and return <em>its sum</em>.


 

**Example 1:**

```

**Input:** nums = [-2,1,-3,4,-1,2,1,-5,4]
**Output:** 6
**Explanation:** The subarray [4,-1,2,1] has the largest sum 6.

```


**Example 2:**

```

**Input:** nums = [1]
**Output:** 1
**Explanation:** The subarray [1] has the largest sum 1.

```


**Example 3:**

```

**Input:** nums = [5,4,-1,7,8]
**Output:** 23
**Explanation:** The subarray [5,4,-1,7,8] has the largest sum 23.

```


 

**Constraints:**



	- `1 &lt;= nums.length &lt;= 10<sup>5</sup>`
	- `-10<sup>4</sup> &lt;= nums[i] &lt;= 10<sup>4</sup>`



 

**Follow up:** If you have figured out the `O(n)` solution, try coding another solution using the **divide and conquer** approach, which is more subtle.
