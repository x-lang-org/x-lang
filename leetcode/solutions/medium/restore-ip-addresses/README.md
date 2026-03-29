# 复原 IP 地址 (Restore IP Addresses)

## 题目信息

- **难度**: MEDIUM
- **通过率**: 62.2%
- **标签**: `string`, `backtracking`
- **LeetCode**: https://leetcode.cn/problems/restore-ip-addresses/

## 题目描述

A **valid IP address** consists of exactly four integers separated by single dots. Each integer is between `0` and `255` (**inclusive**) and cannot have leading zeros.



	- For example, `&quot;0.1.2.201&quot;` and `&quot;192.168.1.1&quot;` are **valid** IP addresses, but `&quot;0.011.255.245&quot;`, `&quot;192.168.1.312&quot;` and `&quot;192.168@1.1&quot;` are **invalid** IP addresses.



Given a string `s` containing only digits, return <em>all possible valid IP addresses that can be formed by inserting dots into </em>`s`. You are **not** allowed to reorder or remove any digits in `s`. You may return the valid IP addresses in **any** order.


 

**Example 1:**

```

**Input:** s = &quot;25525511135&quot;
**Output:** [&quot;255.255.11.135&quot;,&quot;255.255.111.35&quot;]

```


**Example 2:**

```

**Input:** s = &quot;0000&quot;
**Output:** [&quot;0.0.0.0&quot;]

```


**Example 3:**

```

**Input:** s = &quot;101023&quot;
**Output:** [&quot;1.0.10.23&quot;,&quot;1.0.102.3&quot;,&quot;10.1.0.23&quot;,&quot;10.10.2.3&quot;,&quot;101.0.2.3&quot;]

```


 

**Constraints:**



	- `1 &lt;= s.length &lt;= 20`
	- `s` consists of digits only.
