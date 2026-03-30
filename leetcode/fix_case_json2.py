#!/usr/bin/env python3
"""
Better case.json generator that parses problem content properly.
"""
import os
import json
import re
import html

BASE_DIR = os.path.dirname(os.path.abspath(__file__))
PROBLEMS_DIR = os.path.join(BASE_DIR, "problems")
SOLUTIONS_DIR = os.path.join(BASE_DIR, "solutions")

def extract_number(s):
    """Extract first number from string."""
    match = re.search(r'\d+', s)
    return match.group(0) if match else s

def extract_array(s):
    """Extract array from string like '[1,2,3]'."""
    # Try to parse as JSON array
    try:
        return json.loads(s)
    except:
        pass
    return None

def parse_test_cases(content):
    """Parse test cases from problem content."""
    testcases = []

    # Find all <pre> blocks that contain Input/Output
    pre_blocks = re.findall(r'<pre>(.*?)</pre>', content, re.DOTALL)

    inputs = []
    outputs = []

    for block in pre_blocks:
        if 'Input:' in block and 'Output:' in block:
            # Extract input
            input_match = re.search(r'Input:\s*(.+)', block, re.DOTALL)
            # Extract output
            output_match = re.search(r'Output:\s*(.+)', block, re.DOTALL)

            if input_match and output_match:
                inp = html.unescape(input_match.group(1).strip())
                out = html.unescape(output_match.group(1).strip())
                inputs.append(inp)
                outputs.append(out)

    # Also check exampleTestcases
    if not inputs:
        return []

    for inp, out in zip(inputs, outputs):
        # Clean up the values
        inp_clean = inp.strip().replace('\n', ' ').replace('  ', ' ')
        out_clean = out.strip().replace('\n', ' ').replace('  ', ' ')

        # Try to parse input as array
        parsed_input = extract_array(inp_clean)

        # Determine expected output
        expected = out_clean.strip('"')

        testcases.append({
            "input": parsed_input if parsed_input else inp_clean,
            "expectedOutput": expected
        })

    return testcases

def fix_case_json(slug, difficulty):
    """Fix case.json for a specific problem."""
    problem_path = os.path.join(PROBLEMS_DIR, difficulty, f"{slug}.json")
    if not os.path.exists(problem_path):
        return False

    with open(problem_path, 'r', encoding='utf-8') as f:
        problem_data = json.load(f)

    detail = problem_data.get('detail', {})
    content = detail.get('content', '')

    if not content:
        return False

    testcases = parse_test_cases(content)

    if not testcases:
        return False

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

    case_path = os.path.join(SOLUTIONS_DIR, difficulty, slug, "case.json")
    os.makedirs(os.path.dirname(case_path), exist_ok=True)

    with open(case_path, 'w', encoding='utf-8') as f:
        json.dump(case_data, f, indent=2, ensure_ascii=False)

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
                slug = filename[:-5]
                if fix_case_json(slug, difficulty):
                    fixed += 1
                else:
                    errors += 1

    print(f"Fixed {fixed} case.json files, {errors} errors")

if __name__ == "__main__":
    main()
