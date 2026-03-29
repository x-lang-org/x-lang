# Z 字形变换 (Zigzag Conversion)

## 题目信息

- **难度**: MEDIUM
- **通过率**: 54.2%
- **标签**: `string`
- **LeetCode**: https://leetcode.cn/problems/zigzag-conversion/

## 题目描述

The string `&quot;PAYPALISHIRING&quot;` is written in a zigzag pattern on a given number of rows like this: (you may want to display this pattern in a fixed font for better legibility)

```

P   A   H   N
A P L S I I G
Y   I   R

```


And then read line by line: `&quot;PAHNAPLSIIGYIR&quot;`


Write the code that will take a string and make this conversion given a number of rows:

```

string convert(string s, int numRows);

```


 

**Example 1:**

```

**Input:** s = &quot;PAYPALISHIRING&quot;, numRows = 3
**Output:** &quot;PAHNAPLSIIGYIR&quot;

```


**Example 2:**

```

**Input:** s = &quot;PAYPALISHIRING&quot;, numRows = 4
**Output:** &quot;PINALSIGYAHRPI&quot;
**Explanation:**
P     I    N
A   L S  I G
Y A   H R
P     I

```


**Example 3:**

```

**Input:** s = &quot;A&quot;, numRows = 1
**Output:** &quot;A&quot;

```


 

**Constraints:**



	- `1 &lt;= s.length &lt;= 1000`
	- `s` consists of English letters (lower-case and upper-case), `&#39;,&#39;` and `&#39;.&#39;`.
	- `1 &lt;= numRows &lt;= 1000`
