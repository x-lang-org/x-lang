#!/usr/bin/env python3
"""
Fix case.json files by extracting correct test cases from problem content.
"""
import os
import json
import re
import html

BASE_DIR = os.path.dirname(os.path.abspath(__file__))
PROBLEMS_DIR = os.path.join(BASE_DIR, "problems")
SOLUTIONS_DIR = os.path.join(BASE_DIR, "solutions")

def parse_example_testcases(content):
    """Parse example test cases from problem content HTML."""
    testcases = []

    # Find all example blocks
    # Pattern: <strong class="example">Example N:</strong>...<strong>Input:</strong>...<strong>Output:</strong>...
    example_pattern = r'<strong class="example">Example \d+:</strong>.*?<pre>(.*?)</pre>'
    examples = re.findall(example_pattern, content, re.DOTALL | re.IGNORECASE)

    for example in examples:
        # Parse input and output
        input_match = re.search(r'<strong>Input:</strong>\s*(.*?)(?:<strong>Output:</strong>|$)', example, re.DOTALL)
        output_match = re.search(r'<strong>Output:</strong>\s*(.*?)(?:<strong>|$)', example, re.DOTALL)

        if input_match and output_match:
            input_text = html.unescape(input_match.group(1).strip())
            output_text = html.unescape(output_match.group(1).strip())

            # Clean up the input/output
            input_text = input_text.strip().strip('\n')
            output_text = output_text.strip().strip('\n')

            # Try to parse as JSON if possible
            try:
                # Handle various formats
                if input_text.startswith('[') or input_text.startswith('{'):
                    input_val = json.loads(input_text.split('\n')[0])
                else:
                    # Could be "a = 11, b = 1" format
                    # Try to extract values
                    assignments = {}
                    for match in re.finditer(r'(\w+)\s*=\s*([^\n,]+)', input_text):
                        key = match.group(1)
                        val = match.group(2).strip().strip('"')
                        try:
                            assignments[key] = int(val)
                        except:
                            assignments[key] = val

                    if assignments:
                        if len(assignments) == 1:
                            input_val = list(assignments.values())[0]
                        else:
                            input_val = assignments
                    else:
                        input_val = input_text.strip('"')

                # Parse output
                output_val = output_text.strip('"')

            except:
                input_val = input_text.strip('"')
                output_val = output_text.strip('"')

            testcases.append({
                "input": input_val if not isinstance(input_val, str) else str(input_val),
                "expectedOutput": str(output_val)
            })

    return testcases

def fix_case_json(slug, difficulty):
    """Fix case.json for a specific problem."""
    # Read problem file
    problem_path = os.path.join(PROBLEMS_DIR, difficulty, f"{slug}.json")
    if not os.path.exists(problem_path):
        print(f"Problem file not found: {problem_path}")
        return False

    with open(problem_path, 'r', encoding='utf-8') as f:
        problem_data = json.load(f)

    detail = problem_data.get('detail', {})
    content = detail.get('content', '')

    if not content:
        print(f"No content for {slug}")
        return False

    # Parse test cases from content
    testcases = parse_example_testcases(content)

    if not testcases:
        print(f"No test cases found for {slug}")
        return False

    # Create case.json
    case_data = {
        "problemId": detail.get('questionId', ''),
        "titleSlug": slug,
        "title": problem_data.get('title', ''),
        "titleCn": problem_data.get('titleCn', ''),
        "difficulty": problem_data.get('difficulty', '').upper(),
        "topicTags": problem_data.get('topicTags', []),
        "acRate": problem_data.get('acRate', 0),
        "testcases": testcases
    }

    # Write to solutions directory
    case_path = os.path.join(SOLUTIONS_DIR, difficulty, slug, "case.json")
    os.makedirs(os.path.dirname(case_path), exist_ok=True)

    with open(case_path, 'w', encoding='utf-8') as f:
        json.dump(case_data, f, indent=2, ensure_ascii=False)

    print(f"Fixed: {slug} with {len(testcases)} test cases")
    return True

def main():
    fixed = 0
    errors = 0

    for difficulty in ["easy", "medium", "hard"]:
        problems_dir = os.path.join(PROBLEMS_DIR, difficulty)
        if not os.path.exists(problems_dir):
            continue

        for filename in os.listdir(problems_dir):
            if filename.endswith('.json'):
                slug = filename[:-5]  # Remove .json
                if fix_case_json(slug, difficulty):
                    fixed += 1
                else:
                    errors += 1

    print(f"\nFixed {fixed} case.json files, {errors} errors")

if __name__ == "__main__":
    main()
