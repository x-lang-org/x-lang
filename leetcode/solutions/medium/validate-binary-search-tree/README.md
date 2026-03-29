# 验证二叉搜索树 (Validate Binary Search Tree)

## 题目信息

- **难度**: MEDIUM
- **通过率**: 40.8%
- **标签**: `tree`, `depth-first-search`, `binary-search-tree`, `binary-tree`
- **LeetCode**: https://leetcode.cn/problems/validate-binary-search-tree/

## 题目描述

Given the `root` of a binary tree, <em>determine if it is a valid binary search tree (BST)</em>.


A **valid BST** is defined as follows:



	- The left <span data-keyword="subtree">subtree</span> of a node contains only nodes with keys **strictly less than** the node&#39;s key.
	- The right subtree of a node contains only nodes with keys **strictly greater than** the node&#39;s key.
	- Both the left and right subtrees must also be binary search trees.



 

**Example 1:**
<img alt="" src="https://assets.leetcode.com/uploads/2020/12/01/tree1.jpg" style="width: 302px; height: 182px;" />
```

**Input:** root = [2,1,3]
**Output:** true

```


**Example 2:**
<img alt="" src="https://assets.leetcode.com/uploads/2020/12/01/tree2.jpg" style="width: 422px; height: 292px;" />
```

**Input:** root = [5,1,4,null,null,3,6]
**Output:** false
**Explanation:** The root node&#39;s value is 5 but its right child&#39;s value is 4.

```


 

**Constraints:**



	- The number of nodes in the tree is in the range `[1, 10<sup>4</sup>]`.
	- `-2<sup>31</sup> &lt;= Node.val &lt;= 2<sup>31</sup> - 1`
