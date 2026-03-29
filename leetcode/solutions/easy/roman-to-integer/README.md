# 罗马数字转整数 (Roman to Integer)

## 题目信息

- **难度**: EASY
- **通过率**: 64.2%
- **标签**: `hash-table`, `math`, `string`
- **LeetCode**: https://leetcode.cn/problems/roman-to-integer/

## 题目描述

Roman numerals are represented by seven different symbols: `I`, `V`, `X`, `L`, `C`, `D` and `M`.

```

**Symbol**       **Value**
I             1
V             5
X             10
L             50
C             100
D             500
M             1000
```


For example, `2` is written as `II` in Roman numeral, just two ones added together. `12` is written as `XII`, which is simply `X + II`. The number `27` is written as `XXVII`, which is `XX + V + II`.


Roman numerals are usually written largest to smallest from left to right. However, the numeral for four is not `IIII`. Instead, the number four is written as `IV`. Because the one is before the five we subtract it making four. The same principle applies to the number nine, which is written as `IX`. There are six instances where subtraction is used:



	- `I` can be placed before `V` (5) and `X` (10) to make 4 and 9. 
	- `X` can be placed before `L` (50) and `C` (100) to make 40 and 90. 
	- `C` can be placed before `D` (500) and `M` (1000) to make 400 and 900.



Given a roman numeral, convert it to an integer.


 

**Example 1:**

```

**Input:** s = &quot;III&quot;
**Output:** 3
**Explanation:** III = 3.

```


**Example 2:**

```

**Input:** s = &quot;LVIII&quot;
**Output:** 58
**Explanation:** L = 50, V= 5, III = 3.

```


**Example 3:**

```

**Input:** s = &quot;MCMXCIV&quot;
**Output:** 1994
**Explanation:** M = 1000, CM = 900, XC = 90 and IV = 4.

```


 

**Constraints:**



	- `1 &lt;= s.length &lt;= 15`
	- `s` contains only the characters `(&#39;I&#39;, &#39;V&#39;, &#39;X&#39;, &#39;L&#39;, &#39;C&#39;, &#39;D&#39;, &#39;M&#39;)`.
	- It is **guaranteed** that `s` is a valid roman numeral in the range `[1, 3999]`.
