# 柱状图中最大的矩形 (Largest Rectangle in Histogram)

## 题目信息

- **难度**: HARD
- **通过率**: 49.2%
- **标签**: `stack`, `array`, `monotonic-stack`
- **LeetCode**: https://leetcode.cn/problems/largest-rectangle-in-histogram/

## 题目描述

Given an array of integers `heights` representing the histogram&#39;s bar height where the width of each bar is `1`, return <em>the area of the largest rectangle in the histogram</em>.


 

**Example 1:**
<img alt="" src="https://assets.leetcode.com/uploads/2021/01/04/histogram.jpg" style="width: 522px; height: 242px;" />
```

**Input:** heights = [2,1,5,6,2,3]
**Output:** 10
**Explanation:** The above is a histogram where width of each bar is 1.
The largest rectangle is shown in the red area, which has an area = 10 units.

```


**Example 2:**
<img alt="" src="https://assets.leetcode.com/uploads/2021/01/04/histogram-1.jpg" style="width: 202px; height: 362px;" />
```

**Input:** heights = [2,4]
**Output:** 4

```


 

**Constraints:**



	- `1 &lt;= heights.length &lt;= 10<sup>5</sup>`
	- `0 &lt;= heights[i] &lt;= 10<sup>4</sup>`
