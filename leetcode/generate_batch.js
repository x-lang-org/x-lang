#!/usr/bin/env node
/**
 * Batch generate problem solutions structure
 * Generates problems by problemId from start to end
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
- **LeetCode**: https://leetcode.cn/problems/${problem.titleSlug}/

## 题目描述

${detail.content.replace(/<p>/g, '\n').replace(/<\/p>/g, '').replace(/<ul>/g, '\n').replace(/<\/ul>/g, '').replace(/<li>/g, '- ').replace(/<\/li>/g, '').replace(/<pre>/g, '```\n').replace(/<\/pre>/g, '\n```').replace(/<code>/g, '`').replace(/<\/code>/g, '`').replace(/<strong[^>]*>/g, '**').replace(/<\/strong>/g, '**').replace(/&nbsp;/g, ' ').trim()}
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
    topicTags: problem.topicTags,
    acRate: problem.acRate,
    testcases: examples
  }, null, 2) + '\n';
}

function generateProblem(problemData) {
  const difficulty = problemData.difficulty.toLowerCase();
  const outputDir = path.join(process.cwd(), 'solutions', difficulty, problemData.titleSlug);

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

  return true;
}

async function main() {
  // Read the detailed index
  const indexPath = path.join(process.cwd(), 'index_detailed.json');
  if (!fs.existsSync(indexPath)) {
    console.error('index_detailed.json not found');
    process.exit(1);
  }

  const { problems } = JSON.parse(fs.readFileSync(indexPath, 'utf-8'));

  // Load full details for each problem from problems directory
  const startId = 1;
  const endId = 100;
  console.log(`Generating problems from problemId ${startId} to ${endId}...`);

  let generated = 0;
  let skipped = 0;

  for (const indexEntry of problems) {
    // problem path is: problems/<diff>/<titleSlug>.json (could be \ on Windows)
    const parts = indexEntry.path.split(/[\/\\]/);
    const diff = parts[1];
    const problemPath = path.join(process.cwd(), 'problems', diff, `${indexEntry.titleSlug}.json`);

    if (!fs.existsSync(problemPath)) {
      console.warn(`Warning: ${problemPath} not found, skipping`);
      skipped++;
      continue;
    }

    const fullData = JSON.parse(fs.readFileSync(problemPath, 'utf-8'));
    const questionId = parseInt(fullData.detail.questionId, 10);

    if (isNaN(questionId)) {
      // Some problems have non-numeric IDs (LCof special), skip
      skipped++;
      continue;
    }

    if (questionId >= startId && questionId <= endId) {
      generateProblem(fullData);
      generated++;
      if (generated % 10 === 0) {
        console.log(`Generated ${generated} problems (qid ${questionId}: ${indexEntry.titleSlug})`);
      }
    } else {
      skipped++;
    }
  }

  console.log(`\nBatch generation complete!
  Generated: ${generated}
  Skipped: ${skipped}
  Total: ${generated + skipped}`);
}

main().catch(error => {
  console.error('Error:', error);
  process.exit(1);
});
