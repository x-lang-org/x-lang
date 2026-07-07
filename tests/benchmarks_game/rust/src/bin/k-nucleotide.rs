use std::collections::HashMap;
use std::io::{self, BufRead};

fn count_frequencies(seq: &[char], k: usize) -> HashMap<String, usize> {
    let mut map = HashMap::new();
    for i in 0..=seq.len().saturating_sub(k) {
        let sub: String = seq[i..i+k].iter().collect();
        *map.entry(sub).or_insert(0) += 1;
    }
    map
}

fn print_sorted(map: &HashMap<String, usize>, total: f64) {
    let mut vec: Vec<(&String, &usize)> = map.iter().collect();
    vec.sort_by(|a, b| b.1.cmp(a.1).then_with(|| a.0.cmp(b.0)));
    for (k, v) in vec {
        let percent = (*v as f64) * 100.0 / total;
        println!("{} {:.3}", k, percent);
    }
}

fn print_count(map: &HashMap<String, usize>, k: &str) {
    let count = map.get(k).cloned().unwrap_or(0);
    println!("{} {}", count, k);
}

fn main() {
    let stdin = io::stdin();
    let mut reading = false;
    let mut seq = Vec::new();
    for line in stdin.lock().lines() {
        let line = line.unwrap();
        if line.starts_with('>') {
            if line.starts_with(">THREE") {
                reading = true;
            } else {
                reading = false;
            }
        } else if reading {
            for c in line.chars() {
                seq.push(c.to_ascii_uppercase());
            }
        }
    }
    
    let total = seq.len() as f64;
    if total > 0.0 {
        let dict1 = count_frequencies(&seq, 1);
        print_sorted(&dict1, total);
        println!();
        
        let dict2 = count_frequencies(&seq, 2);
        print_sorted(&dict2, total - 1.0);
        println!();
        
        let dict3 = count_frequencies(&seq, 3);
        print_count(&dict3, "GGT");
        
        let dict4 = count_frequencies(&seq, 4);
        print_count(&dict4, "GGTA");
        
        let dict6 = count_frequencies(&seq, 6);
        print_count(&dict6, "GGTATT");
        
        let dict12 = count_frequencies(&seq, 12);
        print_count(&dict12, "GGTATTTTAATT");
        
        let dict18 = count_frequencies(&seq, 18);
        print_count(&dict18, "GGTATTTTAATTTATAGT");
    }
}
