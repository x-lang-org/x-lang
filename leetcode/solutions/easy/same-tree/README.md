# 相同的树 (Same Tree)

## 题目信息

- **难度**: EASY
- **通过率**: 63.9%
- **标签**: `tree`, `depth-first-search`, `breadth-first-search`, `binary-tree`
- **LeetCode**: https://leetcode.cn/problems/same-tree/

## 题目描述

Given the roots of two binary trees `p` and `q`, write a function to check if they are the same or not.


Two binary trees are considered the same if they are structurally identical, and the nodes have the same value.


 

**Example 1:**
<img alt="" src="https://assets.leetcode.com/uploads/2020/12/20/ex1.jpg" style="width: 622px; height: 182px;" />
```

**Input:** p = [1,2,3], q = [1,2,3]
**Output:** true

```


**Example 2:**
<img alt="" src="https://assets.leetcode.com/uploads/2020/12/20/ex2.jpg" style="width: 382px; height: 182px;" />
```

**Input:** p = [1,2], q = [1,null,2]
**Output:** false

```


**Example 3:**
<img alt="" src="https://assets.leetcode.com/uploads/2020/12/20/ex3.jpg" style="width: 622px; height: 182px;" />
```

**Input:** p = [1,2,1], q = [1,1,2]
**Output:** false

```


 

**Constraints:**



	- The number of nodes in both trees is in the range `[0, 100]`.
	- `-10<sup>4</sup> &lt;= Node.val &lt;= 10<sup>4</sup>`
