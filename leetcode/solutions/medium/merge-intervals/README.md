# 合并区间 (Merge Intervals)

## 题目信息

- **难度**: MEDIUM
- **通过率**: 53.3%
- **标签**: `array`, `sorting`
- **LeetCode**: https://leetcode.cn/problems/merge-intervals/

## 题目描述

Given an array of `intervals` where `intervals[i] = [start<sub>i</sub>, end<sub>i</sub>]`, merge all overlapping intervals, and return <em>an array of the non-overlapping intervals that cover all the intervals in the input</em>.


 

**Example 1:**

```

**Input:** intervals = [[1,3],[2,6],[8,10],[15,18]]
**Output:** [[1,6],[8,10],[15,18]]
**Explanation:** Since intervals [1,3] and [2,6] overlap, merge them into [1,6].

```


**Example 2:**

```

**Input:** intervals = [[1,4],[4,5]]
**Output:** [[1,5]]
**Explanation:** Intervals [1,4] and [4,5] are considered overlapping.

```


**Example 3:**

```

**Input:** intervals = [[4,7],[1,4]]
**Output:** [[1,7]]
**Explanation:** Intervals [1,4] and [4,7] are considered overlapping.

```


 

**Constraints:**



	- `1 &lt;= intervals.length &lt;= 10<sup>4</sup>`
	- `intervals[i].length == 2`
	- `0 &lt;= start<sub>i</sub> &lt;= end<sub>i</sub> &lt;= 10<sup>4</sup>`
