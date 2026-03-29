# 矩阵置零 (Set Matrix Zeroes)

## 题目信息

- **难度**: MEDIUM
- **通过率**: 71.5%
- **标签**: `array`, `hash-table`, `matrix`
- **LeetCode**: https://leetcode.cn/problems/set-matrix-zeroes/

## 题目描述

Given an `m x n` integer matrix `matrix`, if an element is `0`, set its entire row and column to `0`&#39;s.


You must do it <a href="https://en.wikipedia.org/wiki/In-place_algorithm" target="_blank">in place</a>.


 

**Example 1:**
<img alt="" src="https://assets.leetcode.com/uploads/2020/08/17/mat1.jpg" style="width: 450px; height: 169px;" />
```

**Input:** matrix = [[1,1,1],[1,0,1],[1,1,1]]
**Output:** [[1,0,1],[0,0,0],[1,0,1]]

```


**Example 2:**
<img alt="" src="https://assets.leetcode.com/uploads/2020/08/17/mat2.jpg" style="width: 450px; height: 137px;" />
```

**Input:** matrix = [[0,1,2,0],[3,4,5,2],[1,3,1,5]]
**Output:** [[0,0,0,0],[0,4,5,0],[0,3,1,0]]

```


 

**Constraints:**



	- `m == matrix.length`
	- `n == matrix[0].length`
	- `1 &lt;= m, n &lt;= 200`
	- `-2<sup>31</sup> &lt;= matrix[i][j] &lt;= 2<sup>31</sup> - 1`



 

**Follow up:**



	- A straightforward solution using `O(mn)` space is probably a bad idea.
	- A simple improvement uses `O(m + n)` space, but still not the best solution.
	- Could you devise a constant space solution?
