// X语言标准库
//
// 标准库架构：
// - prelude: 自动导入的核心类型和函数
// - option: Option类型（替代null）
// - result: Result类型（替代异常）
// - math: 数学函数和常量
// - string: 字符串操作
// - collections: 集合类型（List, Map, Set）
// - iter: 迭代器
// - io: 输入输出
// - time: 时间处理
// - sys: 系统功能

use std::fmt;

// 模块导出
pub mod prelude;
pub mod option;
pub mod result;
pub mod math;
pub mod string;
pub mod collections;
pub mod iter;
pub mod io;
pub mod time;
pub mod sys;

// 重新导出常用类型
pub use prelude::*;
pub use option::{Option, Option::*};
pub use result::{Result, Result::*};

// 标准库值类型 - 供解释器使用
#[derive(Debug, PartialEq, Clone)]
pub enum StdValue {
    // 基本类型
    Integer(i64),
    Float(f64),
    Boolean(bool),
    String(String),
    Char(char),
    Unit,

    // 标准库类型
    Option(Box<Option<StdValue>>),
    Result(Box<Result<StdValue, StdValue>>),

    // 集合
    List(Vec<StdValue>),
    Map(std::collections::HashMap<String, StdValue>),

    // 函数引用
    Function(String),

    // 其他
    Null,
}

// 标准库函数签名
pub type StdFunction = fn(&[StdValue]) -> Result<StdValue, StdError>;

// 标准库错误
#[derive(Debug, Clone)]
pub enum StdError {
    InvalidArgument(String),
    OutOfBounds(String),
    IoError(String),
    TypeError(String),
    RuntimeError(String),
}

impl fmt::Display for StdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StdError::InvalidArgument(msg) => write!(f, "无效参数: {}", msg),
            StdError::OutOfBounds(msg) => write!(f, "越界: {}", msg),
            StdError::IoError(msg) => write!(f, "IO错误: {}", msg),
            StdError::TypeError(msg) => write!(f, "类型错误: {}", msg),
            StdError::RuntimeError(msg) => write!(f, "运行时错误: {}", msg),
        }
    }
}

impl StdValue {
    pub fn type_name(&self) -> &'static str {
        match self {
            StdValue::Integer(_) => "Int",
            StdValue::Float(_) => "Float",
            StdValue::Boolean(_) => "Bool",
            StdValue::String(_) => "String",
            StdValue::Char(_) => "Char",
            StdValue::Unit => "Unit",
            StdValue::Option(_) => "Option",
            StdValue::Result(_) => "Result",
            StdValue::List(_) => "List",
            StdValue::Map(_) => "Map",
            StdValue::Function(_) => "Function",
            StdValue::Null => "Null",
        }
    }

    pub fn to_string_repr(&self) -> String {
        match self {
            StdValue::Integer(n) => n.to_string(),
            StdValue::Float(n) => n.to_string(),
            StdValue::Boolean(b) => b.to_string(),
            StdValue::String(s) => s.clone(),
            StdValue::Char(c) => c.to_string(),
            StdValue::Unit => "()".to_string(),
            StdValue::Option(opt) => match opt.as_ref() {
                Option::Some(v) => format!("Some({})", v.to_string_repr()),
                Option::None => "None".to_string(),
            },
            StdValue::Result(res) => match res.as_ref() {
                Result::Ok(v) => format!("Ok({})", v.to_string_repr()),
                Result::Err(e) => format!("Err({})", e.to_string_repr()),
            },
            StdValue::List(items) => {
                let items_str: Vec<String> = items.iter().map(|v| v.to_string_repr()).collect();
                format!("[{}]", items_str.join(", "))
            }
            StdValue::Map(map) => {
                let pairs: Vec<String> = map
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, v.to_string_repr()))
                    .collect();
                format!("{{{}}}", pairs.join(", "))
            }
            StdValue::Function(name) => format!("<function {}>", name),
            StdValue::Null => "null".to_string(),
        }
    }

    // 类型转换辅助方法
    pub fn as_integer(&self) -> Result<i64, StdError> {
        match self {
            StdValue::Integer(n) => Ok(*n),
            _ => Err(StdError::TypeError(format!("期望Int，得到{}", self.type_name()))),
        }
    }

    pub fn as_float(&self) -> Result<f64, StdError> {
        match self {
            StdValue::Float(n) => Ok(*n),
            StdValue::Integer(n) => Ok(*n as f64),
            _ => Err(StdError::TypeError(format!("期望Float，得到{}", self.type_name()))),
        }
    }

    pub fn as_boolean(&self) -> Result<bool, StdError> {
        match self {
            StdValue::Boolean(b) => Ok(*b),
            _ => Err(StdError::TypeError(format!("期望Bool，得到{}", self.type_name()))),
        }
    }

    pub fn as_string(&self) -> Result<&str, StdError> {
        match self {
            StdValue::String(s) => Ok(s),
            _ => Err(StdError::TypeError(format!("期望String，得到{}", self.type_name()))),
        }
    }

    pub fn as_list(&self) -> Result<&[StdValue], StdError> {
        match self {
            StdValue::List(l) => Ok(l),
            _ => Err(StdError::TypeError(format!("期望List，得到{}", self.type_name()))),
        }
    }

    pub fn as_list_mut(&mut self) -> Result<&mut Vec<StdValue>, StdError> {
        match self {
            StdValue::List(l) => Ok(l),
            _ => Err(StdError::TypeError(format!("期望List，得到{}", self.type_name()))),
        }
    }
}

// 标准库函数注册表
pub struct StdLib {
    functions: std::collections::HashMap<String, StdFunction>,
}

impl StdLib {
    pub fn new() -> Self {
        let mut functions = std::collections::HashMap::new();

        // Prelude函数
        functions.insert("print".to_string(), prelude::print as StdFunction);
        functions.insert("println".to_string(), prelude::println as StdFunction);
        functions.insert("input".to_string(), prelude::input as StdFunction);
        functions.insert("format".to_string(), prelude::format as StdFunction);

        // Option函数
        functions.insert("Some".to_string(), option::some as StdFunction);
        functions.insert("None".to_string(), option::none as StdFunction);
        functions.insert("is_some".to_string(), option::is_some as StdFunction);
        functions.insert("is_none".to_string(), option::is_none as StdFunction);
        functions.insert("unwrap".to_string(), option::unwrap as StdFunction);
        functions.insert("unwrap_or".to_string(), option::unwrap_or as StdFunction);

        // Result函数
        functions.insert("Ok".to_string(), result::ok as StdFunction);
        functions.insert("Err".to_string(), result::err as StdFunction);
        functions.insert("is_ok".to_string(), result::is_ok as StdFunction);
        functions.insert("is_err".to_string(), result::is_err as StdFunction);

        // 数学函数
        functions.insert("abs".to_string(), math::abs as StdFunction);
        functions.insert("sqrt".to_string(), math::sqrt as StdFunction);
        functions.insert("pow".to_string(), math::pow as StdFunction);
        functions.insert("sin".to_string(), math::sin as StdFunction);
        functions.insert("cos".to_string(), math::cos as StdFunction);
        functions.insert("tan".to_string(), math::tan as StdFunction);
        functions.insert("asin".to_string(), math::asin as StdFunction);
        functions.insert("acos".to_string(), math::acos as StdFunction);
        functions.insert("atan".to_string(), math::atan as StdFunction);
        functions.insert("atan2".to_string(), math::atan2 as StdFunction);
        functions.insert("exp".to_string(), math::exp as StdFunction);
        functions.insert("ln".to_string(), math::ln as StdFunction);
        functions.insert("log2".to_string(), math::log2 as StdFunction);
        functions.insert("log10".to_string(), math::log10 as StdFunction);
        functions.insert("floor".to_string(), math::floor as StdFunction);
        functions.insert("ceil".to_string(), math::ceil as StdFunction);
        functions.insert("round".to_string(), math::round as StdFunction);
        functions.insert("trunc".to_string(), math::trunc as StdFunction);
        functions.insert("min".to_string(), math::min as StdFunction);
        functions.insert("max".to_string(), math::max as StdFunction);
        functions.insert("clamp".to_string(), math::clamp as StdFunction);

        // 字符串函数
        functions.insert("str_len".to_string(), string::str_len as StdFunction);
        functions.insert("str_chars".to_string(), string::str_chars as StdFunction);
        functions.insert("str_concat".to_string(), string::str_concat as StdFunction);
        functions.insert("str_contains".to_string(), string::str_contains as StdFunction);
        functions.insert("str_starts_with".to_string(), string::str_starts_with as StdFunction);
        functions.insert("str_ends_with".to_string(), string::str_ends_with as StdFunction);
        functions.insert("str_substring".to_string(), string::str_substring as StdFunction);
        functions.insert("str_replace".to_string(), string::str_replace as StdFunction);
        functions.insert("str_to_lowercase".to_string(), string::str_to_lowercase as StdFunction);
        functions.insert("str_to_uppercase".to_string(), string::str_to_uppercase as StdFunction);
        functions.insert("str_trim".to_string(), string::str_trim as StdFunction);
        functions.insert("str_split".to_string(), string::str_split as StdFunction);
        functions.insert("str_parse_int".to_string(), string::str_parse_int as StdFunction);
        functions.insert("str_parse_float".to_string(), string::str_parse_float as StdFunction);

        // 集合函数
        functions.insert("list_new".to_string(), collections::list_new as StdFunction);
        functions.insert("list_len".to_string(), collections::list_len as StdFunction);
        functions.insert("list_is_empty".to_string(), collections::list_is_empty as StdFunction);
        functions.insert("list_get".to_string(), collections::list_get as StdFunction);
        functions.insert("list_first".to_string(), collections::list_first as StdFunction);
        functions.insert("list_last".to_string(), collections::list_last as StdFunction);
        functions.insert("list_push".to_string(), collections::list_push as StdFunction);
        functions.insert("list_pop".to_string(), collections::list_pop as StdFunction);
        functions.insert("list_insert".to_string(), collections::list_insert as StdFunction);
        functions.insert("list_remove".to_string(), collections::list_remove as StdFunction);
        functions.insert("list_append".to_string(), collections::list_append as StdFunction);
        functions.insert("list_reverse".to_string(), collections::list_reverse as StdFunction);
        functions.insert("list_contains".to_string(), collections::list_contains as StdFunction);
        functions.insert("list_map".to_string(), collections::list_map as StdFunction);
        functions.insert("list_filter".to_string(), collections::list_filter as StdFunction);
        functions.insert("list_fold".to_string(), collections::list_fold as StdFunction);
        functions.insert("list_sum".to_string(), collections::list_sum as StdFunction);
        functions.insert("list_range".to_string(), collections::list_range as StdFunction);

        functions.insert("map_new".to_string(), collections::map_new as StdFunction);
        functions.insert("map_len".to_string(), collections::map_len as StdFunction);
        functions.insert("map_is_empty".to_string(), collections::map_is_empty as StdFunction);
        functions.insert("map_get".to_string(), collections::map_get as StdFunction);
        functions.insert("map_insert".to_string(), collections::map_insert as StdFunction);
        functions.insert("map_remove".to_string(), collections::map_remove as StdFunction);
        functions.insert("map_contains_key".to_string(), collections::map_contains_key as StdFunction);
        functions.insert("map_keys".to_string(), collections::map_keys as StdFunction);
        functions.insert("map_values".to_string(), collections::map_values as StdFunction);

        // 迭代器函数
        functions.insert("iter_range".to_string(), iter::iter_range as StdFunction);
        functions.insert("iter_map".to_string(), iter::iter_map as StdFunction);
        functions.insert("iter_filter".to_string(), iter::iter_filter as StdFunction);
        functions.insert("iter_collect".to_string(), iter::iter_collect as StdFunction);

        // IO函数
        functions.insert("read_file".to_string(), io::read_file as StdFunction);
        functions.insert("write_file".to_string(), io::write_file as StdFunction);
        functions.insert("file_exists".to_string(), io::file_exists as StdFunction);
        functions.insert("create_dir".to_string(), io::create_dir as StdFunction);
        functions.insert("list_dir".to_string(), io::list_dir as StdFunction);

        // 时间函数
        functions.insert("now".to_string(), time::now as StdFunction);
        functions.insert("sleep".to_string(), time::sleep as StdFunction);
        functions.insert("timestamp".to_string(), time::timestamp as StdFunction);

        // 系统函数
        functions.insert("exit".to_string(), sys::exit as StdFunction);
        functions.insert("env".to_string(), sys::env as StdFunction);
        functions.insert("args".to_string(), sys::args as StdFunction);

        Self { functions }
    }

    pub fn call(&self, name: &str, args: &[StdValue]) -> Result<StdValue, StdError> {
        if let Some(func) = self.functions.get(name) {
            func(args)
        } else {
            Err(StdError::RuntimeError(format!("未找到标准库函数: {}", name)))
        }
    }

    pub fn has_function(&self, name: &str) -> bool {
        self.functions.contains_key(name)
    }
}

impl Default for StdLib {
    fn default() -> Self {
        Self::new()
    }
}
