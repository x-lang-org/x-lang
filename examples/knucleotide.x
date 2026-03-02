// k-nucleotide (Benchmarks Game)
// Count frequencies of k-mers in a DNA sequence.
// Reference: https://benchmarksgame-team.pages.debian.net/benchmarksgame/description/knucleotide.html

fun count_kmers(seq, k) {
  var counts = new_map()
  let n = len(seq)
  var i = 0
  while i + k <= n {
    let kmer = str_upper(substring(seq, i, i + k))
    let cur = map_get(counts, kmer)
    map_set(counts, kmer, cur + 1)
    i = i + 1
  }
  return counts
}

fun print_frequencies(seq, k) {
  let counts = count_kmers(seq, k)
  let keys = map_keys(counts)
  let total = len(seq) - k + 1

  // Build array of [key, count] pairs for sorting
  var pairs = []
  var i = 0
  while i < len(keys) {
    let key = keys[i]
    let cnt = map_get(counts, key)
    push(pairs, [key, cnt])
    i = i + 1
  }

  sort_by_value_desc(pairs)

  var j = 0
  while j < len(pairs) {
    let key = pairs[j][0]
    let cnt = pairs[j][1]
    let freq = 100.0 * to_float(cnt) / to_float(total)
    print(concat(key, concat(" ", format_float(freq, 3))))
    j = j + 1
  }
  print("")
}

fun print_count(seq, fragment) {
  let k = len(fragment)
  let counts = count_kmers(seq, k)
  let frag_upper = str_upper(fragment)
  let cnt = map_get(counts, frag_upper)
  print(concat(to_string(cnt), concat("\t", frag_upper)))
}

fun main() {
  // Embedded test sequence (simplified version of >THREE from standard input)
  let seq = "aatattatgataatttcccaacttatttatagNgaagcactaaNcattcaBtgNttatcgaatNactaaNctttttcaaactgNcaatatatgtgaaactttNgccatgattaaagaatNcattNcctacatatgatNBcNBNgaacatattatatBctataagtacBcatagatgatgtatagttBtaHBagttacacgatctcNatgaatatBtcNagagatWtNttHaagYtHcHcaaRtaaaaDcagatgaagcNtttcHcgatMcWaaHatttgattaaa"

  // Print 1-mer and 2-mer frequency tables
  print_frequencies(seq, 1)
  print_frequencies(seq, 2)

  // Print specific k-mer counts
  print_count(seq, "GGT")
  print_count(seq, "GGTA")
  print_count(seq, "GGTATT")
  print_count(seq, "GGTATTTTAATT")
  print_count(seq, "GGTATTTTAATTTATAGT")
}
