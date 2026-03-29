# 两两交换链表中的节点 (Swap Nodes in Pairs)

## 题目信息

- **难度**: MEDIUM
- **通过率**: 75.0%
- **标签**: `recursion`, `linked-list`
- **LeetCode**: https://leetcode.cn/problems/swap-nodes-in-pairs/

## 题目描述

Given a linked list, swap every two adjacent nodes and return its head. You must solve the problem without modifying the values in the list&#39;s nodes (i.e., only nodes themselves may be changed.)


 

**Example 1:**

<div class="example-block">

**Input:** <span class="example-io">head = [1,2,3,4]</span>


**Output:** <span class="example-io">[2,1,4,3]</span>


**Explanation:**


<img alt="" src="https://assets.leetcode.com/uploads/2020/10/03/swap_ex1.jpg" style="width: 422px; height: 222px;" />
</div>


**Example 2:**

<div class="example-block">

**Input:** <span class="example-io">head = []</span>


**Output:** <span class="example-io">[]</span>
</div>


**Example 3:**

<div class="example-block">

**Input:** <span class="example-io">head = [1]</span>


**Output:** <span class="example-io">[1]</span>
</div>


**Example 4:**

<div class="example-block">

**Input:** <span class="example-io">head = [1,2,3]</span>


**Output:** <span class="example-io">[2,1,3]</span>
</div>


 

**Constraints:**



	- The number of nodes in the list is in the range `[0, 100]`.
	- `0 &lt;= Node.val &lt;= 100`
