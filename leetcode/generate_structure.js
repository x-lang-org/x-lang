#!/usr/bin/env node
/**
 * Generate problem solution structure for leetcode
 * Creates:
 *   solutions/<difficulty>/<titleSlug>/README.md - problem description
 *   solutions/<difficulty>/<titleSlug>/main.x - X language solution template
 *   solutions/<difficulty>/<titleSlug>/case.json - test cases
 */

import fs from 'node:fs';
import path from 'node:path';

function generateREADME(problem) {
  const { title, titleCn, difficulty, acRate, topicTags, detail } = problem;

  let tags = topicTags.map(tag => `\`${tag}\``).join(', ');

  let content = `# ${titleCn} (${title})

## 题目信息

- **难度**: ${difficulty}
- **通过率**: ${(acRate * 100).toFixed(1)}%
- **标签**: ${tags}

## 题目描述

${detail.content.replace(/<p>/g, '\n').replace(/<\/p>/g, '').replace(/<ul>/g, '\n').replace(/<\/ul>/g, '').replace(/<li>/g, '- ').replace(/<\/li>/g, '').replace(/<pre>/g, '```\n').replace(/<\/pre>/g, '\n```').replace(/<code>/g, '`').replace(/<\/code>/g, '`').replace(/<strong[^>]*>/g, '**').replace(/<\/strong>/g, '**').trim()}

## 测试用例

See case.json
`;

  return content;
}

function generateMainX(problem) {
  return `// ${problem.titleCn}
// https://leetcode.cn/problems/${problem.titleSlug}/

needs stdio

// TODO: 实现你的解法
`;
}

function parseExampleTestcases(content) {
  if (!content) return [];
  const lines = content.trim().split('\n').filter(line => line.trim());
  const cases = [];

  // leetcode stores one line per input/output value
  // Group all lines into test cases where each test case has N lines for input + 1 line output
  // For simplicity, we pair them as (input line, output line) for each example
  for (let i = 0; i < lines.length; i += 2) {
    cases.push({
      input: lines[i],
      expectedOutput: i + 1 < lines.length ? lines[i + 1] : ''
    });
  }

  return cases;
}

function generateCaseJSON(problem) {
  const examples = parseExampleTestcases(problem.detail.exampleTestcases);

  return JSON.stringify({
    problemId: problem.detail.questionId,
    titleSlug: problem.titleSlug,
    title: problem.title,
    titleCn: problem.titleCn,
    difficulty: problem.difficulty,
    testcases: examples
  }, null, 2) + '\n';
}

async function generateProblem(titleSlug) {
  // Find the problem file
  const problemsDir = path.join(process.cwd(), 'problems');
  let foundPath = null;
  let problemData = null;

  for (const diff of ['easy', 'medium', 'hard']) {
    const filePath = path.join(problemsDir, diff, `${titleSlug}.json`);
    if (fs.existsSync(filePath)) {
      foundPath = filePath;
      problemData = JSON.parse(fs.readFileSync(filePath, 'utf-8'));
      break;
    }
  }

  if (!foundPath) {
    console.error(`Problem ${titleSlug} not found`);
    return false;
  }

  const difficulty = problemData.difficulty.toLowerCase();
  const outputDir = path.join(process.cwd(), 'solutions', difficulty, titleSlug);

  if (!fs.existsSync(outputDir)) {
    fs.mkdirSync(outputDir, { recursive: true });
  }

  // Write README.md
  fs.writeFileSync(
    path.join(outputDir, 'README.md'),
    generateREADME(problemData)
  );

  // Write main.x
  fs.writeFileSync(
    path.join(outputDir, 'main.x'),
    generateMainX(problemData)
  );

  // Write case.json
  fs.writeFileSync(
    path.join(outputDir, 'case.json'),
    generateCaseJSON(problemData)
  );

  console.log(`Generated: ${outputDir}`);
  return true;
}

async function main() {
  const args = process.argv.slice(2);

  if (args.length === 0) {
    console.log('Usage: node generate_structure.js <titleSlug> [titleSlug...]');
    console.log('Example: node generate_structure.js two-sum add-two-numbers');
    process.exit(1);
  }

  let success = 0;
  for (const slug of args) {
    if (await generateProblem(slug)) {
      success++;
    }
  }

  console.log(`\nCompleted: ${success}/${args.length} problems generated`);
}

main().catch(error => {
  console.error('Error:', error);
  process.exit(1);
});
