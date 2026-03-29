# 缺失的第一个正数 (First Missing Positive)

## 题目信息

- **难度**: HARD
- **通过率**: 49.1%
- **标签**: `array`, `hash-table`
- **LeetCode**: https://leetcode.cn/problems/first-missing-positive/

## 题目描述

Given an unsorted integer array `nums`. Return the <em>smallest positive integer</em> that is <em>not present</em> in `nums`.


You must implement an algorithm that runs in `O(n)` time and uses `O(1)` auxiliary space.


 

**Example 1:**

```

**Input:** nums = [1,2,0]
**Output:** 3
**Explanation:** The numbers in the range [1,2] are all in the array.

```


**Example 2:**

```

**Input:** nums = [3,4,-1,1]
**Output:** 2
**Explanation:** 1 is in the array but 2 is missing.

```


**Example 3:**

```

**Input:** nums = [7,8,9,11,12]
**Output:** 1
**Explanation:** The smallest positive integer 1 is missing.

```


 

**Constraints:**



	- `1 &lt;= nums.length &lt;= 10<sup>5</sup>`
	- `-2<sup>31</sup> &lt;= nums[i] &lt;= 2<sup>31</sup> - 1`
