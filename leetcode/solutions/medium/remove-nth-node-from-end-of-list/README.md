# 删除链表的倒数第 N 个结点 (Remove Nth Node From End of List)

## 题目信息

- **难度**: MEDIUM
- **通过率**: 53.0%
- **标签**: `linked-list`, `two-pointers`
- **LeetCode**: https://leetcode.cn/problems/remove-nth-node-from-end-of-list/

## 题目描述

Given the `head` of a linked list, remove the `n<sup>th</sup>` node from the end of the list and return its head.


 

**Example 1:**
<img alt="" src="https://assets.leetcode.com/uploads/2020/10/03/remove_ex1.jpg" style="width: 542px; height: 222px;" />
```

**Input:** head = [1,2,3,4,5], n = 2
**Output:** [1,2,3,5]

```


**Example 2:**

```

**Input:** head = [1], n = 1
**Output:** []

```


**Example 3:**

```

**Input:** head = [1,2], n = 1
**Output:** [1]

```


 

**Constraints:**



	- The number of nodes in the list is `sz`.
	- `1 &lt;= sz &lt;= 30`
	- `0 &lt;= Node.val &lt;= 100`
	- `1 &lt;= n &lt;= sz`



 

**Follow up:** Could you do this in one pass?
