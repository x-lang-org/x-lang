// fannkuch-redux (Benchmarks Game)
// Pancake flipping on all permutations of [1..n].
// Reference: https://benchmarksgame-team.pages.debian.net/benchmarksgame/description/fannkuchredux.html

fun main() {
  let n = 7
  var perm = new_array(n, 0)
  var count = new_array(n, 0)
  var perm1 = new_array(n, 0)

  var i = 0
  while i < n {
    perm1[i] = i
    i = i + 1
  }

  var checksum = 0
  var max_flips = 0
  var perm_count = 0

  var r = n
  while true {
    while r > 1 {
      count[r - 1] = r
      r = r - 1
    }

    // Copy perm1 to perm
    var k = 0
    while k < n {
      perm[k] = perm1[k]
      k = k + 1
    }

    // Count flips
    var flips = 0
    var first = perm[0]
    while first != 0 {
      // Reverse perm[0..first]
      var lo = 0
      var hi = first
      while lo < hi {
        let tmp = perm[lo]
        perm[lo] = perm[hi]
        perm[hi] = tmp
        lo = lo + 1
        hi = hi - 1
      }
      flips = flips + 1
      first = perm[0]
    }

    if flips > max_flips {
      max_flips = flips
    }

    if perm_count % 2 == 0 {
      checksum = checksum + flips
    } else {
      checksum = checksum - flips
    }

    perm_count = perm_count + 1

    // Next permutation (rotate perm1)
    var done = false
    r = 1
    while r < n && !done {
      let perm0 = perm1[0]
      var ii = 0
      while ii < r {
        perm1[ii] = perm1[ii + 1]
        ii = ii + 1
      }
      perm1[r] = perm0

      count[r] = count[r] - 1
      if count[r] > 0 {
        done = true
      } else {
        r = r + 1
      }
    }

    if r >= n {
      // All permutations done
      print(checksum)
      print(concat("Pfannkuchen(", concat(to_string(n), concat(") = ", to_string(max_flips)))))
      return 0
    }
  }
}
