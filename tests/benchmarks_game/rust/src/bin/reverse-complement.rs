use std::io::{self, BufRead};

fn complement(c: char) -> char {
    match c {
        'A' | 'a' => 'T',
        'T' | 't' | 'U' | 'u' => 'A',
        'C' | 'c' => 'G',
        'G' | 'g' => 'C',
        'R' | 'r' => 'Y',
        'Y' | 'y' => 'R',
        'K' | 'k' => 'M',
        'M' | 'm' => 'K',
        'B' | 'b' => 'V',
        'D' | 'd' => 'H',
        'H' | 'h' => 'D',
        'V' | 'v' => 'B',
        _ => c,
    }
}

fn print_rc(seq: &[char]) {
    let mut line = String::new();
    for &c in seq.iter().rev() {
        line.push(complement(c));
        if line.len() == 60 {
            println!("{}", line);
            line.clear();
        }
    }
    if !line.is_empty() {
        println!("{}", line);
    }
}

fn main() {
    let stdin = io::stdin();
    let mut seq = Vec::new();
    for line in stdin.lock().lines() {
        let line = line.unwrap();
        if line.starts_with('>') {
            print_rc(&seq);
            seq.clear();
            println!("{}", line);
        } else {
            for c in line.chars() {
                seq.push(c);
            }
        }
    }
    print_rc(&seq);
}
