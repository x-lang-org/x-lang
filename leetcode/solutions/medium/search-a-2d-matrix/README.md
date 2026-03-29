# 搜索二维矩阵 (Search a 2D Matrix)

## 题目信息

- **难度**: MEDIUM
- **通过率**: 52.4%
- **标签**: `array`, `binary-search`, `matrix`
- **LeetCode**: https://leetcode.cn/problems/search-a-2d-matrix/

## 题目描述

You are given an `m x n` integer matrix `matrix` with the following two properties:



	- Each row is sorted in non-decreasing order.
	- The first integer of each row is greater than the last integer of the previous row.



Given an integer `target`, return `true` <em>if</em> `target` <em>is in</em> `matrix` <em>or</em> `false` <em>otherwise</em>.


You must write a solution in `O(log(m * n))` time complexity.


 

**Example 1:**
<img alt="" src="https://assets.leetcode.com/uploads/2020/10/05/mat.jpg" style="width: 322px; height: 242px;" />
```

**Input:** matrix = [[1,3,5,7],[10,11,16,20],[23,30,34,60]], target = 3
**Output:** true

```


**Example 2:**
<img alt="" src="https://assets.leetcode.com/uploads/2020/10/05/mat2.jpg" style="width: 322px; height: 242px;" />
```

**Input:** matrix = [[1,3,5,7],[10,11,16,20],[23,30,34,60]], target = 13
**Output:** false

```


 

**Constraints:**



	- `m == matrix.length`
	- `n == matrix[i].length`
	- `1 &lt;= m, n &lt;= 100`
	- `-10<sup>4</sup> &lt;= matrix[i][j], target &lt;= 10<sup>4</sup>`
