# 格雷编码 (Gray Code)

## 题目信息

- **难度**: MEDIUM
- **通过率**: 75.2%
- **标签**: `bit-manipulation`, `math`, `backtracking`
- **LeetCode**: https://leetcode.cn/problems/gray-code/

## 题目描述

An **n-bit gray code sequence** is a sequence of `2<sup>n</sup>` integers where:



	- Every integer is in the **inclusive** range `[0, 2<sup>n</sup> - 1]`,
	- The first integer is `0`,
	- An integer appears **no more than once** in the sequence,
	- The binary representation of every pair of **adjacent** integers differs by **exactly one bit**, and
	- The binary representation of the **first** and **last** integers differs by **exactly one bit**.



Given an integer `n`, return <em>any valid **n-bit gray code sequence**</em>.


 

**Example 1:**

```

**Input:** n = 2
**Output:** [0,1,3,2]
**Explanation:**
The binary representation of [0,1,3,2] is [00,01,11,10].
- 0<u>0</u> and 0<u>1</u> differ by one bit
- <u>0</u>1 and <u>1</u>1 differ by one bit
- 1<u>1</u> and 1<u>0</u> differ by one bit
- <u>1</u>0 and <u>0</u>0 differ by one bit
[0,2,3,1] is also a valid gray code sequence, whose binary representation is [00,10,11,01].
- <u>0</u>0 and <u>1</u>0 differ by one bit
- 1<u>0</u> and 1<u>1</u> differ by one bit
- <u>1</u>1 and <u>0</u>1 differ by one bit
- 0<u>1</u> and 0<u>0</u> differ by one bit

```


**Example 2:**

```

**Input:** n = 1
**Output:** [0,1]

```


 

**Constraints:**



	- `1 &lt;= n &lt;= 16`
