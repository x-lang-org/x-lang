// pidigits (Benchmarks Game)
// Compute N digits of Pi using the spigot algorithm with big integers.
// Reference: https://benchmarksgame-team.pages.debian.net/benchmarksgame/description/pidigits.html

fun main() {
  let n = 27
  let digits = compute_pi_digits(n)

  var printed = 0
  var line = ""

  while printed < n {
    let remaining = n - printed
    var take = 10
    if take > remaining {
      take = remaining
    }

    line = substring(digits, printed, printed + take)
    printed = printed + take

    // Pad to 10 chars if needed
    var padded = line
    while len(padded) < 10 {
      padded = concat(padded, " ")
    }

    print(concat(padded, concat("\t:", to_string(printed))))
  }
}
