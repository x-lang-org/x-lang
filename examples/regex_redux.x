// regex-redux (Benchmarks Game)
// DNA sequence pattern matching and replacement.
// Reference: https://benchmarksgame-team.pages.debian.net/benchmarksgame/description/regexredux.html

fun main() {
  let input = ">ONE Homo sapiens alu\nGGCCGGGCGCGGTGGCTCACGCCTGTAATCCCAGCACTTTGGGAGGCCGAGGCGGGCGGATCACCTGAGGTCAGGAGTTCGAGACCAGCCTGGCCAACATGGTGAAACCCCGTCTCTACTAAAAATACAAAAATTAGCCGGGCGTGGTGGCGCGCGCCTGTAATCCCAGCTACTCGGGAGGCTGAGGCAGGAGAATCGCTTGAACCCGGGAGGCGGAGGTTGCAGTGAGCCGAGATCGCGCCACTGCACTCCAGCCTGGGCGACAGAGCGAGACTCCGTCTCAAAAA\n>TWO IUB ambiguity codes\ncttBtatcatatgctaKggNcataaaSatgtaaaDcDRtBggDtctttataattcBgtcg\n>THREE Homo sapiens frequency\naatattatgataatttcccaacttatttatag"

  // Strip headers and newlines
  var seq = ""
  let lines = str_split(input, "\n")
  var i = 0
  while i < len(lines) {
    let line = lines[i]
    if !str_starts_with(line, ">") {
      seq = concat(seq, line)
    }
    i = i + 1
  }

  let original_len = len(seq)

  // Count matches for each of the 9 patterns
  let patterns = [
    "agggtaaa|tttaccct",
    "[cgt]gggtaaa|tttaccc[acg]",
    "a[act]ggtaaa|tttacc[agt]t",
    "ag[act]gtaaa|tttac[agt]ct",
    "agg[act]taaa|ttta[agt]cct",
    "aggg[acg]aaa|ttt[cgt]ccct",
    "agggt[cgt]aa|tt[acg]accct",
    "agggta[cgt]a|t[acg]taccct",
    "agggtaa[cgt]|[acg]ttaccct"
  ]

  var p = 0
  while p < len(patterns) {
    let pat = patterns[p]
    let count = regex_match_count(seq, pat)
    print(concat(pat, concat(" ", to_string(count))))
    p = p + 1
  }

  // IUB code replacements (single-character IUPAC to regex alternatives)
  var result = seq
  result = str_replace(result, "B", "(c|g|t)")
  result = str_replace(result, "D", "(a|g|t)")
  result = str_replace(result, "H", "(a|c|t)")
  result = str_replace(result, "K", "(g|t)")
  result = str_replace(result, "M", "(a|c)")
  result = str_replace(result, "N", "(a|c|g|t)")
  result = str_replace(result, "R", "(a|g)")
  result = str_replace(result, "S", "(c|g)")
  result = str_replace(result, "V", "(a|c|g)")
  result = str_replace(result, "W", "(a|t)")
  result = str_replace(result, "Y", "(c|t)")

  print("")
  print(to_string(original_len))
  print(to_string(len(seq)))
  print(to_string(len(result)))
}
