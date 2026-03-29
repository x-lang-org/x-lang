# 恢复二叉搜索树 (Recover Binary Search Tree)

## 题目信息

- **难度**: MEDIUM
- **通过率**: 61.7%
- **标签**: `tree`, `depth-first-search`, `binary-search-tree`, `binary-tree`
- **LeetCode**: https://leetcode.cn/problems/recover-binary-search-tree/

## 题目描述

You are given the `root` of a binary search tree (BST), where the values of **exactly** two nodes of the tree were swapped by mistake. <em>Recover the tree without changing its structure</em>.


 

**Example 1:**
<img alt="" src="https://assets.leetcode.com/uploads/2020/10/28/recover1.jpg" style="width: 422px; height: 302px;" />
```

**Input:** root = [1,3,null,null,2]
**Output:** [3,1,null,null,2]
**Explanation:** 3 cannot be a left child of 1 because 3 &gt; 1. Swapping 1 and 3 makes the BST valid.

```


**Example 2:**
<img alt="" src="https://assets.leetcode.com/uploads/2020/10/28/recover2.jpg" style="width: 581px; height: 302px;" />
```

**Input:** root = [3,1,4,null,null,2]
**Output:** [2,1,4,null,null,3]
**Explanation:** 2 cannot be in the right subtree of 3 because 2 &lt; 3. Swapping 2 and 3 makes the BST valid.

```


 

**Constraints:**



	- The number of nodes in the tree is in the range `[2, 1000]`.
	- `-2<sup>31</sup> &lt;= Node.val &lt;= 2<sup>31</sup> - 1`



 
**Follow up:** A solution using `O(n)` space is pretty straight-forward. Could you devise a constant `O(1)` space solution?
