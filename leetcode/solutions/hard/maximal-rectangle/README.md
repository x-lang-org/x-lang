# 最大矩形 (Maximal Rectangle)

## 题目信息

- **难度**: HARD
- **通过率**: 56.8%
- **标签**: `stack`, `array`, `dynamic-programming`, `matrix`, `monotonic-stack`
- **LeetCode**: https://leetcode.cn/problems/maximal-rectangle/

## 题目描述

Given a `rows x cols` binary `matrix` filled with `0`&#39;s and `1`&#39;s, find the largest rectangle containing only `1`&#39;s and return <em>its area</em>.


 

**Example 1:**
<img alt="" src="https://assets.leetcode.com/uploads/2020/09/14/maximal.jpg" style="width: 402px; height: 322px;" />
```

**Input:** matrix = [[&quot;1&quot;,&quot;0&quot;,&quot;1&quot;,&quot;0&quot;,&quot;0&quot;],[&quot;1&quot;,&quot;0&quot;,&quot;1&quot;,&quot;1&quot;,&quot;1&quot;],[&quot;1&quot;,&quot;1&quot;,&quot;1&quot;,&quot;1&quot;,&quot;1&quot;],[&quot;1&quot;,&quot;0&quot;,&quot;0&quot;,&quot;1&quot;,&quot;0&quot;]]
**Output:** 6
**Explanation:** The maximal rectangle is shown in the above picture.

```


**Example 2:**

```

**Input:** matrix = [[&quot;0&quot;]]
**Output:** 0

```


**Example 3:**

```

**Input:** matrix = [[&quot;1&quot;]]
**Output:** 1

```


 

**Constraints:**



	- `rows == matrix.length`
	- `cols == matrix[i].length`
	- `1 &lt;= rows, cols &lt;= 200`
	- `matrix[i][j]` is `&#39;0&#39;` or `&#39;1&#39;`.
