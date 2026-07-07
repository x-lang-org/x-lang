fn random(max: f64, last: &mut u32) -> f64 {
    *last = (*last * 3877 + 29573) % 139968;
    max * (*last as f64) / 139968.0
}

fn select_random(chars: &[&str], probs: &[f64], last: &mut u32) -> String {
    let r = random(1.0, last);
    for (i, &p) in probs.iter().enumerate() {
        if r < p {
            return chars[i].to_string();
        }
    }
    chars[chars.len() - 1].to_string()
}

fn make_repeated_fasta(desc: &str, alu: &str, n: usize) {
    println!("{}", desc);
    let bytes = alu.as_bytes();
    let alu_len = bytes.len();
    let mut line = String::new();
    for i in 0..n {
        line.push(bytes[i % alu_len] as char);
        if line.len() == 60 {
            println!("{}", line);
            line.clear();
        }
    }
    if !line.is_empty() {
        println!("{}", line);
    }
}

fn make_random_fasta(desc: &str, chars: &[&str], probs: &[f64], n: usize, last: &mut u32) {
    println!("{}", desc);
    let mut line = String::new();
    for _ in 0..n {
        line.push_str(&select_random(chars, probs, last));
        if line.len() >= 60 {
            println!("{}", line);
            line.clear();
        }
    }
    if !line.is_empty() {
        println!("{}", line);
    }
}

fn main() {
    let alu = "GGCCGGGCGCGGTGGCTCACGCCTGTAATCCCAGCACTTTGGGAGGCCGAGGCGGGCGGATCACCTGAGGTCAGGAGTTCGAGACCAGCCTGGCCAACATGGTGAAACCCCGTCTCTACTAAAAATACAAAAATTAGCCGGGCGTGGTGGCGCGCGCCTGTAATCCCAGCTACTCGGGAGGCTGAGGCAGGAGAATCGCTTGAACCCGGGAGGCGGAGGTTGCAGTGAGCCGAGATCGCGCCACTGCACTCCAGCCTGGGCGACAGAGCGAGACTCCGTCTCAAAAA";
    
    let iub_chars = ["a", "c", "g", "t", "B", "D", "H", "K", "M", "N", "R", "S", "V", "W", "Y"];
    let iub_probs = [0.27, 0.39, 0.51, 0.78, 0.80, 0.82, 0.84, 0.86, 0.88, 0.90, 0.92, 0.94, 0.96, 0.98, 1.00];
    
    let homo_chars = ["a", "c", "g", "t"];
    let homo_probs = [0.3029549426680, 0.5009432431601, 0.6984905497992, 1.0000000000000];
    
    let mut last = 42;
    make_repeated_fasta(">ONE Homo sapiens alu", alu, 1000);
    make_random_fasta(">TWO IUB ambiguity codes", &iub_chars, &iub_probs, 1000, &mut last);
    make_random_fasta(">THREE Homo sapiens frequency", &homo_chars, &homo_probs, 1000, &mut last);
}
