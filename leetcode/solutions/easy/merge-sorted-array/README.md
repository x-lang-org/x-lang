# 合并两个有序数组 (Merge Sorted Array)

## 题目信息

- **难度**: EASY
- **通过率**: 55.0%
- **标签**: `array`, `two-pointers`, `sorting`
- **LeetCode**: https://leetcode.cn/problems/merge-sorted-array/

## 题目描述

You are given two integer arrays `nums1` and `nums2`, sorted in **non-decreasing order**, and two integers `m` and `n`, representing the number of elements in `nums1` and `nums2` respectively.


**Merge** `nums1` and `nums2` into a single array sorted in **non-decreasing order**.


The final sorted array should not be returned by the function, but instead be <em>stored inside the array </em>`nums1`. To accommodate this, `nums1` has a length of `m + n`, where the first `m` elements denote the elements that should be merged, and the last `n` elements are set to `0` and should be ignored. `nums2` has a length of `n`.


 

**Example 1:**

```

**Input:** nums1 = [1,2,3,0,0,0], m = 3, nums2 = [2,5,6], n = 3
**Output:** [1,2,2,3,5,6]
**Explanation:** The arrays we are merging are [1,2,3] and [2,5,6].
The result of the merge is [<u>1</u>,<u>2</u>,2,<u>3</u>,5,6] with the underlined elements coming from nums1.

```


**Example 2:**

```

**Input:** nums1 = [1], m = 1, nums2 = [], n = 0
**Output:** [1]
**Explanation:** The arrays we are merging are [1] and [].
The result of the merge is [1].

```


**Example 3:**

```

**Input:** nums1 = [0], m = 0, nums2 = [1], n = 1
**Output:** [1]
**Explanation:** The arrays we are merging are [] and [1].
The result of the merge is [1].
Note that because m = 0, there are no elements in nums1. The 0 is only there to ensure the merge result can fit in nums1.

```


 

**Constraints:**



	- `nums1.length == m + n`
	- `nums2.length == n`
	- `0 &lt;= m, n &lt;= 200`
	- `1 &lt;= m + n &lt;= 200`
	- `-10<sup>9</sup> &lt;= nums1[i], nums2[j] &lt;= 10<sup>9</sup>`



 

**Follow up: **Can you come up with an algorithm that runs in `O(m + n)` time?
