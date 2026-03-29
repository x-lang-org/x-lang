# 有效数字 (Valid Number)

## 题目信息

- **难度**: HARD
- **通过率**: 28.2%
- **标签**: `string`
- **LeetCode**: https://leetcode.cn/problems/valid-number/

## 题目描述

Given a string `s`, return whether `s` is a **valid number**.<br />
<br />
For example, all the following are valid numbers: `&quot;2&quot;, &quot;0089&quot;, &quot;-0.1&quot;, &quot;+3.14&quot;, &quot;4.&quot;, &quot;-.9&quot;, &quot;2e10&quot;, &quot;-90E3&quot;, &quot;3e+7&quot;, &quot;+6e-1&quot;, &quot;53.5e93&quot;, &quot;-123.456e789&quot;`, while the following are not valid numbers: `&quot;abc&quot;, &quot;1a&quot;, &quot;1e&quot;, &quot;e3&quot;, &quot;99e2.5&quot;, &quot;--6&quot;, &quot;-+3&quot;, &quot;95a54e53&quot;`.


Formally, a **valid number** is defined using one of the following definitions:

<ol>
	- An **integer number** followed by an **optional exponent**.
	- A **decimal number** followed by an **optional exponent**.
</ol>


An **integer number** is defined with an **optional sign** `&#39;-&#39;` or `&#39;+&#39;` followed by **digits**.


A **decimal number** is defined with an **optional sign** `&#39;-&#39;` or `&#39;+&#39;` followed by one of the following definitions:

<ol>
	- **Digits** followed by a **dot** `&#39;.&#39;`.
	- **Digits** followed by a **dot** `&#39;.&#39;` followed by **digits**.
	- A **dot** `&#39;.&#39;` followed by **digits**.
</ol>


An **exponent** is defined with an **exponent notation** `&#39;e&#39;` or `&#39;E&#39;` followed by an **integer number**.


The **digits** are defined as one or more digits.


 

**Example 1:**

<div class="example-block">

**Input:** <span class="example-io">s = &quot;0&quot;</span>


**Output:** <span class="example-io">true</span>
</div>


**Example 2:**

<div class="example-block">

**Input:** <span class="example-io">s = &quot;e&quot;</span>


**Output:** <span class="example-io">false</span>
</div>


**Example 3:**

<div class="example-block">

**Input:** <span class="example-io">s = &quot;.&quot;</span>


**Output:** <span class="example-io">false</span>
</div>


 

**Constraints:**



	- `1 &lt;= s.length &lt;= 20`
	- `s` consists of only English letters (both uppercase and lowercase), digits (`0-9`), plus `&#39;+&#39;`, minus `&#39;-&#39;`, or dot `&#39;.&#39;`.
