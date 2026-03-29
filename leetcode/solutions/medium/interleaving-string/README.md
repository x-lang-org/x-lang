# 交错字符串 (Interleaving String)

## 题目信息

- **难度**: MEDIUM
- **通过率**: 46.7%
- **标签**: `string`, `dynamic-programming`
- **LeetCode**: https://leetcode.cn/problems/interleaving-string/

## 题目描述

Given strings `s1`, `s2`, and `s3`, find whether `s3` is formed by an **interleaving** of `s1` and `s2`.


An **interleaving** of two strings `s` and `t` is a configuration where `s` and `t` are divided into `n` and `m` <span data-keyword="substring-nonempty">substrings</span> respectively, such that:



	- `s = s<sub>1</sub> + s<sub>2</sub> + ... + s<sub>n</sub>`
	- `t = t<sub>1</sub> + t<sub>2</sub> + ... + t<sub>m</sub>`
	- `|n - m| &lt;= 1`
	- The **interleaving** is `s<sub>1</sub> + t<sub>1</sub> + s<sub>2</sub> + t<sub>2</sub> + s<sub>3</sub> + t<sub>3</sub> + ...` or `t<sub>1</sub> + s<sub>1</sub> + t<sub>2</sub> + s<sub>2</sub> + t<sub>3</sub> + s<sub>3</sub> + ...`



**Note:** `a + b` is the concatenation of strings `a` and `b`.


 

**Example 1:**
<img alt="" src="https://assets.leetcode.com/uploads/2020/09/02/interleave.jpg" style="width: 561px; height: 203px;" />
```

**Input:** s1 = &quot;aabcc&quot;, s2 = &quot;dbbca&quot;, s3 = &quot;aadbbcbcac&quot;
**Output:** true
**Explanation:** One way to obtain s3 is:
Split s1 into s1 = &quot;aa&quot; + &quot;bc&quot; + &quot;c&quot;, and s2 into s2 = &quot;dbbc&quot; + &quot;a&quot;.
Interleaving the two splits, we get &quot;aa&quot; + &quot;dbbc&quot; + &quot;bc&quot; + &quot;a&quot; + &quot;c&quot; = &quot;aadbbcbcac&quot;.
Since s3 can be obtained by interleaving s1 and s2, we return true.

```


**Example 2:**

```

**Input:** s1 = &quot;aabcc&quot;, s2 = &quot;dbbca&quot;, s3 = &quot;aadbbbaccc&quot;
**Output:** false
**Explanation:** Notice how it is impossible to interleave s2 with any other string to obtain s3.

```


**Example 3:**

```

**Input:** s1 = &quot;&quot;, s2 = &quot;&quot;, s3 = &quot;&quot;
**Output:** true

```


 

**Constraints:**



	- `0 &lt;= s1.length, s2.length &lt;= 100`
	- `0 &lt;= s3.length &lt;= 200`
	- `s1`, `s2`, and `s3` consist of lowercase English letters.



 

**Follow up:** Could you solve it using only `O(s2.length)` additional memory space?
