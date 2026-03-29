#!/usr/bin/env node
/**
 * Fetch detailed information for all problems from leetcode.cn
 * Each problem is saved as a separate JSON file
 */

import { LeetCodeServiceFactory } from "../leetcode-mcp-server/build/leetcode/leetcode-service-factory.js";
import fs from 'node:fs';
import path from 'node:path';

async function fetchAllDetails() {
    console.log("Starting to fetch problem details from leetcode.cn...");

    // Load the problem list
    const problemsFile = path.join(process.cwd(), 'problems_cn.json');
    if (!fs.existsSync(problemsFile)) {
        console.error("problems_cn.json not found. Run fetch_all.js first.");
        process.exit(1);
    }

    const { problems } = JSON.parse(fs.readFileSync(problemsFile, 'utf-8'));
    console.log(`Found ${problems.length} problems. Starting to fetch details...`);

    // Create service
    const service = await LeetCodeServiceFactory.createService("cn");

    // Create problems directory
    const problemsDir = path.join(process.cwd(), 'problems');
    if (!fs.existsSync(problemsDir)) {
        fs.mkdirSync(problemsDir);
    }

    // Create difficulty subdirectories
    for (const diff of ['easy', 'medium', 'hard']) {
        const dir = path.join(problemsDir, diff);
        if (!fs.existsSync(dir)) {
            fs.mkdirSync(dir);
        }
    }

    let success = 0;
    let failed = 0;
    let skipped = 0;

    // Fetch each problem
    for (let i = 0; i < problems.length; i++) {
        const problem = problems[i];
        const difficulty = problem.difficulty.toLowerCase();
        const outputPath = path.join(problemsDir, difficulty, `${problem.titleSlug}.json`);

        // Skip if already fetched
        if (fs.existsSync(outputPath)) {
            skipped++;
            if (skipped % 100 === 0) {
                console.log(`${i + 1}/${problems.length} - Skipped ${skipped} (already exists)`);
            }
            continue;
        }

        try {
            console.log(`${i + 1}/${problems.length} - Fetching ${problem.titleSlug}...`);
            const detail = await service.fetchProblemSimplified(problem.titleSlug);

            // Add metadata
            const fullData = {
                ...problem,
                detail,
                fetchedAt: new Date().toISOString()
            };

            fs.writeFileSync(outputPath, JSON.stringify(fullData, null, 2));
            success++;

            // Rate limiting - delay to avoid being blocked (faster)
            await new Promise(resolve => setTimeout(resolve, 200));
        } catch (error) {
            console.error(`Failed to fetch ${problem.titleSlug}:`, error.message);
            failed++;
            // Longer delay on failure
            await new Promise(resolve => setTimeout(resolve, 3000));
        }
    }

    console.log(`\nFinished!
  - Success: ${success}
  - Failed:  ${failed}
  - Skipped: ${skipped}
  - Total:   ${success + failed + skipped}
`);

    // Generate an index with all detailed problems
    const allDetailed = [];
    for (const diff of ['easy', 'medium', 'hard']) {
        const dir = path.join(problemsDir, diff);
        if (fs.existsSync(dir)) {
            const files = fs.readdirSync(dir);
            files.forEach(file => {
                if (file.endsWith('.json')) {
                    const data = JSON.parse(fs.readFileSync(path.join(dir, file), 'utf-8'));
                    allDetailed.push({
                        titleSlug: data.titleSlug,
                        title: data.title,
                        titleCn: data.titleCn,
                        difficulty: data.difficulty,
                        acRate: data.acRate,
                        topicTags: data.topicTags,
                        path: path.join('problems', diff, file)
                    });
                }
            });
        }
    }

    allDetailed.sort((a, b) => a.titleSlug.localeCompare(b.titleSlug));

    fs.writeFileSync(
        path.join(process.cwd(), 'index_detailed.json'),
        JSON.stringify({
            total: allDetailed.length,
            generatedAt: new Date().toISOString(),
            problems: allDetailed
        }, null, 2)
    );

    console.log(`Generated detailed index: index_detailed.json (${allDetailed.length} problems)`);
}

fetchAllDetails().catch(error => {
    console.error("Error fetching problem details:", error);
    process.exit(1);
});
