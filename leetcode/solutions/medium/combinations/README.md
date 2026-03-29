# 组合 (Combinations)

## 题目信息

- **难度**: MEDIUM
- **通过率**: 77.5%
- **标签**: `backtracking`
- **LeetCode**: https://leetcode.cn/problems/combinations/

## 题目描述

Given two integers `n` and `k`, return <em>all possible combinations of</em> `k` <em>numbers chosen from the range</em> `[1, n]`.


You may return the answer in **any order**.


 

**Example 1:**

```

**Input:** n = 4, k = 2
**Output:** [[1,2],[1,3],[1,4],[2,3],[2,4],[3,4]]
**Explanation:** There are 4 choose 2 = 6 total combinations.
Note that combinations are unordered, i.e., [1,2] and [2,1] are considered to be the same combination.

```


**Example 2:**

```

**Input:** n = 1, k = 1
**Output:** [[1]]
**Explanation:** There is 1 choose 1 = 1 total combination.

```


 

**Constraints:**



	- `1 &lt;= n &lt;= 20`
	- `1 &lt;= k &lt;= n`
