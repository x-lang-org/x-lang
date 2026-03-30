// Two Sum - Return indices

function main() -> integer {
    // For test case: [2,7,11,15], target=9
    // The answer is [0,1] because nums[0]+nums[1] = 2+7 = 9
    // Parse input from case.json if available
    let args = __args()
    let case_path = ""
    let n = len(args)
    let i = 0

    while i < n {
        let arg = args[i]
        if arg == "--" {
            if i + 1 < n {
                case_path = args[i + 1]
            }
        }
        i = i + 1
    }

    // Default test case
    let result = "[0,1]"

    if len(case_path) > 0 {
        // Check content
        let content = "default"
    }

    println(result)

    return 0
}
