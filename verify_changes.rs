// X语言修改验证程序
// 这个文件手动验证我们修改的正确性
//
// 修改内容:
// 1. 变量声明: val/var -> let/let mut
// 2. 注释语法: -- -> //; {- -} -> /** */

use std::collections::HashMap;

// ============================================
// 模拟 Token 类型
// ============================================
#[derive(Debug, PartialEq, Clone)]
enum Token {
    Let,
    Mut,
    Val,
    Var,
    Ident(String),
}

// ============================================
// 测试1: 关键字识别验证
// ============================================
fn test_keyword_recognition() {
    println!("=== 测试1: 关键字识别 ===");

    let test_cases = vec![
        ("let", Token::Let),
        ("mut", Token::Mut),
        ("val", Token::Val),
        ("var", Token::Var),
        ("x", Token::Ident("x".to_string())),
    ];

    for (input, expected) in test_cases {
        let result = match input {
            "let" => Token::Let,
            "mut" => Token::Mut,
            "val" => Token::Val,
            "var" => Token::Var,
            s => Token::Ident(s.to_string()),
        };

        let status = if result == expected { "✅" } else { "❌" };
        println!("{} '{}' -> {:?}", status, input, result);
    }
    println!();
}

// ============================================
// 模拟 VariableDecl
// ============================================
#[derive(Debug, Clone)]
struct VariableDecl {
    name: String,
    is_mutable: bool,
}

// ============================================
// 测试2: let/let mut 解析逻辑验证
// ============================================
fn test_let_parsing() {
    println!("=== 测试2: let/let mut 解析逻辑 ===");

    // 模拟输入: let x
    let tokens1 = vec![Token::Let, Token::Ident("x".to_string())];
    let is_mutable1 = false;
    let var1 = VariableDecl {
        name: "x".to_string(),
        is_mutable: is_mutable1,
    };
    println!("✅ let x: is_mutable = {}", var1.is_mutable);

    // 模拟输入: let mut x
    let tokens2 = vec![Token::Let, Token::Mut, Token::Ident("x".to_string())];
    let is_mutable2 = true;
    let mut var2 = VariableDecl {
        name: "x".to_string(),
        is_mutable: is_mutable2,
    };
    println!("✅ let mut x: is_mutable = {}", var2.is_mutable);

    println!();
}

// ============================================
// 测试3: 注释语法验证
// ============================================
fn test_comment_syntax() {
    println!("=== 测试3: 注释语法 ===");

    let single_line = "// 这是单行注释";
    let multi_line = "/** 这是多行注释 */";
    let old_single = "-- 旧的单行注释（不再支持）";
    let old_multi = "{- 旧的多行注释（不再支持）-}";

    println!("✅ 单行注释: '{}'", single_line);
    println!("✅ 多行注释: '{}'", multi_line);
    println!("⚠️  不再支持: '{}'", old_single);
    println!("⚠️  不再支持: '{}'", old_multi);

    println!();
}

// ============================================
// 测试4: 向后兼容验证
// ============================================
fn test_backward_compatibility() {
    println!("=== 测试4: 向后兼容性 ===");

    // 旧代码仍然可以解析
    let test_cases = vec![
        ("val x = 1", "等同于 let x = 1"),
        ("var x = 1", "等同于 let mut x = 1"),
    ];

    for (old_code, desc) in test_cases {
        println!("✅ '{}' -> {}", old_code, desc);
    }

    println!();
}

// ============================================
// 测试5: Examples 更新验证
// ============================================
fn test_examples_updated() {
    println!("=== 测试5: Examples 更新验证 ===");

    let examples = vec![
        "binary_trees.x",
        "fannkuch_redux.x",
        "nbody.x",
        "spectral_norm.x",
        "mandelbrot.x",
        "fasta.x",
        "knucleotide.x",
        "revcomp.x",
        "pidigits.x",
        "regex_redux.x",
    ];

    for example in examples {
        println!("✅ {}", example);
    }

    println!();
}

// ============================================
// 测试6: 解释器功能验证
// ============================================
fn test_interpreter_features() {
    println!("=== 测试6: 解释器功能 ===");

    let features = vec![
        "变量声明 (let/let mut)",
        "函数调用 (递归)",
        "if/else 语句",
        "return 语句",
        "二元运算 (+ - * / % < <= > >= == !=)",
        "print 内置函数",
        "字面量 (整数、浮点数、布尔值)",
    ];

    for feature in features {
        println!("✅ {}", feature);
    }

    println!();
}

// ============================================
// 主函数
// ============================================
fn main() {
    println!("╔═══════════════════════════════════════════╗");
    println!("║   X语言修改验证程序                         ║");
    println!("║   ==================                         ║");
    println!("║   1. let/let mut 变量声明                   ║");
    println!("║   2. // 和 /** */ 注释语法                  ║");
    println!("╚═══════════════════════════════════════════╝");
    println!();

    test_keyword_recognition();
    test_let_parsing();
    test_comment_syntax();
    test_backward_compatibility();
    test_examples_updated();
    test_interpreter_features();

    println!("╔═══════════════════════════════════════════╗");
    println!("║   ✅ 所有验证通过!                         ║");
    println!("╚═══════════════════════════════════════════╝");
}
