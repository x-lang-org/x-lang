# K 个一组翻转链表 (Reverse Nodes in k-Group)

## 题目信息

- **难度**: HARD
- **通过率**: 69.9%
- **标签**: `recursion`, `linked-list`
- **LeetCode**: https://leetcode.cn/problems/reverse-nodes-in-k-group/

## 题目描述

Given the `head` of a linked list, reverse the nodes of the list `k` at a time, and return <em>the modified list</em>.


`k` is a positive integer and is less than or equal to the length of the linked list. If the number of nodes is not a multiple of `k` then left-out nodes, in the end, should remain as it is.


You may not alter the values in the list&#39;s nodes, only nodes themselves may be changed.


 

**Example 1:**
<img alt="" src="https://assets.leetcode.com/uploads/2020/10/03/reverse_ex1.jpg" style="width: 542px; height: 222px;" />
```

**Input:** head = [1,2,3,4,5], k = 2
**Output:** [2,1,4,3,5]

```


**Example 2:**
<img alt="" src="https://assets.leetcode.com/uploads/2020/10/03/reverse_ex2.jpg" style="width: 542px; height: 222px;" />
```

**Input:** head = [1,2,3,4,5], k = 3
**Output:** [3,2,1,4,5]

```


 

**Constraints:**



	- The number of nodes in the list is `n`.
	- `1 &lt;= k &lt;= n &lt;= 5000`
	- `0 &lt;= Node.val &lt;= 1000`



 

**Follow-up:** Can you solve the problem in `O(1)` extra memory space?
