use num_bigint::BigInt;
use num_traits::{One, Zero};

fn compute_pi_digits(n: usize) -> String {
    let mut digits = String::new();
    let mut q = BigInt::one();
    let mut r = BigInt::zero();
    let mut t = BigInt::one();
    let mut k = BigInt::one();
    let mut n1 = BigInt::from(3);
    let mut l = BigInt::from(3);
    
    while digits.len() < n {
        if &q * 4 + &r - &t < &n1 * &t {
            digits.push_str(&n1.to_string());
            let nr = (&r - &n1 * &t) * 10;
            n1 = ((&q * 3 + &r) * 10) / &t - &n1 * 10;
            q = &q * 10;
            r = nr;
        } else {
            let nr = (&q * 2 + &r) * &l;
            let nn = (&q * (&k * 7 + 2) + &r * &l) / (&t * &l);
            q = &q * &k;
            t = &t * &l;
            l += 2;
            k += 1;
            r = nr;
            n1 = nn;
        }
    }
    digits.truncate(n);
    digits
}

fn main() {
    let n = 27;
    let digits = compute_pi_digits(n);
    let len = digits.len();
    let mut i = 0;
    while i < len {
        let chunk_size = std::cmp::min(10, len - i);
        let chunk = &digits[i..i+chunk_size];
        print!("{:<10}\t:{}\n", chunk, i + chunk_size);
        i += 10;
    }
}
