# 删除排序链表中的重复元素 II (Remove Duplicates from Sorted List II)

## 题目信息

- **难度**: MEDIUM
- **通过率**: 55.4%
- **标签**: `linked-list`, `two-pointers`
- **LeetCode**: https://leetcode.cn/problems/remove-duplicates-from-sorted-list-ii/

## 题目描述

Given the `head` of a sorted linked list, <em>delete all nodes that have duplicate numbers, leaving only distinct numbers from the original list</em>. Return <em>the linked list **sorted** as well</em>.


 

**Example 1:**
<img alt="" src="https://assets.leetcode.com/uploads/2021/01/04/linkedlist1.jpg" style="width: 500px; height: 142px;" />
```

**Input:** head = [1,2,3,3,4,4,5]
**Output:** [1,2,5]

```


**Example 2:**
<img alt="" src="https://assets.leetcode.com/uploads/2021/01/04/linkedlist2.jpg" style="width: 500px; height: 205px;" />
```

**Input:** head = [1,1,1,2,3]
**Output:** [2,3]

```


 

**Constraints:**



	- The number of nodes in the list is in the range `[0, 300]`.
	- `-100 &lt;= Node.val &lt;= 100`
	- The list is guaranteed to be **sorted** in ascending order.
