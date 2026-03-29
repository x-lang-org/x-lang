# 串联所有单词的子串 (Substring with Concatenation of All Words)

## 题目信息

- **难度**: HARD
- **通过率**: 38.2%
- **标签**: `hash-table`, `string`, `sliding-window`
- **LeetCode**: https://leetcode.cn/problems/substring-with-concatenation-of-all-words/

## 题目描述

You are given a string `s` and an array of strings `words`. All the strings of `words` are of **the same length**.


A **concatenated string** is a string that exactly contains all the strings of any permutation of `words` concatenated.



	- For example, if `words = [&quot;ab&quot;,&quot;cd&quot;,&quot;ef&quot;]`, then `&quot;abcdef&quot;`, `&quot;abefcd&quot;`, `&quot;cdabef&quot;`, `&quot;cdefab&quot;`, `&quot;efabcd&quot;`, and `&quot;efcdab&quot;` are all concatenated strings. `&quot;acdbef&quot;` is not a concatenated string because it is not the concatenation of any permutation of `words`.



Return an array of <em>the starting indices</em> of all the concatenated substrings in `s`. You can return the answer in **any order**.


 

**Example 1:**

<div class="example-block">

**Input:** <span class="example-io">s = &quot;barfoothefoobarman&quot;, words = [&quot;foo&quot;,&quot;bar&quot;]</span>


**Output:** <span class="example-io">[0,9]</span>


**Explanation:**


The substring starting at 0 is `&quot;barfoo&quot;`. It is the concatenation of `[&quot;bar&quot;,&quot;foo&quot;]` which is a permutation of `words`.<br />
The substring starting at 9 is `&quot;foobar&quot;`. It is the concatenation of `[&quot;foo&quot;,&quot;bar&quot;]` which is a permutation of `words`.
</div>


**Example 2:**

<div class="example-block">

**Input:** <span class="example-io">s = &quot;wordgoodgoodgoodbestword&quot;, words = [&quot;word&quot;,&quot;good&quot;,&quot;best&quot;,&quot;word&quot;]</span>


**Output:** <span class="example-io">[]</span>


**Explanation:**


There is no concatenated substring.
</div>


**Example 3:**

<div class="example-block">

**Input:** <span class="example-io">s = &quot;barfoofoobarthefoobarman&quot;, words = [&quot;bar&quot;,&quot;foo&quot;,&quot;the&quot;]</span>


**Output:** <span class="example-io">[6,9,12]</span>


**Explanation:**


The substring starting at 6 is `&quot;foobarthe&quot;`. It is the concatenation of `[&quot;foo&quot;,&quot;bar&quot;,&quot;the&quot;]`.<br />
The substring starting at 9 is `&quot;barthefoo&quot;`. It is the concatenation of `[&quot;bar&quot;,&quot;the&quot;,&quot;foo&quot;]`.<br />
The substring starting at 12 is `&quot;thefoobar&quot;`. It is the concatenation of `[&quot;the&quot;,&quot;foo&quot;,&quot;bar&quot;]`.
</div>


 

**Constraints:**



	- `1 &lt;= s.length &lt;= 10<sup>4</sup>`
	- `1 &lt;= words.length &lt;= 5000`
	- `1 &lt;= words[i].length &lt;= 30`
	- `s` and `words[i]` consist of lowercase English letters.
