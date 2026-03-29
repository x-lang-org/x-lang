# 两数相除 (Divide Two Integers)

## 题目信息

- **难度**: MEDIUM
- **通过率**: 22.5%
- **标签**: `bit-manipulation`, `math`
- **LeetCode**: https://leetcode.cn/problems/divide-two-integers/

## 题目描述

Given two integers `dividend` and `divisor`, divide two integers **without** using multiplication, division, and mod operator.


The integer division should truncate toward zero, which means losing its fractional part. For example, `8.345` would be truncated to `8`, and `-2.7335` would be truncated to `-2`.


Return <em>the **quotient** after dividing </em>`dividend`<em> by </em>`divisor`.


**Note: **Assume we are dealing with an environment that could only store integers within the **32-bit** signed integer range: `[&minus;2<sup>31</sup>, 2<sup>31</sup> &minus; 1]`. For this problem, if the quotient is **strictly greater than** `2<sup>31</sup> - 1`, then return `2<sup>31</sup> - 1`, and if the quotient is **strictly less than** `-2<sup>31</sup>`, then return `-2<sup>31</sup>`.


 

**Example 1:**

```

**Input:** dividend = 10, divisor = 3
**Output:** 3
**Explanation:** 10/3 = 3.33333.. which is truncated to 3.

```


**Example 2:**

```

**Input:** dividend = 7, divisor = -3
**Output:** -2
**Explanation:** 7/-3 = -2.33333.. which is truncated to -2.

```


 

**Constraints:**



	- `-2<sup>31</sup> &lt;= dividend, divisor &lt;= 2<sup>31</sup> - 1`
	- `divisor != 0`
