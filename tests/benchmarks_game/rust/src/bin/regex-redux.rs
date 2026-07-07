use regex::Regex;
use std::io::{self, BufRead};

fn main() {
    let stdin = io::stdin();
    let mut initial_len = 0;
    let mut seq = String::new();
    for line in stdin.lock().lines() {
        let line = line.unwrap();
        initial_len += line.len() + 1;
        if !line.starts_with('>') {
            seq.push_str(&line);
        }
    }
    let clean_len = seq.len();
    
    let patterns = [
        "agggtaaa|tttaccct",
        "[cgt]gggtaaa|tttaccc[acg]",
        "a[act]ggtaaa|tttacc[agt]t",
        "ag[act]gtaaa|tttac[agt]ct",
        "agg[act]taaa|ttta[agt]cct",
        "aggg[acg]aaa|ttt[cgt]ccct",
        "agggt[cgt]aa|tt[acg]accct",
        "agggta[cgt]a|t[acg]taccct",
        "agggtaa[cgt]|[acg]ttaccct",
    ];
    
    for &pat in &patterns {
        let re = Regex::new(pat).unwrap();
        let count = re.find_iter(&seq).count();
        println!("{} {}", pat, count);
    }
    
    let replacements = [
        ("B", "(c|g|t)"),
        ("D", "(a|g|t)"),
        ("H", "(a|c|t)"),
        ("K", "(g|t)"),
        ("M", "(a|c)"),
        ("N", "(a|c|g|t)"),
        ("R", "(a|g)"),
        ("S", "(c|g)"),
        ("V", "(a|c|g)"),
        ("W", "(a|t)"),
        ("Y", "(c|t)"),
    ];
    
    let mut replaced = seq;
    for &(pat, rep) in &replacements {
        let re = Regex::new(pat).unwrap();
        replaced = re.replace_all(&replaced, rep).to_string();
    }
    
    println!();
    println!("{}", initial_len);
    println!("{}", clean_len);
    println!("{}", replaced.len());
}
