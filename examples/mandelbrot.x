// mandelbrot (Benchmarks Game)
// Generate Mandelbrot set as PBM image.
// Reference: https://benchmarksgame-team.pages.debian.net/benchmarksgame/description/mandelbrot.html

fun main() {
  let n = 200
  let max_iter = 50
  let limit2 = 4.0

  print(concat("P4\n", concat(to_string(n), concat(" ", to_string(n)))))

  var y = 0
  while y < n {
    var bits = 0
    var bit_count = 0
    var line = ""

    var x = 0
    while x < n {
      let cr = 2.0 * to_float(x) / to_float(n) - 1.5
      let ci = 2.0 * to_float(y) / to_float(n) - 1.0

      var zr = 0.0
      var zi = 0.0
      var escaped = false
      var iter = 0

      while iter < max_iter && !escaped {
        let tr = zr * zr - zi * zi + cr
        let ti = 2.0 * zr * zi + ci
        zr = tr
        zi = ti
        if zr * zr + zi * zi > limit2 {
          escaped = true
        }
        iter = iter + 1
      }

      bits = bits * 2
      if !escaped {
        bits = bits + 1
      }
      bit_count = bit_count + 1

      if bit_count == 8 {
        line = concat(line, to_string(bits))
        line = concat(line, " ")
        bits = 0
        bit_count = 0
      }

      x = x + 1
    }

    if bit_count > 0 {
      var shift = 8 - bit_count
      while shift > 0 {
        bits = bits * 2
        shift = shift - 1
      }
      line = concat(line, to_string(bits))
    }

    print(str_trim(line))
    y = y + 1
  }
}
