//! 穷尽性检查模块
//!
//! 检查 match 表达式是否覆盖所有可能的模式

use x_lexer::span::Span;
use x_parser::ast::{Literal, Pattern, Type};

use crate::TypeError;

/// 穷尽性检查错误
#[derive(Debug, Clone)]
pub struct NonExhaustiveError {
    /// 未覆盖的模式描述
    pub uncovered_patterns: Vec<String>,
}

/// 检查模式列表是否穷尽
pub fn check_exhaustive(patterns: &[Pattern], scrutinee_type: &Type) -> Result<(), NonExhaustiveError> {
    // 构建模式矩阵
    let matrix: PatternMatrix = patterns.iter().map(normalize_pattern).collect();

    // 根据类型检查穷尽性
    let uncovered = compute_uncovered(&matrix, scrutinee_type)?;

    if uncovered.is_empty() {
        Ok(())
    } else {
        Err(NonExhaustiveError {
            uncovered_patterns: uncovered,
        })
    }
}

/// 模式矩阵：每行是一个模式的分量列表
type PatternMatrix = Vec<Vec<NormalizedPattern>>;

/// 规范化的模式（便于分析）
#[derive(Debug, Clone, PartialEq)]
enum NormalizedPattern {
    /// 通配符 _
    Wildcard,
    /// 变量绑定
    Variable(String),
    /// 构造器模式：构造器名 + 参数
    Constructor(String, Vec<NormalizedPattern>),
    /// 字面量
    Literal(LiteralValue),
    /// 或模式
    Or(Box<NormalizedPattern>, Box<NormalizedPattern>),
}

/// 字面量值（用于穷尽性检查）
#[derive(Debug, Clone, PartialEq)]
enum LiteralValue {
    Integer(i64),
    Float(f64),
    Boolean(bool),
    String(String),
    Char(char),
    Unit,
    Null,
    None,
}

/// 将 AST Pattern 规范化
fn normalize_pattern(p: &Pattern) -> Vec<NormalizedPattern> {
    match p {
        Pattern::Wildcard => vec![NormalizedPattern::Wildcard],
        Pattern::Variable(name) => vec![NormalizedPattern::Variable(name.clone())],
        Pattern::Literal(lit) => vec![NormalizedPattern::Literal(literal_to_value(lit))],
        Pattern::Array(elements) => {
            // 数组模式：转换为构造器形式
            let elements: Vec<NormalizedPattern> = elements
                .iter()
                .flat_map(normalize_pattern)
                .collect();
            vec![NormalizedPattern::Constructor("Array".to_string(), elements)]
        }
        Pattern::Dictionary(entries) => {
            // 字典模式：转换为构造器形式
            let entries: Vec<NormalizedPattern> = entries
                .iter()
                .flat_map(|(k, v)| {
                    let mut result = normalize_pattern(k);
                    result.extend(normalize_pattern(v));
                    result
                })
                .collect();
            vec![NormalizedPattern::Constructor("Dictionary".to_string(), entries)]
        }
        Pattern::Record(name, fields) => {
            // 记录模式
            let fields: Vec<NormalizedPattern> = fields
                .iter()
                .flat_map(|(_, p)| normalize_pattern(p))
                .collect();
            vec![NormalizedPattern::Constructor(name.clone(), fields)]
        }
        Pattern::Tuple(elements) => {
            // 元组模式
            let elements: Vec<NormalizedPattern> = elements
                .iter()
                .flat_map(normalize_pattern)
                .collect();
            vec![NormalizedPattern::Constructor("Tuple".to_string(), elements)]
        }
        Pattern::Or(left, right) => {
            // 或模式：展开
            let mut result = normalize_pattern(left);
            result.extend(normalize_pattern(right));
            result
        }
        Pattern::Guard(inner, _) => {
            // 带守卫的模式：保守地视为通配符（守卫可能失败）
            normalize_pattern(inner)
        }
        Pattern::EnumConstructor(type_name, variant_name, args) => {
            // 枚举构造器模式
            let args: Vec<NormalizedPattern> = args
                .iter()
                .flat_map(normalize_pattern)
                .collect();
            // 组合类型名和变体名作为构造器名
            let constructor_name = format!("{}.{}", type_name, variant_name);
            vec![NormalizedPattern::Constructor(constructor_name, args)]
        }
    }
}

fn literal_to_value(lit: &Literal) -> LiteralValue {
    match lit {
        Literal::Integer(n) => LiteralValue::Integer(*n),
        Literal::Float(f) => LiteralValue::Float(*f),
        Literal::Boolean(b) => LiteralValue::Boolean(*b),
        Literal::String(s) => LiteralValue::String(s.clone()),
        Literal::Char(c) => LiteralValue::Char(*c),
        Literal::Null => LiteralValue::Null,
        Literal::None => LiteralValue::None,
        Literal::Unit => LiteralValue::Unit,
    }
}

/// 计算未覆盖的模式
fn compute_uncovered(matrix: &PatternMatrix, ty: &Type) -> Result<Vec<String>, NonExhaustiveError> {
    // 如果矩阵为空，则需要匹配整个类型
    if matrix.is_empty() {
        return Ok(vec![type_to_pattern_string(ty)]);
    }

    // 检查是否有通配符模式
    for row in matrix {
        if row.iter().all(|p| matches!(p, NormalizedPattern::Wildcard | NormalizedPattern::Variable(_))) {
            return Ok(vec![]);
        }
    }

    // 根据类型进行特定的穷尽性检查
    match ty {
        Type::Bool => check_bool_exhaustive(matrix),
        Type::Option(inner) => check_option_exhaustive(matrix, inner),
        Type::Result(ok, err) => check_result_exhaustive(matrix, ok, err),
        Type::Union(name, _) => check_union_exhaustive(matrix, name),
        _ => {
            // 对于无限类型（Int, String 等），需要通配符才能穷尽
            for row in matrix {
                for p in row {
                    if matches!(p, NormalizedPattern::Wildcard | NormalizedPattern::Variable(_)) {
                        return Ok(vec![]);
                    }
                }
            }
            Ok(vec![format!("通配符 _ 来匹配 {}", ty)])
        }
    }
}

/// 检查 Bool 类型的穷尽性
fn check_bool_exhaustive(matrix: &PatternMatrix) -> Result<Vec<String>, NonExhaustiveError> {
    let mut has_true = false;
    let mut has_false = false;
    let mut has_wildcard = false;

    for row in matrix {
        for p in row {
            match p {
                NormalizedPattern::Literal(LiteralValue::Boolean(true)) => has_true = true,
                NormalizedPattern::Literal(LiteralValue::Boolean(false)) => has_false = true,
                NormalizedPattern::Wildcard | NormalizedPattern::Variable(_) => has_wildcard = true,
                _ => {}
            }
        }
    }

    if has_wildcard || (has_true && has_false) {
        Ok(vec![])
    } else {
        let mut missing = vec![];
        if !has_true {
            missing.push("true".to_string());
        }
        if !has_false {
            missing.push("false".to_string());
        }
        Ok(missing)
    }
}

/// 检查 Option<T> 类型的穷尽性
fn check_option_exhaustive(matrix: &PatternMatrix, _inner: &Type) -> Result<Vec<String>, NonExhaustiveError> {
    let mut has_some = false;
    let mut has_none = false;
    let mut has_wildcard = false;

    for row in matrix {
        for p in row {
            match p {
                NormalizedPattern::Constructor(name, _) if name == "Some" => has_some = true,
                NormalizedPattern::Literal(LiteralValue::None) => has_none = true,
                NormalizedPattern::Constructor(name, _) if name == "None" => has_none = true,
                NormalizedPattern::Wildcard | NormalizedPattern::Variable(_) => has_wildcard = true,
                _ => {}
            }
        }
    }

    if has_wildcard || (has_some && has_none) {
        Ok(vec![])
    } else {
        let mut missing = vec![];
        if !has_some {
            missing.push("Some(_)".to_string());
        }
        if !has_none {
            missing.push("None".to_string());
        }
        Ok(missing)
    }
}

/// 检查 Result<T, E> 类型的穷尽性
fn check_result_exhaustive(
    matrix: &PatternMatrix,
    _ok: &Type,
    _err: &Type,
) -> Result<Vec<String>, NonExhaustiveError> {
    let mut has_ok = false;
    let mut has_err = false;
    let mut has_wildcard = false;

    for row in matrix {
        for p in row {
            match p {
                NormalizedPattern::Constructor(name, _) if name == "Ok" => has_ok = true,
                NormalizedPattern::Constructor(name, _) if name == "Err" => has_err = true,
                NormalizedPattern::Wildcard | NormalizedPattern::Variable(_) => has_wildcard = true,
                _ => {}
            }
        }
    }

    if has_wildcard || (has_ok && has_err) {
        Ok(vec![])
    } else {
        let mut missing = vec![];
        if !has_ok {
            missing.push("Ok(_)".to_string());
        }
        if !has_err {
            missing.push("Err(_)".to_string());
        }
        Ok(missing)
    }
}

/// 检查联合类型的穷尽性
fn check_union_exhaustive(matrix: &PatternMatrix, _name: &str) -> Result<Vec<String>, NonExhaustiveError> {
    // 对于联合类型，需要匹配所有变体或使用通配符
    // 这里简化实现，检查是否有通配符
    for row in matrix {
        for p in row {
            if matches!(p, NormalizedPattern::Wildcard | NormalizedPattern::Variable(_)) {
                return Ok(vec![]);
            }
        }
    }
    // 无法确定所有变体，返回需要通配符
    Ok(vec!["通配符 _".to_string()])
}

/// 将类型转换为模式字符串
fn type_to_pattern_string(ty: &Type) -> String {
    match ty {
        Type::Bool => "true 或 false".to_string(),
        Type::Option(inner) => format!("Some({}) 或 None", type_to_pattern_string(inner)),
        Type::Result(ok, err) => format!("Ok({}) 或 Err({})", type_to_pattern_string(ok), type_to_pattern_string(err)),
        Type::Int => "整数字面量或 _".to_string(),
        Type::Float => "浮点字面量或 _".to_string(),
        Type::String => "字符串字面量或 _".to_string(),
        Type::Char => "字符字面量或 _".to_string(),
        _ => format!("_ 来匹配 {}", ty),
    }
}

/// 将 NonExhaustiveError 转换为 TypeError
impl From<NonExhaustiveError> for TypeError {
    fn from(err: NonExhaustiveError) -> Self {
        TypeError::NotImplemented {
            feature: format!("非穷尽模式匹配：缺少 {}", err.uncovered_patterns.join(", ")),
            span: Span::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bool_exhaustive() {
        // true 和 false 都存在
        let patterns = vec![
            Pattern::Literal(Literal::Boolean(true)),
            Pattern::Literal(Literal::Boolean(false)),
        ];
        assert!(check_exhaustive(&patterns, &Type::Bool).is_ok());

        // 只有 true
        let patterns = vec![Pattern::Literal(Literal::Boolean(true))];
        let result = check_exhaustive(&patterns, &Type::Bool);
        assert!(result.is_err());
    }

    #[test]
    fn test_option_exhaustive() {
        // Some 和 None 都存在
        let patterns = vec![
            Pattern::Record("Some".to_string(), vec![("_".to_string(), Pattern::Wildcard)]),
            Pattern::Literal(Literal::None),
        ];
        assert!(check_exhaustive(&patterns, &Type::Option(Box::new(Type::Int))).is_ok());

        // 只有 Some
        let patterns = vec![
            Pattern::Record("Some".to_string(), vec![("_".to_string(), Pattern::Wildcard)]),
        ];
        let result = check_exhaustive(&patterns, &Type::Option(Box::new(Type::Int)));
        assert!(result.is_err());
    }

    #[test]
    fn test_wildcard_is_exhaustive() {
        let patterns = vec![Pattern::Wildcard];
        assert!(check_exhaustive(&patterns, &Type::Int).is_ok());
        assert!(check_exhaustive(&patterns, &Type::Bool).is_ok());
        assert!(check_exhaustive(&patterns, &Type::Option(Box::new(Type::Int))).is_ok());
    }
}
