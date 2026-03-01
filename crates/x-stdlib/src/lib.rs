// 标准库

use std::io::Write;

pub mod io;
pub mod collections;

pub use io::*;
pub use collections::*;

/// 打印函数
pub fn print<T: ToString>(value: T) {
    println!("{}", value.to_string());
}

/// 输入函数
pub fn input(prompt: &str) -> String {
    print!("{}", prompt);
    std::io::stdout().flush().unwrap();
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

/// 类型转换
pub trait ToString {
    fn to_string(&self) -> String;
}

impl ToString for i64 {
    fn to_string(&self) -> String {
        std::string::ToString::to_string(self)
    }
}

impl ToString for f64 {
    fn to_string(&self) -> String {
        std::string::ToString::to_string(self)
    }
}

impl ToString for bool {
    fn to_string(&self) -> String {
        std::string::ToString::to_string(self)
    }
}

impl ToString for &str {
    fn to_string(&self) -> String {
        std::string::ToString::to_string(self)
    }
}

impl ToString for String {
    fn to_string(&self) -> String {
        self.clone()
    }
}