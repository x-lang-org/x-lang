# N 皇后 (N-Queens)

## 题目信息

- **难度**: HARD
- **通过率**: 75.4%
- **标签**: `array`, `backtracking`
- **LeetCode**: https://leetcode.cn/problems/n-queens/

## 题目描述

The **n-queens** puzzle is the problem of placing `n` queens on an `n x n` chessboard such that no two queens attack each other.


Given an integer `n`, return <em>all distinct solutions to the **n-queens puzzle**</em>. You may return the answer in **any order**.


Each solution contains a distinct board configuration of the n-queens&#39; placement, where `&#39;Q&#39;` and `&#39;.&#39;` both indicate a queen and an empty space, respectively.


 

**Example 1:**
<img alt="" src="https://assets.leetcode.com/uploads/2020/11/13/queens.jpg" style="width: 600px; height: 268px;" />
```

**Input:** n = 4
**Output:** [[&quot;.Q..&quot;,&quot;...Q&quot;,&quot;Q...&quot;,&quot;..Q.&quot;],[&quot;..Q.&quot;,&quot;Q...&quot;,&quot;...Q&quot;,&quot;.Q..&quot;]]
**Explanation:** There exist two distinct solutions to the 4-queens puzzle as shown above

```


**Example 2:**

```

**Input:** n = 1
**Output:** [[&quot;Q&quot;]]

```


 

**Constraints:**



	- `1 &lt;= n &lt;= 9`
