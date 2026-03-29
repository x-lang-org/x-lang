# 不同路径 (Unique Paths)

## 题目信息

- **难度**: MEDIUM
- **通过率**: 70.2%
- **标签**: `math`, `dynamic-programming`, `combinatorics`
- **LeetCode**: https://leetcode.cn/problems/unique-paths/

## 题目描述

There is a robot on an `m x n` grid. The robot is initially located at the **top-left corner** (i.e., `grid[0][0]`). The robot tries to move to the **bottom-right corner** (i.e., `grid[m - 1][n - 1]`). The robot can only move either down or right at any point in time.


Given the two integers `m` and `n`, return <em>the number of possible unique paths that the robot can take to reach the bottom-right corner</em>.


The test cases are generated so that the answer will be less than or equal to `2 * 10<sup>9</sup>`.


 

**Example 1:**
<img src="https://assets.leetcode.com/uploads/2018/10/22/robot_maze.png" style="width: 400px; height: 183px;" />
```

**Input:** m = 3, n = 7
**Output:** 28

```


**Example 2:**

```

**Input:** m = 3, n = 2
**Output:** 3
**Explanation:** From the top-left corner, there are a total of 3 ways to reach the bottom-right corner:
1. Right -&gt; Down -&gt; Down
2. Down -&gt; Down -&gt; Right
3. Down -&gt; Right -&gt; Down

```


 

**Constraints:**



	- `1 &lt;= m, n &lt;= 100`
