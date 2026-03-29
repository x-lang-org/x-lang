# 不同路径 II (Unique Paths II)

## 题目信息

- **难度**: MEDIUM
- **通过率**: 42.5%
- **标签**: `array`, `dynamic-programming`, `matrix`
- **LeetCode**: https://leetcode.cn/problems/unique-paths-ii/

## 题目描述

You are given an `m x n` integer array `grid`. There is a robot initially located at the <b>top-left corner</b> (i.e., `grid[0][0]`). The robot tries to move to the **bottom-right corner** (i.e., `grid[m - 1][n - 1]`). The robot can only move either down or right at any point in time.


An obstacle and space are marked as `1` or `0` respectively in `grid`. A path that the robot takes cannot include **any** square that is an obstacle.


Return <em>the number of possible unique paths that the robot can take to reach the bottom-right corner</em>.


The testcases are generated so that the answer will be less than or equal to `2 * 10<sup>9</sup>`.


 

**Example 1:**
<img alt="" src="https://assets.leetcode.com/uploads/2020/11/04/robot1.jpg" style="width: 242px; height: 242px;" />
```

**Input:** obstacleGrid = [[0,0,0],[0,1,0],[0,0,0]]
**Output:** 2
**Explanation:** There is one obstacle in the middle of the 3x3 grid above.
There are two ways to reach the bottom-right corner:
1. Right -&gt; Right -&gt; Down -&gt; Down
2. Down -&gt; Down -&gt; Right -&gt; Right

```


**Example 2:**
<img alt="" src="https://assets.leetcode.com/uploads/2020/11/04/robot2.jpg" style="width: 162px; height: 162px;" />
```

**Input:** obstacleGrid = [[0,1],[0,0]]
**Output:** 1

```


 

**Constraints:**



	- `m == obstacleGrid.length`
	- `n == obstacleGrid[i].length`
	- `1 &lt;= m, n &lt;= 100`
	- `obstacleGrid[i][j]` is `0` or `1`.
