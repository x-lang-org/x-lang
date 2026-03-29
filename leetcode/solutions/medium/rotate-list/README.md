# 旋转链表 (Rotate List)

## 题目信息

- **难度**: MEDIUM
- **通过率**: 41.5%
- **标签**: `linked-list`, `two-pointers`
- **LeetCode**: https://leetcode.cn/problems/rotate-list/

## 题目描述

Given the `head` of a linked list, rotate the list to the right by `k` places.


 

**Example 1:**
<img alt="" src="https://assets.leetcode.com/uploads/2020/11/13/rotate1.jpg" style="width: 450px; height: 191px;" />
```

**Input:** head = [1,2,3,4,5], k = 2
**Output:** [4,5,1,2,3]

```


**Example 2:**
<img alt="" src="https://assets.leetcode.com/uploads/2020/11/13/roate2.jpg" style="width: 305px; height: 350px;" />
```

**Input:** head = [0,1,2], k = 4
**Output:** [2,0,1]

```


 

**Constraints:**



	- The number of nodes in the list is in the range `[0, 500]`.
	- `-100 &lt;= Node.val &lt;= 100`
	- `0 &lt;= k &lt;= 2 * 10<sup>9</sup>`
