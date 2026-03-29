# 单词搜索 (Word Search)

## 题目信息

- **难度**: MEDIUM
- **通过率**: 50.8%
- **标签**: `depth-first-search`, `array`, `string`, `backtracking`, `matrix`
- **LeetCode**: https://leetcode.cn/problems/word-search/

## 题目描述

Given an `m x n` grid of characters `board` and a string `word`, return `true` <em>if</em> `word` <em>exists in the grid</em>.


The word can be constructed from letters of sequentially adjacent cells, where adjacent cells are horizontally or vertically neighboring. The same letter cell may not be used more than once.


 

**Example 1:**
<img alt="" src="https://assets.leetcode.com/uploads/2020/11/04/word2.jpg" style="width: 322px; height: 242px;" />
```

**Input:** board = [[&quot;A&quot;,&quot;B&quot;,&quot;C&quot;,&quot;E&quot;],[&quot;S&quot;,&quot;F&quot;,&quot;C&quot;,&quot;S&quot;],[&quot;A&quot;,&quot;D&quot;,&quot;E&quot;,&quot;E&quot;]], word = &quot;ABCCED&quot;
**Output:** true

```


**Example 2:**
<img alt="" src="https://assets.leetcode.com/uploads/2020/11/04/word-1.jpg" style="width: 322px; height: 242px;" />
```

**Input:** board = [[&quot;A&quot;,&quot;B&quot;,&quot;C&quot;,&quot;E&quot;],[&quot;S&quot;,&quot;F&quot;,&quot;C&quot;,&quot;S&quot;],[&quot;A&quot;,&quot;D&quot;,&quot;E&quot;,&quot;E&quot;]], word = &quot;SEE&quot;
**Output:** true

```


**Example 3:**
<img alt="" src="https://assets.leetcode.com/uploads/2020/10/15/word3.jpg" style="width: 322px; height: 242px;" />
```

**Input:** board = [[&quot;A&quot;,&quot;B&quot;,&quot;C&quot;,&quot;E&quot;],[&quot;S&quot;,&quot;F&quot;,&quot;C&quot;,&quot;S&quot;],[&quot;A&quot;,&quot;D&quot;,&quot;E&quot;,&quot;E&quot;]], word = &quot;ABCB&quot;
**Output:** false

```


 

**Constraints:**



	- `m == board.length`
	- `n = board[i].length`
	- `1 &lt;= m, n &lt;= 6`
	- `1 &lt;= word.length &lt;= 15`
	- `board` and `word` consists of only lowercase and uppercase English letters.



 

**Follow up:** Could you use search pruning to make your solution faster with a larger `board`?
