// n-body (Benchmarks Game)
// Symplectic integrator for Jovian planet orbits.
// Reference: https://benchmarksgame-team.pages.debian.net/benchmarksgame/description/nbody.html

fun main() {
  let pi = 3.141592653589793
  let solar_mass = 4.0 * pi * pi
  let days_per_year = 365.24

  // Body data: [x, y, z, vx, vy, vz, mass] for Sun + 4 Jovian planets
  var sun = [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, solar_mass]

  var jupiter = [
    4.84143144246472090,
    -1.16032004402742839,
    -0.103622044471123109,
    0.00166007664274403694 * days_per_year,
    0.00769901118419740425 * days_per_year,
    -0.0000690460016972063023 * days_per_year,
    0.000954791938424326609 * solar_mass
  ]

  var saturn = [
    8.34336671824457987,
    4.12479856412430479,
    -0.403523417114321381,
    -0.00276742510726862411 * days_per_year,
    0.00499852801234917238 * days_per_year,
    0.0000230417297573763929 * days_per_year,
    0.000285885980666130812 * solar_mass
  ]

  var uranus = [
    12.8943695621391310,
    -15.1111514016986312,
    -0.223307578892655734,
    0.00296460137564761618 * days_per_year,
    0.00237847173959480950 * days_per_year,
    -0.0000296589568540237556 * days_per_year,
    0.0000436624404335156298 * solar_mass
  ]

  var neptune = [
    15.3796971148509165,
    -25.9193146099879641,
    0.179258772950371181,
    0.00268067772490389322 * days_per_year,
    0.00162824170038242295 * days_per_year,
    -0.0000951592254519715870 * days_per_year,
    0.0000515138902046611451 * solar_mass
  ]

  // Offset momentum
  var px = 0.0
  var py = 0.0
  var pz = 0.0
  px = px + jupiter[3] * jupiter[6]
  py = py + jupiter[4] * jupiter[6]
  pz = pz + jupiter[5] * jupiter[6]
  px = px + saturn[3] * saturn[6]
  py = py + saturn[4] * saturn[6]
  pz = pz + saturn[5] * saturn[6]
  px = px + uranus[3] * uranus[6]
  py = py + uranus[4] * uranus[6]
  pz = pz + uranus[5] * uranus[6]
  px = px + neptune[3] * neptune[6]
  py = py + neptune[4] * neptune[6]
  pz = pz + neptune[5] * neptune[6]
  sun[3] = -1.0 * px / solar_mass
  sun[4] = -1.0 * py / solar_mass
  sun[5] = -1.0 * pz / solar_mass

  // Energy
  print(format_float(energy(sun, jupiter, saturn, uranus, neptune), 9))

  // Advance N=1000 steps with dt=0.01
  var n = 0
  while n < 1000 {
    advance(sun, jupiter, saturn, uranus, neptune, 0.01)
    n = n + 1
  }

  print(format_float(energy(sun, jupiter, saturn, uranus, neptune), 9))
}

fun energy(s, j, sa, u, ne) {
  var e = 0.0
  // Kinetic
  e = e + 0.5 * s[6] * (s[3]*s[3] + s[4]*s[4] + s[5]*s[5])
  e = e + 0.5 * j[6] * (j[3]*j[3] + j[4]*j[4] + j[5]*j[5])
  e = e + 0.5 * sa[6] * (sa[3]*sa[3] + sa[4]*sa[4] + sa[5]*sa[5])
  e = e + 0.5 * u[6] * (u[3]*u[3] + u[4]*u[4] + u[5]*u[5])
  e = e + 0.5 * ne[6] * (ne[3]*ne[3] + ne[4]*ne[4] + ne[5]*ne[5])
  // Potential (all 10 pairs)
  e = e - pair_energy(s, j)
  e = e - pair_energy(s, sa)
  e = e - pair_energy(s, u)
  e = e - pair_energy(s, ne)
  e = e - pair_energy(j, sa)
  e = e - pair_energy(j, u)
  e = e - pair_energy(j, ne)
  e = e - pair_energy(sa, u)
  e = e - pair_energy(sa, ne)
  e = e - pair_energy(u, ne)
  return e
}

fun pair_energy(a, b) {
  let dx = a[0] - b[0]
  let dy = a[1] - b[1]
  let dz = a[2] - b[2]
  let dist = sqrt(dx*dx + dy*dy + dz*dz)
  return a[6] * b[6] / dist
}

fun advance(s, j, sa, u, ne, dt) {
  // All 10 pairs
  advance_pair(s, j, dt)
  advance_pair(s, sa, dt)
  advance_pair(s, u, dt)
  advance_pair(s, ne, dt)
  advance_pair(j, sa, dt)
  advance_pair(j, u, dt)
  advance_pair(j, ne, dt)
  advance_pair(sa, u, dt)
  advance_pair(sa, ne, dt)
  advance_pair(u, ne, dt)
  // Update positions
  move_body(s, dt)
  move_body(j, dt)
  move_body(sa, dt)
  move_body(u, dt)
  move_body(ne, dt)
}

fun advance_pair(a, b, dt) {
  let dx = a[0] - b[0]
  let dy = a[1] - b[1]
  let dz = a[2] - b[2]
  let d2 = dx*dx + dy*dy + dz*dz
  let dist = sqrt(d2)
  let mag = dt / (d2 * dist)
  a[3] = a[3] - dx * b[6] * mag
  a[4] = a[4] - dy * b[6] * mag
  a[5] = a[5] - dz * b[6] * mag
  b[3] = b[3] + dx * a[6] * mag
  b[4] = b[4] + dy * a[6] * mag
  b[5] = b[5] + dz * a[6] * mag
}

fun move_body(b, dt) {
  b[0] = b[0] + dt * b[3]
  b[1] = b[1] + dt * b[4]
  b[2] = b[2] + dt * b[5]
}
