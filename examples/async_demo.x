// Async demo for X language
// Demonstrates async functions and wait expressions

// Async function that returns a future
async function fetchData(id: Int) -> Async<String> {
    return "data";
}

// Function using single wait
function singleWait() {
    let result = wait fetchData(42);
    println(result);
}

// Function using wait together for parallel execution
function parallelFetch() {
    wait together {
        fetchData(1),
        fetchData(2),
        fetchData(3)
    };
}

// Function using wait race for competitive execution
function raceFetch() {
    wait race {
        fetchData(1),
        fetchData(2)
    };
}

// Function using wait timeout
function fetchWithTimeout() {
    wait timeout(5000) {
        fetchData(42)
    };
}

// Main entry point
function main() {
    singleWait();
    parallelFetch();
    raceFetch();
    fetchWithTimeout();
}
