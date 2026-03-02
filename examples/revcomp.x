// reverse-complement (Benchmarks Game)
// Read FASTA input and output reverse-complement of each sequence.
// Reference: https://benchmarksgame-team.pages.debian.net/benchmarksgame/description/revcomp.html
// Since we have no stdin, we use an embedded test sequence.

fun complement(ch) {
  if ch == "A" { return "T" }
  if ch == "T" { return "A" }
  if ch == "C" { return "G" }
  if ch == "G" { return "C" }
  if ch == "a" { return "t" }
  if ch == "t" { return "a" }
  if ch == "c" { return "g" }
  if ch == "g" { return "c" }
  if ch == "M" { return "K" }
  if ch == "K" { return "M" }
  if ch == "R" { return "Y" }
  if ch == "Y" { return "R" }
  if ch == "W" { return "W" }
  if ch == "S" { return "S" }
  if ch == "B" { return "V" }
  if ch == "V" { return "B" }
  if ch == "D" { return "H" }
  if ch == "H" { return "D" }
  if ch == "N" { return "N" }
  return ch
}

fun reverse_complement(seq) {
  let n = len(seq)
  var result = ""
  var i = n - 1
  while i >= 0 {
    result = concat(result, complement(char_at(seq, i)))
    i = i - 1
  }
  return result
}

fun print_seq(seq) {
  let n = len(seq)
  var pos = 0
  while pos < n {
    var end = pos + 60
    if end > n {
      end = n
    }
    print(substring(seq, pos, end))
    pos = pos + 60
  }
}

fun main() {
  // Test with the first part of the standard revcomp input
  let header1 = ">ONE Homo sapiens alu"
  let seq1 = "GGCCGGGCGCGGTGGCTCACGCCTGTAATCCCAGCACTTTGGGAGGCCGAGGCGGGCGGATCACCTGAGGTCAGGAGTTCGAGACCAGCCTGGCCAACATGGTGAAACCCCGTCTCTACTAAAAATACAAAAATTAGCCGGGCGTGGTGGCGCGCGCCTGTAATCCCAGCTACTCGGGAGGCTGAGGCAGGAGAATCGCTTGAACCCGGGAGGCGGAGGTTGCAGTGAGCCGAGATCGCGCCACTGCACTCCAGCCTGGGCGACAGAGCGAGACTCCGTCTCAAAAA"

  let header2 = ">TWO IUB ambiguity codes"
  let seq2 = "cttBtatcatatgctaKggNcataaaSatgtaaaDcDRtBggDtctttataattcBgtcg"

  let header3 = ">THREE Homo sapiens frequency"
  let seq3 = "aatattatgataatttcccaacttatttatagNgaagcactaaNcattcaBtgNttatcg"

  print(header1)
  let rc1 = reverse_complement(seq1)
  print_seq(rc1)

  print(header2)
  let rc2 = reverse_complement(seq2)
  print_seq(rc2)

  print(header3)
  let rc3 = reverse_complement(seq3)
  print_seq(rc3)
}
