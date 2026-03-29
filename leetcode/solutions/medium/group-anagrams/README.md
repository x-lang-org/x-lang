# 字母异位词分组 (Group Anagrams)

## 题目信息

- **难度**: MEDIUM
- **通过率**: 69.6%
- **标签**: `array`, `hash-table`, `string`, `sorting`
- **LeetCode**: https://leetcode.cn/problems/group-anagrams/

## 题目描述

Given an array of strings `strs`, group the <span data-keyword="anagram">anagrams</span> together. You can return the answer in **any order**.


 

**Example 1:**

<div class="example-block">

**Input:** <span class="example-io">strs = [&quot;eat&quot;,&quot;tea&quot;,&quot;tan&quot;,&quot;ate&quot;,&quot;nat&quot;,&quot;bat&quot;]</span>


**Output:** <span class="example-io">[[&quot;bat&quot;],[&quot;nat&quot;,&quot;tan&quot;],[&quot;ate&quot;,&quot;eat&quot;,&quot;tea&quot;]]</span>


**Explanation:**



	- There is no string in strs that can be rearranged to form `&quot;bat&quot;`.
	- The strings `&quot;nat&quot;` and `&quot;tan&quot;` are anagrams as they can be rearranged to form each other.
	- The strings `&quot;ate&quot;`, `&quot;eat&quot;`, and `&quot;tea&quot;` are anagrams as they can be rearranged to form each other.

</div>


**Example 2:**

<div class="example-block">

**Input:** <span class="example-io">strs = [&quot;&quot;]</span>


**Output:** <span class="example-io">[[&quot;&quot;]]</span>
</div>


**Example 3:**

<div class="example-block">

**Input:** <span class="example-io">strs = [&quot;a&quot;]</span>


**Output:** <span class="example-io">[[&quot;a&quot;]]</span>
</div>


 

**Constraints:**



	- `1 &lt;= strs.length &lt;= 10<sup>4</sup>`
	- `0 &lt;= strs[i].length &lt;= 100`
	- `strs[i]` consists of lowercase English letters.
