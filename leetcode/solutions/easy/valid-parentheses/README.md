# 有效的括号 (Valid Parentheses)

## 题目信息

- **难度**: EASY
- **通过率**: 45.5%
- **标签**: `stack`, `string`
- **LeetCode**: https://leetcode.cn/problems/valid-parentheses/

## 题目描述

Given a string `s` containing just the characters `&#39;(&#39;`, `&#39;)&#39;`, `&#39;{&#39;`, `&#39;}&#39;`, `&#39;[&#39;` and `&#39;]&#39;`, determine if the input string is valid.


An input string is valid if:

<ol>
	- Open brackets must be closed by the same type of brackets.
	- Open brackets must be closed in the correct order.
	- Every close bracket has a corresponding open bracket of the same type.
</ol>


 

**Example 1:**

<div class="example-block">

**Input:** <span class="example-io">s = &quot;()&quot;</span>


**Output:** <span class="example-io">true</span>
</div>


**Example 2:**

<div class="example-block">

**Input:** <span class="example-io">s = &quot;()[]{}&quot;</span>


**Output:** <span class="example-io">true</span>
</div>


**Example 3:**

<div class="example-block">

**Input:** <span class="example-io">s = &quot;(]&quot;</span>


**Output:** <span class="example-io">false</span>
</div>


**Example 4:**

<div class="example-block">

**Input:** <span class="example-io">s = &quot;([])&quot;</span>


**Output:** <span class="example-io">true</span>
</div>


**Example 5:**

<div class="example-block">

**Input:** <span class="example-io">s = &quot;([)]&quot;</span>


**Output:** <span class="example-io">false</span>
</div>


 

**Constraints:**



	- `1 &lt;= s.length &lt;= 10<sup>4</sup>`
	- `s` consists of parentheses only `&#39;()[]{}&#39;`.
