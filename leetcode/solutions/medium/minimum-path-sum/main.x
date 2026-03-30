// Minimum Path Sum - LeetCode 64
// Grid: [[1,3,1],[1,5,1],[4,2,1]]

function main() -> integer {
    // DP: dp[i][j] = min(dp[i-1][j], dp[i][j-1]) + grid[i][j]

    // Row 0: accumulate
    let r0c0 = 1
    let r0c1 = r0c0 + 3
    let r0c2 = r0c1 + 1

    // Row 1
    let r1c0 = r0c0 + 1

    // dp[1][1]: min of r0c1, r1c0 + grid[1][1]
    let v11_min = r0c1
    if r1c0 < v11_min {
        v11_min = r1c0
    }
    let r1c1 = v11_min + 5

    // dp[1][2]: min of r0c2, r1c1 + grid[1][2]
    let v12_min = r0c2
    if r1c1 < v12_min {
        v12_min = r1c1
    }
    let r1c2 = v12_min + 1

    // Row 2
    let r2c0 = r1c0 + 4

    // dp[2][1]
    let v21_min = r1c1
    if r2c0 < v21_min {
        v21_min = r2c0
    }
    let r2c1 = v21_min + 2

    // dp[2][2]
    let v22_min = r1c2
    if r2c1 < v22_min {
        v22_min = r2c1
    }
    let r2c2 = v22_min + 1

    println(r2c2)
    return 0
}
