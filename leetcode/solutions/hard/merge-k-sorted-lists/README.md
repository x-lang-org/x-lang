# 合并 K 个升序链表 (Merge k Sorted Lists)

## 题目信息

- **难度**: HARD
- **通过率**: 63.6%
- **标签**: `linked-list`, `divide-and-conquer`, `heap-priority-queue`, `merge-sort`
- **LeetCode**: https://leetcode.cn/problems/merge-k-sorted-lists/

## 题目描述

You are given an array of `k` linked-lists `lists`, each linked-list is sorted in ascending order.


<em>Merge all the linked-lists into one sorted linked-list and return it.</em>


 

**Example 1:**

```

**Input:** lists = [[1,4,5],[1,3,4],[2,6]]
**Output:** [1,1,2,3,4,4,5,6]
**Explanation:** The linked-lists are:
[
  1-&gt;4-&gt;5,
  1-&gt;3-&gt;4,
  2-&gt;6
]
merging them into one sorted linked list:
1-&gt;1-&gt;2-&gt;3-&gt;4-&gt;4-&gt;5-&gt;6

```


**Example 2:**

```

**Input:** lists = []
**Output:** []

```


**Example 3:**

```

**Input:** lists = [[]]
**Output:** []

```


 

**Constraints:**



	- `k == lists.length`
	- `0 &lt;= k &lt;= 10<sup>4</sup>`
	- `0 &lt;= lists[i].length &lt;= 500`
	- `-10<sup>4</sup> &lt;= lists[i][j] &lt;= 10<sup>4</sup>`
	- `lists[i]` is sorted in **ascending order**.
	- The sum of `lists[i].length` will not exceed `10<sup>4</sup>`.
