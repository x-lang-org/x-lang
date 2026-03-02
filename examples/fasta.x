// fasta (Benchmarks Game)
// Generate FASTA format sequences.
// Reference: https://benchmarksgame-team.pages.debian.net/benchmarksgame/description/fasta.html

fun main() {
  let n = 1000
  let alu = "GGCCGGGCGCGGTGGCTCACGCCTGTAATCCCAGCACTTTGGGAGGCCGAGGCGGGCGGATCACCTGAGGTCAGGAGTTCGAGACCAGCCTGGCCAACATGGTGAAACCCCGTCTCTACTAAAAATACAAAAATTAGCCGGGCGTGGTGGCGCGCGCCTGTAATCCCAGCTACTCGGGAGGCTGAGGCAGGAGAATCGCTTGAACCCGGGAGGCGGAGGTTGCAGTGAGCCGAGATCGCGCCACTGCACTCCAGCCTGGGCGACAGAGCGAGACTCCGTCTCAAAAA"

  let iub_chars = "agtc" // Simplified alphabet
  let iub_probs = [0.27, 0.12, 0.12, 0.27]
  let homo_chars = "acgt"
  let homo_probs = [0.3029549426680, 0.1979883004921, 0.1975473066391, 0.3015094502008]

  print(">ONE Homo sapiens alu")
  repeat_fasta(alu, n * 2)

  print(">TWO IUB ambiguity codes")
  random_fasta(iub_chars, iub_probs, n * 3)

  print(">THREE Homo sapiens frequency")
  random_fasta(homo_chars, homo_probs, n * 5)
}

fun repeat_fasta(seq, count) {
  let seq_len = len(seq)
  var pos = 0
  var remaining = count
  while remaining > 0 {
    let line_len = 60
    var actual = line_len
    if actual > remaining {
      actual = remaining
    }
    var line = ""
    var i = 0
    while i < actual {
      line = concat(line, char_at(seq, pos % seq_len))
      pos = pos + 1
      i = i + 1
    }
    print(line)
    remaining = remaining - actual
  }
}

var last_random = 42

fun gen_random(max_val) {
  let ia = 3877
  let ic = 29573
  let im = 139968
  last_random = (last_random * ia + ic) % im
  return max_val * to_float(last_random) / to_float(im)
}

fun random_fasta(chars, probs, count) {
  let nchars = len(chars)
  var remaining = count
  while remaining > 0 {
    let line_len = 60
    var actual = line_len
    if actual > remaining {
      actual = remaining
    }
    var line = ""
    var i = 0
    while i < actual {
      let r = gen_random(1.0)
      var cum = 0.0
      var j = 0
      var ch = char_at(chars, nchars - 1)
      while j < nchars {
        cum = cum + probs[j]
        if r < cum {
          ch = char_at(chars, j)
          j = nchars
        }
        j = j + 1
      }
      line = concat(line, ch)
      i = i + 1
    }
    print(line)
    remaining = remaining - actual
  }
}
