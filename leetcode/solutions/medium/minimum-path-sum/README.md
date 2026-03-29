# 最小路径和 (Minimum Path Sum)

## 题目信息

- **难度**: MEDIUM
- **通过率**: 72.5%
- **标签**: `array`, `dynamic-programming`, `matrix`
- **LeetCode**: https://leetcode.cn/problems/minimum-path-sum/

## 题目描述

Given a `m x n` `grid` filled with non-negative numbers, find a path from top left to bottom right, which minimizes the sum of all numbers along its path.


**Note:** You can only move either down or right at any point in time.


 

**Example 1:**
<img alt="" src="https://assets.leetcode.com/uploads/2020/11/05/minpath.jpg" style="width: 242px; height: 242px;" />
```

**Input:** grid = [[1,3,1],[1,5,1],[4,2,1]]
**Output:** 7
**Explanation:** Because the path 1 &rarr; 3 &rarr; 1 &rarr; 1 &rarr; 1 minimizes the sum.

```


**Example 2:**

```

**Input:** grid = [[1,2,3],[4,5,6]]
**Output:** 12

```


 

**Constraints:**



	- `m == grid.length`
	- `n == grid[i].length`
	- `1 &lt;= m, n &lt;= 200`
	- `0 &lt;= grid[i][j] &lt;= 200`
