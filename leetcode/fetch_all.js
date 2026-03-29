#!/usr/bin/env node
/**
 * Fetch all problems from leetcode.cn using search_problems API
 * and save them to problems_cn.json
 */

import { LeetCodeServiceFactory } from "../leetcode-mcp-server/build/leetcode/leetcode-service-factory.js";
import fs from 'node:fs';
import path from 'node:path';

const BATCH_SIZE = 50;

async function fetchAllProblems() {
    console.log("Starting to fetch all problems from leetcode.cn...");

    // Create service for leetcode.cn (no authentication needed for public search)
    const service = await LeetCodeServiceFactory.createService("cn");

    // First fetch to get total
    console.log(`Fetching first batch (${BATCH_SIZE} problems)...`);
    const firstResult = await service.searchProblems(
        "all-code-essentials",
        undefined,
        undefined,
        BATCH_SIZE,
        0
    );

    const total = firstResult.total;
    console.log(`Total problems: ${total}. Fetching remaining...`);

    const allQuestions = [...firstResult.questions];
    let hasMore = firstResult.hasMore;
    let offset = BATCH_SIZE;

    while (hasMore && offset < total) {
        console.log(`Fetching ${offset} / ${total}...`);
        const result = await service.searchProblems(
            "all-code-essentials",
            undefined,
            undefined,
            BATCH_SIZE,
            offset
        );

        allQuestions.push(...result.questions);
        hasMore = result.hasMore;
        offset += BATCH_SIZE;

        // Rate limiting
        await new Promise(resolve => setTimeout(resolve, 200));
    }

    console.log(`Finished! Total ${allQuestions.length} problems fetched.`);

    // Sort by titleSlug
    allQuestions.sort((a, b) => a.titleSlug.localeCompare(b.titleSlug));

    // Create output directory structure
    for (const dir of ['easy', 'medium', 'hard']) {
        const dirPath = path.join(process.cwd(), dir);
        if (!fs.existsSync(dirPath)) {
            fs.mkdirSync(dirPath);
        }
    }

    // Save full list
    const output = {
        total: allQuestions.length,
        fetchedAt: new Date().toISOString(),
        problems: allQuestions
    };

    fs.writeFileSync(
        path.join(process.cwd(), 'problems_cn.json'),
        JSON.stringify(output, null, 2)
    );

    // Save by difficulty
    const byDifficulty = { EASY: [], MEDIUM: [], HARD: [] };
    allQuestions.forEach(q => {
        byDifficulty[q.difficulty].push(q);
    });

    for (const [diff, questions] of Object.entries(byDifficulty)) {
        const lower = diff.toLowerCase();
        fs.writeFileSync(
            path.join(process.cwd(), lower, `index.json`),
            JSON.stringify(questions, null, 2)
        );
        console.log(`Saved ${questions.length} ${diff} problems to ${lower}/index.json`);
    }

    console.log(`\nAll done! Saved to problems_cn.json`);
    console.log(`Summary:
  - Easy:   ${byDifficulty.EASY.length}
  - Medium: ${byDifficulty.MEDIUM.length}
  - Hard:   ${byDifficulty.HARD.length}
  - Total:  ${allQuestions.length}`);
}

fetchAllProblems().catch(error => {
    console.error("Error fetching problems:", error);
    process.exit(1);
});
