# 字符串转换整数 (atoi) (String to Integer (atoi))

## 题目信息

- **难度**: MEDIUM
- **通过率**: 21.9%
- **标签**: `string`
- **LeetCode**: https://leetcode.cn/problems/string-to-integer-atoi/

## 题目描述

Implement the `myAtoi(string s)` function, which converts a string to a 32-bit signed integer.


The algorithm for `myAtoi(string s)` is as follows:

<ol>
	- **Whitespace**: Ignore any leading whitespace (`&quot; &quot;`).
	- **Signedness**: Determine the sign by checking if the next character is `&#39;-&#39;` or `&#39;+&#39;`, assuming positivity if neither present.
	- **Conversion**: Read the integer by skipping leading zeros until a non-digit character is encountered or the end of the string is reached. If no digits were read, then the result is 0.
	- **Rounding**: If the integer is out of the 32-bit signed integer range `[-2<sup>31</sup>, 2<sup>31</sup> - 1]`, then round the integer to remain in the range. Specifically, integers less than `-2<sup>31</sup>` should be rounded to `-2<sup>31</sup>`, and integers greater than `2<sup>31</sup> - 1` should be rounded to `2<sup>31</sup> - 1`.
</ol>


Return the integer as the final result.


 

**Example 1:**

<div class="example-block">

**Input:** <span class="example-io">s = &quot;42&quot;</span>


**Output:** <span class="example-io">42</span>


**Explanation:**

```

The underlined characters are what is read in and the caret is the current reader position.
Step 1: &quot;42&quot; (no characters read because there is no leading whitespace)
         ^
Step 2: &quot;42&quot; (no characters read because there is neither a &#39;-&#39; nor &#39;+&#39;)
         ^
Step 3: &quot;<u>42</u>&quot; (&quot;42&quot; is read in)
           ^

```
</div>


**Example 2:**

<div class="example-block">

**Input:** <span class="example-io">s = &quot; -042&quot;</span>


**Output:** <span class="example-io">-42</span>


**Explanation:**

```

Step 1: &quot;<u>   </u>-042&quot; (leading whitespace is read and ignored)
            ^
Step 2: &quot;   <u>-</u>042&quot; (&#39;-&#39; is read, so the result should be negative)
             ^
Step 3: &quot;   -<u>042</u>&quot; (&quot;042&quot; is read in, leading zeros ignored in the result)
               ^

```
</div>


**Example 3:**

<div class="example-block">

**Input:** <span class="example-io">s = &quot;1337c0d3&quot;</span>


**Output:** <span class="example-io">1337</span>


**Explanation:**

```

Step 1: &quot;1337c0d3&quot; (no characters read because there is no leading whitespace)
         ^
Step 2: &quot;1337c0d3&quot; (no characters read because there is neither a &#39;-&#39; nor &#39;+&#39;)
         ^
Step 3: &quot;<u>1337</u>c0d3&quot; (&quot;1337&quot; is read in; reading stops because the next character is a non-digit)
             ^

```
</div>


**Example 4:**

<div class="example-block">

**Input:** <span class="example-io">s = &quot;0-1&quot;</span>


**Output:** <span class="example-io">0</span>


**Explanation:**

```

Step 1: &quot;0-1&quot; (no characters read because there is no leading whitespace)
         ^
Step 2: &quot;0-1&quot; (no characters read because there is neither a &#39;-&#39; nor &#39;+&#39;)
         ^
Step 3: &quot;<u>0</u>-1&quot; (&quot;0&quot; is read in; reading stops because the next character is a non-digit)
          ^

```
</div>


**Example 5:**

<div class="example-block">

**Input:** <span class="example-io">s = &quot;words and 987&quot;</span>


**Output:** <span class="example-io">0</span>


**Explanation:**


Reading stops at the first non-digit character &#39;w&#39;.
</div>


 

**Constraints:**



	- `0 &lt;= s.length &lt;= 200`
	- `s` consists of English letters (lower-case and upper-case), digits (`0-9`), `&#39; &#39;`, `&#39;+&#39;`, `&#39;-&#39;`, and `&#39;.&#39;`.
