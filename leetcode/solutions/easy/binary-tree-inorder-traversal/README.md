# 二叉树的中序遍历 (Binary Tree Inorder Traversal)

## 题目信息

- **难度**: EASY
- **通过率**: 78.3%
- **标签**: `stack`, `tree`, `depth-first-search`, `binary-tree`
- **LeetCode**: https://leetcode.cn/problems/binary-tree-inorder-traversal/

## 题目描述

Given the `root` of a binary tree, return <em>the inorder traversal of its nodes&#39; values</em>.


 

**Example 1:**

<div class="example-block">

**Input:** <span class="example-io">root = [1,null,2,3]</span>


**Output:** <span class="example-io">[1,3,2]</span>


**Explanation:**


<img alt="" src="https://assets.leetcode.com/uploads/2024/08/29/screenshot-2024-08-29-202743.png" style="width: 200px; height: 264px;" />
</div>


**Example 2:**

<div class="example-block">

**Input:** <span class="example-io">root = [1,2,3,4,5,null,8,null,null,6,7,9]</span>


**Output:** <span class="example-io">[4,2,6,5,7,1,3,9,8]</span>


**Explanation:**


<img alt="" src="https://assets.leetcode.com/uploads/2024/08/29/tree_2.png" style="width: 350px; height: 286px;" />
</div>


**Example 3:**

<div class="example-block">

**Input:** <span class="example-io">root = []</span>


**Output:** <span class="example-io">[]</span>
</div>


**Example 4:**

<div class="example-block">

**Input:** <span class="example-io">root = [1]</span>


**Output:** <span class="example-io">[1]</span>
</div>


 

**Constraints:**



	- The number of nodes in the tree is in the range `[0, 100]`.
	- `-100 &lt;= Node.val &lt;= 100`



 
**Follow up:** Recursive solution is trivial, could you do it iteratively?
