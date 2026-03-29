# 合并两个有序链表 (Merge Two Sorted Lists)

## 题目信息

- **难度**: EASY
- **通过率**: 68.1%
- **标签**: `recursion`, `linked-list`
- **LeetCode**: https://leetcode.cn/problems/merge-two-sorted-lists/

## 题目描述

You are given the heads of two sorted linked lists `list1` and `list2`.


Merge the two lists into one **sorted** list. The list should be made by splicing together the nodes of the first two lists.


Return <em>the head of the merged linked list</em>.


 

**Example 1:**
<img alt="" src="https://assets.leetcode.com/uploads/2020/10/03/merge_ex1.jpg" style="width: 662px; height: 302px;" />
```

**Input:** list1 = [1,2,4], list2 = [1,3,4]
**Output:** [1,1,2,3,4,4]

```


**Example 2:**

```

**Input:** list1 = [], list2 = []
**Output:** []

```


**Example 3:**

```

**Input:** list1 = [], list2 = [0]
**Output:** [0]

```


 

**Constraints:**



	- The number of nodes in both lists is in the range `[0, 50]`.
	- `-100 &lt;= Node.val &lt;= 100`
	- Both `list1` and `list2` are sorted in **non-decreasing** order.
