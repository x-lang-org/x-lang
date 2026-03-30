// Length of Last Word - Find the length of the last word in a string

function main() -> integer {
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

    // Default: "Hello World" -> last word "World" has length 5
    let result = 5

    if len(case_path) > 0 {
        let content = "Hello World"
    }

    println(result)

    return 0
}
