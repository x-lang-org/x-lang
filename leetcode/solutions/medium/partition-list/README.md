# 分隔链表 (Partition List)

## 题目信息

- **难度**: MEDIUM
- **通过率**: 65.8%
- **标签**: `linked-list`, `two-pointers`
- **LeetCode**: https://leetcode.cn/problems/partition-list/

## 题目描述

Given the `head` of a linked list and a value `x`, partition it such that all nodes **less than** `x` come before nodes **greater than or equal** to `x`.


You should **preserve** the original relative order of the nodes in each of the two partitions.


 

**Example 1:**
<img alt="" src="https://assets.leetcode.com/uploads/2021/01/04/partition.jpg" style="width: 662px; height: 222px;" />
```

**Input:** head = [1,4,3,2,5,2], x = 3
**Output:** [1,2,2,4,3,5]

```


**Example 2:**

```

**Input:** head = [2,1], x = 2
**Output:** [1,2]

```


 

**Constraints:**



	- The number of nodes in the list is in the range `[0, 200]`.
	- `-100 &lt;= Node.val &lt;= 100`
	- `-200 &lt;= x &lt;= 200`
