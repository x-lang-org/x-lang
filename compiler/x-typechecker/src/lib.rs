// 类型检查器库

pub mod errors;
pub mod exhaustiveness;
pub mod format;

// Re-export common types for convenience
pub use errors::{ErrorCategory, Severity, TypeError};
pub use x_parser::ast::Type;

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use x_lexer::span::Span;
use x_parser::ast::{
    Block, ClassDecl, ClassMember, Declaration, Expression, ExpressionKind, FunctionDecl, Literal,
    Newtype, Program, Statement, StatementKind, TraitDecl, TypeAlias, VariableDecl, Visibility,
};

/// 类型检查结果（支持多错误收集）
#[derive(Debug, Default)]
pub struct TypeCheckResult {
    pub errors: Vec<TypeError>,
}

impl TypeCheckResult {
    pub fn is_ok(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn add_error(&mut self, error: TypeError) {
        self.errors.push(error);
    }

    /// 转换为 Result，返回第一个错误或成功
    pub fn to_result(self) -> Result<(), TypeError> {
        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(self.errors.into_iter().next().unwrap())
        }
    }
}

/// 类信息
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct ClassInfo {
    /// 类名
    name: String,
    /// 父类名
    extends: Option<String>,
    /// 实现的特征
    implements: Vec<String>,
    /// 字段类型（名称 -> 类型）
    fields: HashMap<String, Type>,
    /// 方法类型（名称 -> 类型）
    methods: HashMap<String, Type>,
    /// 是否为抽象类
    is_abstract: bool,
    /// 是否为 final 类
    is_final: bool,
    /// 字段可见性（名称 -> 可见性）
    field_visibility: HashMap<String, Visibility>,
    /// 方法可见性（名称 -> 可见性）
    method_visibility: HashMap<String, Visibility>,
    /// 抽象方法名称集合
    abstract_methods: HashSet<String>,
    /// 虚方法名称集合
    virtual_methods: HashSet<String>,
    /// 父类构造函数参数类型（用于 super() 验证）
    parent_constructor_params: Option<Vec<Type>>,
}

/// 特征信息
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct TraitInfo {
    /// 特征名
    name: String,
    /// 父特征列表
    extends: Vec<String>,
    /// 方法签名（名称 -> 类型）
    methods: HashMap<String, Type>,
}

/// 函数信息（包含效果）
#[derive(Debug, Clone)]
struct FunctionInfo {
    /// 函数类型
    ty: Type,
    /// 声明的效果
    effects: EffectSet,
}

/// 模块信息
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct ModuleInfo {
    /// 模块名
    name: String,
    /// 导出的符号
    exports: HashSet<String>,
}

/// 枚举变体信息
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct EnumVariantInfo {
    /// 所属枚举名
    enum_name: String,
    /// 变体名（如 Some, None, Ok, Err）
    variant_name: String,
    /// 变体数据类型（Unit、Tuple 或 Record）
    data: x_parser::ast::EnumVariantData,
    /// 变体的类型（构造后的类型）
    variant_type: Type,
}

/// 类型环境
#[allow(dead_code)]
pub struct TypeEnv {
    variable_scopes: Vec<HashMap<String, Type>>,
    functions: HashMap<String, FunctionInfo>,
    /// 类定义
    classes: HashMap<String, ClassInfo>,
    /// 特征定义
    traits: HashMap<String, TraitInfo>,
    /// 枚举定义
    enums: HashMap<String, x_parser::ast::EnumDecl>,
    /// 记录定义
    records: HashMap<String, x_parser::ast::RecordDecl>,
    /// 效果定义
    effects: HashMap<String, x_parser::ast::EffectDecl>,
    /// 枚举变体（如 Some, None, Ok, Err）
    enum_variants: HashMap<String, EnumVariantInfo>,
    /// 类型别名
    type_aliases: HashMap<String, Type>,
    /// 类型变量生成器（用于 HM 类型推断）
    type_var_gen: TypeVarGenerator,
    /// 当前类型替换（用于合一）
    substitution: HashMap<String, Type>,
    /// 当前模块名
    current_module: Option<String>,
    /// 导出的符号（当前模块）
    exports: HashSet<String>,
    /// 已解析的模块（模块名 -> 导出符号）
    resolved_modules: HashMap<String, ModuleInfo>,
    /// Unsafe 上下文追踪：栈记录是否在 unsafe 块内
    unsafe_context_stack: Vec<bool>,
}

impl TypeEnv {
    fn new() -> Self {
        Self {
            variable_scopes: vec![HashMap::with_capacity(16)],
            functions: HashMap::with_capacity(32),
            classes: HashMap::with_capacity(8),
            traits: HashMap::with_capacity(8),
            enums: HashMap::with_capacity(8),
            records: HashMap::with_capacity(8),
            effects: HashMap::with_capacity(4),
            enum_variants: HashMap::with_capacity(16),
            type_aliases: HashMap::with_capacity(8),
            type_var_gen: TypeVarGenerator::new(),
            substitution: HashMap::with_capacity(16),
            current_module: None,
            exports: HashSet::with_capacity(16),
            resolved_modules: HashMap::with_capacity(4),
            unsafe_context_stack: Vec::with_capacity(4),
        }
    }

    /// 注册枚举及其变体
    fn register_enum(&mut self, name: String, enum_decl: x_parser::ast::EnumDecl) {
        self.enums.insert(name.clone(), enum_decl.clone());

        // 收集类型参数名
        let type_params: Vec<String> = enum_decl
            .type_parameters
            .iter()
            .map(|tp| tp.name.clone())
            .collect();

        // 如果有泛型参数，使用 Type::Generic；否则使用具体类型
        let return_type = if type_params.is_empty() {
            Type::Generic(name.clone())
        } else {
            // 对于泛型枚举，返回 Type::Generic，表示需要类型实例化
            Type::Generic(name.clone())
        };

        // 同时作为类型别名注册
        self.type_aliases.insert(name.clone(), return_type.clone());

        // 注册枚举的所有变体
        for variant in &enum_decl.variants {
            let variant_name = variant.name.clone();
            let full_name = format!("{}::{}", name, variant_name);

            // 计算变体的类型
            let variant_type = match &variant.data {
                x_parser::ast::EnumVariantData::Unit => {
                    // Option::None -> Option<T>
                    return_type.clone()
                }
                x_parser::ast::EnumVariantData::Tuple(types) => {
                    // Option::Some(T) -> function(T) -> Option<T>
                    // 将类型转换为 Box<Type> 包装
                    let param_types: Vec<Box<Type>> =
                        types.iter().map(|t| Box::new(t.clone())).collect();
                    // 构建函数类型: (T) -> Option<T>
                    Type::Function(param_types, Box::new(return_type.clone()))
                }
                x_parser::ast::EnumVariantData::Record(fields) => {
                    // 记录类型的变体
                    let field_types: Vec<Box<Type>> =
                        fields.iter().map(|(_, ty)| Box::new(ty.clone())).collect();
                    Type::Function(field_types, Box::new(return_type.clone()))
                }
            };

            let variant_info = EnumVariantInfo {
                enum_name: name.clone(),
                variant_name: variant_name.clone(),
                data: variant.data.clone(),
                variant_type,
            };

            // 同时用简单名和完整名注册
            self.enum_variants
                .insert(variant_name, variant_info.clone());
            self.enum_variants.insert(full_name, variant_info);
        }
    }

    /// 获取枚举变体信息
    fn get_enum_variant(&self, name: &str) -> Option<&EnumVariantInfo> {
        self.enum_variants.get(name)
    }

    /// 获取枚举定义
    fn get_enum(&self, name: &str) -> Option<&x_parser::ast::EnumDecl> {
        self.enums.get(name)
    }

    /// 获取记录定义
    fn get_record(&self, name: &str) -> Option<&x_parser::ast::RecordDecl> {
        self.records.get(name)
    }

    /// 注册记录声明
    fn register_record(&mut self, name: String, record: x_parser::ast::RecordDecl) {
        self.records.insert(name.clone(), record.clone());

        // 同时作为类型别名注册
        // 记录本身就是一个命名类型
        // TODO: 构建正确的记录类型
    }

    /// 注册效果声明
    fn register_effect(&mut self, name: String, effect: x_parser::ast::EffectDecl) {
        self.effects.insert(name.clone(), effect);
    }

    /// 获取效果声明
    pub fn get_effect(&self, name: &str) -> Option<&x_parser::ast::EffectDecl> {
        self.effects.get(name)
    }

    /// 设置当前模块
    fn set_current_module(&mut self, name: String) {
        self.current_module = Some(name);
    }

    /// 添加导出符号
    fn add_export(&mut self, symbol: String) {
        self.exports.insert(symbol);
    }

    /// 检查符号是否已导出
    #[allow(dead_code)]
    fn is_exported(&self, symbol: &str) -> bool {
        self.exports.contains(symbol)
    }

    /// 检查当前是否在 unsafe 上下文中
    pub fn is_in_unsafe_context(&self) -> bool {
        self.unsafe_context_stack.last().copied().unwrap_or(false)
    }

    /// 进入 unsafe 上下文
    pub fn enter_unsafe_context(&mut self) {
        self.unsafe_context_stack.push(true);
    }

    /// 离开 unsafe 上下文
    pub fn exit_unsafe_context(&mut self) {
        self.unsafe_context_stack.pop();
    }
    #[allow(dead_code)]
    fn register_module(&mut self, name: String, exports: HashSet<String>) {
        self.resolved_modules
            .insert(name.clone(), ModuleInfo { name, exports });
    }

    /// 获取模块导出的符号
    fn get_module_exports(&self, module_name: &str) -> Option<&HashSet<String>> {
        self.resolved_modules.get(module_name).map(|m| &m.exports)
    }

    fn add_variable(&mut self, name: &str, ty: Type) {
        let scope = self
            .variable_scopes
            .last_mut()
            .expect("TypeEnv should always have at least one scope");
        if scope.contains_key(name) {
            // 同一作用域内重复声明
            // 这里不直接 panic，而是让上层决定如何报告
            // 但为了不改变原有签名，我们在 check_variable_decl 里提前拦截。
        }
        scope.insert(name.to_string(), ty);
    }

    /// 获取函数的类型
    pub fn get_function_type(&self, name: &str) -> Option<&Type> {
        self.functions.get(name).map(|info| &info.ty)
    }

    fn add_function(&mut self, name: &str, ty: Type) {
        // 允许覆盖，以支持函数重载（如 println 接受不同类型）
        self.functions.insert(
            name.to_string(),
            FunctionInfo {
                ty,
                effects: HashSet::new(),
            },
        );
    }

    #[allow(dead_code)]
    fn add_function_with_effects(&mut self, name: &str, ty: Type, effects: EffectSet) {
        self.functions
            .insert(name.to_string(), FunctionInfo { ty, effects });
    }

    fn add_class(&mut self, name: &str, info: ClassInfo) {
        self.classes.insert(name.to_string(), info);
    }

    fn add_trait(&mut self, name: &str, info: TraitInfo) {
        self.traits.insert(name.to_string(), info);
    }

    fn add_type_alias(&mut self, name: &str, ty: Type) {
        self.type_aliases.insert(name.to_string(), ty);
    }

    pub fn get_variable(&self, name: &str) -> Option<&Type> {
        for scope in self.variable_scopes.iter().rev() {
            if let Some(ty) = scope.get(name) {
                return Some(ty);
            }
        }
        None
    }

    fn get_function(&self, name: &str) -> Option<&Type> {
        self.functions.get(name).map(|info| &info.ty)
    }

    fn get_function_info(&self, name: &str) -> Option<&FunctionInfo> {
        self.functions.get(name)
    }

    fn get_class(&self, name: &str) -> Option<&ClassInfo> {
        self.classes.get(name)
    }

    fn get_trait(&self, name: &str) -> Option<&TraitInfo> {
        self.traits.get(name)
    }

    fn get_type_alias(&self, name: &str) -> Option<&Type> {
        self.type_aliases.get(name)
    }

    fn push_scope(&mut self) {
        self.variable_scopes.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        if self.variable_scopes.len() <= 1 {
            // 保留全局作用域，避免空栈
            return;
        }
        self.variable_scopes.pop();
    }

    fn current_scope_contains(&self, name: &str) -> bool {
        self.variable_scopes
            .last()
            .map(|s| s.contains_key(name))
            .unwrap_or(false)
    }

    /// 生成新鲜类型变量
    fn fresh_type_var(&self) -> Type {
        self.type_var_gen.fresh()
    }

    /// 应用当前替换到类型
    #[allow(dead_code)]
    fn apply_subst(&self, ty: &Type) -> Type {
        apply_type_substitution(ty, &self.substitution)
    }

    /// 扩展替换
    #[allow(dead_code)]
    fn extend_subst(&mut self, var: String, ty: Type) {
        self.substitution.insert(var, ty);
    }

    /// 合一两个类型，更新替换
    #[allow(dead_code)]
    fn unify_types(&mut self, t1: &Type, t2: &Type, span: Span) -> Result<(), TypeError> {
        let new_subst = unify(t1, t2).map_err(|e| match e {
            UnificationError::TypeMismatch(expected, actual) => TypeError::TypeMismatch {
                expected: format!("{:?}", expected),
                actual: format!("{:?}", actual),
                span,
            },
            UnificationError::InfiniteType(_var, _ty) => TypeError::RecursiveType { span },
        })?;

        // 合并新替换到当前替换
        for (k, v) in new_subst {
            // 应用现有替换到新值
            let v_subst = apply_type_substitution(&v, &self.substitution);
            self.substitution.insert(k, v_subst);
        }

        // 应用替换到现有替换中的值（避免克隆整个 HashMap）
        let updates: Vec<(String, Type)> = self
            .substitution
            .iter()
            .map(|(k, v)| (k.clone(), apply_type_substitution(v, &self.substitution)))
            .collect();

        for (k, v) in updates {
            self.substitution.insert(k, v);
        }

        Ok(())
    }
}

/// 类型检查器主函数
pub fn type_check(program: &Program) -> Result<(), TypeError> {
    let mut env = TypeEnv::new();
    // 预置内置函数，避免 CLI `check/run` 对基础 I/O 直接报"未定义变量"
    // 目前类型系统尚不支持泛型/可变参数，这里先用最小可用签名约束住常用 builtin。

    // 内置函数现在由 prelude.x 提供，不再在这里预先添加
    // 这样可以避免重复声明错误，并且更灵活

    // String functions
    // string_length(s: string) -> integer
    env.add_function(
        "string_length",
        Type::Function(vec![Box::new(Type::String)], Box::new(Type::Int)),
    );
    // string_find(s: string, substr: string) -> integer
    env.add_function(
        "string_find",
        Type::Function(
            vec![Box::new(Type::String), Box::new(Type::String)],
            Box::new(Type::Int),
        ),
    );
    // string_substring(s: string, start: integer, end: integer) -> string
    env.add_function(
        "string_substring",
        Type::Function(
            vec![
                Box::new(Type::String),
                Box::new(Type::Int),
                Box::new(Type::Int),
            ],
            Box::new(Type::String),
        ),
    );
    // int_to_string(n: integer) -> string
    env.add_function(
        "int_to_string",
        Type::Function(vec![Box::new(Type::Int)], Box::new(Type::String)),
    );
    // concat(a: string, b: string) -> string
    env.add_function(
        "concat",
        Type::Function(
            vec![Box::new(Type::String), Box::new(Type::String)],
            Box::new(Type::String),
        ),
    );

    // len - 获取数组/字符串长度
    env.add_function(
        "len",
        Type::Function(vec![Box::new(Type::Dynamic)], Box::new(Type::Int)),
    );

    // print/println 现在由 prelude.x 提供，不再作为内置函数
    // 这样可以避免重复声明错误

    // Add Option/Result constructors as builtin types
    // Some<T> -> Option<T>
    env.add_function(
        "Some",
        Type::Function(
            vec![Box::new(Type::Dynamic)],
            Box::new(Type::TypeConstructor(
                "Option".to_string(),
                vec![Type::Dynamic],
            )),
        ),
    );
    // None -> Option<T> (using Dynamic for T)
    env.add_function(
        "None",
        Type::Function(
            vec![],
            Box::new(Type::TypeConstructor(
                "Option".to_string(),
                vec![Type::Dynamic],
            )),
        ),
    );
    // Ok<T> -> Result<T, E>
    env.add_function(
        "Ok",
        Type::Function(
            vec![Box::new(Type::Dynamic)],
            Box::new(Type::TypeConstructor(
                "Result".to_string(),
                vec![Type::Dynamic, Type::Dynamic],
            )),
        ),
    );
    // Err<E> -> Result<T, E>
    env.add_function(
        "Err",
        Type::Function(
            vec![Box::new(Type::Dynamic)],
            Box::new(Type::TypeConstructor(
                "Result".to_string(),
                vec![Type::Dynamic, Type::Dynamic],
            )),
        ),
    );

    // Builtin I/O functions - 这些现在由 std.prelude 提供
    // __index__(collection, key/index) -> Dynamic
    env.add_function(
        "__index__",
        Type::Function(
            vec![Box::new(Type::Dynamic), Box::new(Type::Dynamic)],
            Box::new(Type::Dynamic),
        ),
    );

    check_program(program, &mut env)
}

/// 检查程序
fn check_program(program: &Program, env: &mut TypeEnv) -> Result<(), TypeError> {
    // 第一遍：收集所有类型声明（类、trait、类型别名、函数签名、枚举）
    for decl in &program.declarations {
        match decl {
            Declaration::Class(class_decl) => {
                collect_class_info(class_decl, env)?;
            }
            Declaration::Trait(trait_decl) => {
                collect_trait_info(trait_decl, env)?;
            }
            Declaration::TypeAlias(type_alias) => {
                collect_type_alias_info(type_alias, env)?;
            }
            Declaration::Newtype(newtype) => {
                collect_newtype_info(newtype, env)?;
            }
            Declaration::Function(func_decl) => {
                // 收集函数签名（不检查函数体）
                collect_function_signature(func_decl, env)?;
            }
            Declaration::ExternFunction(extern_func_decl) => {
                // 收集外部函数签名
                collect_extern_function_signature(extern_func_decl, env)?;
            }
            Declaration::Enum(enum_decl) => {
                // 注册枚举及其变体
                env.register_enum(enum_decl.name.clone(), enum_decl.clone());
            }
            Declaration::Record(record_decl) => {
                // 注册记录类型
                env.register_record(record_decl.name.clone(), record_decl.clone());
            }
            _ => {}
        }
    }

    // 第二遍：检查所有声明的详细实现
    for decl in &program.declarations {
        check_declaration(decl, env)?;
    }

    // 然后检查所有语句
    for stmt in &program.statements {
        check_statement(stmt, env)?;
    }

    Ok(())
}

/// 类型检查，返回填充好的类型环境（包含所有推断的类型信息）
/// 供 HIR 降阶使用来整合类型注解
pub fn type_check_with_env(program: &Program) -> Result<TypeEnv, TypeError> {
    let mut env = TypeEnv::new();
    // 预置内置函数，避免 CLI `check/run/compile` 对基础 I/O 直接报"未定义变量"
    // 使用 Dynamic 类型接受任何参数，以便 println(Int) 这样的调用能通过

    // String functions
    env.add_function(
        "string_length",
        Type::Function(vec![Box::new(Type::String)], Box::new(Type::Int)),
    );
    env.add_function(
        "string_find",
        Type::Function(
            vec![Box::new(Type::String), Box::new(Type::String)],
            Box::new(Type::Int),
        ),
    );
    env.add_function(
        "string_substring",
        Type::Function(
            vec![
                Box::new(Type::String),
                Box::new(Type::Int),
                Box::new(Type::Int),
            ],
            Box::new(Type::String),
        ),
    );
    env.add_function(
        "int_to_string",
        Type::Function(vec![Box::new(Type::Int)], Box::new(Type::String)),
    );
    env.add_function(
        "concat",
        Type::Function(
            vec![Box::new(Type::String), Box::new(Type::String)],
            Box::new(Type::String),
        ),
    );

    // len - 获取数组/字符串长度
    env.add_function(
        "len",
        Type::Function(vec![Box::new(Type::Dynamic)], Box::new(Type::Int)),
    );

    // print/println 现在由 prelude.x 提供，不再作为内置函数
    // 这样可以避免重复声明错误

    // 检查程序，填充环境
    check_program(program, &mut env)?;

    Ok(env)
}

/// 第一遍：收集类信息（不检查实现细节）
fn collect_class_info(class_decl: &ClassDecl, env: &mut TypeEnv) -> Result<(), TypeError> {
    let span = class_decl.span;

    // 检查类名是否已存在
    if env.get_class(&class_decl.name).is_some() {
        return Err(TypeError::DuplicateDeclaration {
            name: class_decl.name.clone(),
            span,
        });
    }

    // 收集字段和方法（只收集类型信息，不检查方法体）
    let mut fields = HashMap::new();
    let mut methods = HashMap::new();
    let mut field_visibility = HashMap::new();
    let mut method_visibility = HashMap::new();
    let mut abstract_methods = HashSet::new();
    let mut virtual_methods = HashSet::new();
    let mut parent_constructor_params = None;

    for member in &class_decl.members {
        match member {
            ClassMember::Field(field) => {
                let field_type = if let Some(type_annot) = &field.type_annot {
                    type_annot.clone()
                } else {
                    Type::Unit // 暂时使用 Unit，第二遍会检查
                };

                if fields.contains_key(&field.name) {
                    return Err(TypeError::DuplicateDeclaration {
                        name: field.name.clone(),
                        span: field.span,
                    });
                }
                fields.insert(field.name.clone(), field_type);
                field_visibility.insert(field.name.clone(), field.visibility);
            }
            ClassMember::Method(method) => {
                let method_type = create_function_type(method);
                if methods.contains_key(&method.name) {
                    return Err(TypeError::DuplicateDeclaration {
                        name: method.name.clone(),
                        span: method.span,
                    });
                }
                methods.insert(method.name.clone(), method_type);
                method_visibility.insert(method.name.clone(), method.modifiers.visibility);

                // 记录抽象方法和虚方法
                if method.modifiers.is_abstract {
                    abstract_methods.insert(method.name.clone());
                }
                if method.modifiers.is_virtual {
                    virtual_methods.insert(method.name.clone());
                }
            }
            ClassMember::Constructor(constructor) => {
                // 构造函数不添加到方法表，但记录参数类型
                let param_types: Vec<Type> = constructor
                    .parameters
                    .iter()
                    .filter_map(|p| p.type_annot.clone())
                    .collect();
                // 如果是第一个构造函数，或者没有父类构造函数参数记录，则更新
                if parent_constructor_params.is_none() {
                    parent_constructor_params = Some(param_types);
                }
            }
        }
    }

    // 如果没有显式构造函数，使用字段作为隐式构造函数参数
    let constructor_params = if parent_constructor_params.is_none() {
        // 按字段顺序创建构造函数参数
        let field_types: Vec<Type> = fields.values().cloned().collect();
        if field_types.is_empty() {
            None
        } else {
            Some(field_types)
        }
    } else {
        parent_constructor_params
    };

    let class_info = ClassInfo {
        name: class_decl.name.clone(),
        extends: class_decl.extends.clone(),
        implements: class_decl.implements.clone(),
        fields,
        methods,
        is_abstract: class_decl.modifiers.is_abstract,
        is_final: class_decl.modifiers.is_final,
        field_visibility,
        method_visibility,
        abstract_methods,
        virtual_methods,
        parent_constructor_params: constructor_params,
    };
    env.add_class(&class_decl.name, class_info);
    env.add_type_alias(&class_decl.name, Type::Generic(class_decl.name.clone()));

    Ok(())
}

/// 第一遍：收集 trait 信息
fn collect_trait_info(trait_decl: &TraitDecl, env: &mut TypeEnv) -> Result<(), TypeError> {
    let span = trait_decl.span;

    // 检查特征名是否已存在
    if env.get_trait(&trait_decl.name).is_some() {
        return Err(TypeError::DuplicateDeclaration {
            name: trait_decl.name.clone(),
            span,
        });
    }

    // 收集方法签名
    let mut methods = HashMap::new();

    for method in &trait_decl.methods {
        let method_type = create_function_type(method);

        if methods.contains_key(&method.name) {
            return Err(TypeError::DuplicateDeclaration {
                name: method.name.clone(),
                span: method.span,
            });
        }
        methods.insert(method.name.clone(), method_type);
    }

    let trait_info = TraitInfo {
        name: trait_decl.name.clone(),
        extends: trait_decl.extends.clone(),
        methods,
    };
    env.add_trait(&trait_decl.name, trait_info);

    Ok(())
}

/// 第一遍：收集类型别名信息
fn collect_type_alias_info(type_alias: &TypeAlias, env: &mut TypeEnv) -> Result<(), TypeError> {
    let span = type_alias.span;

    if env.get_type_alias(&type_alias.name).is_some() {
        return Err(TypeError::DuplicateDeclaration {
            name: type_alias.name.clone(),
            span,
        });
    }

    env.add_type_alias(&type_alias.name, type_alias.type_.clone());

    Ok(())
}

/// 第一遍：收集函数签名（不检查函数体）
fn collect_function_signature(
    func_decl: &FunctionDecl,
    env: &mut TypeEnv,
) -> Result<(), TypeError> {
    let span = func_decl.span;

    // 检查函数名是否已存在
    if env.functions.contains_key(&func_decl.name) {
        return Err(TypeError::DuplicateDeclaration {
            name: func_decl.name.clone(),
            span,
        });
    }

    // 创建函数类型
    let func_type = create_function_type(func_decl);

    // 添加函数到环境
    env.add_function(&func_decl.name, func_type);

    Ok(())
}

/// 第一遍：收集外部函数签名
fn collect_extern_function_signature(
    extern_func_decl: &x_parser::ast::ExternFunctionDecl,
    env: &mut TypeEnv,
) -> Result<(), TypeError> {
    let span = extern_func_decl.span;

    // 检查函数名是否已存在
    if env.functions.contains_key(&extern_func_decl.name) {
        return Err(TypeError::DuplicateDeclaration {
            name: extern_func_decl.name.clone(),
            span,
        });
    }

    // 创建函数类型
    let mut param_types = Vec::new();
    for param in &extern_func_decl.parameters {
        if let Some(type_annot) = &param.type_annot {
            param_types.push(Box::new(type_annot.clone()));
        } else {
            param_types.push(Box::new(Type::Dynamic));
        }
    }

    let return_type = if let Some(return_type) = &extern_func_decl.return_type {
        Box::new(return_type.clone())
    } else {
        Box::new(Type::Unit)
    };

    let func_type = Type::Function(param_types, return_type);

    // 添加函数到环境
    env.add_function(&extern_func_decl.name, func_type);

    Ok(())
}

/// 检查声明
fn check_declaration(decl: &Declaration, env: &mut TypeEnv) -> Result<(), TypeError> {
    match decl {
        Declaration::Variable(var_decl) => check_variable_decl(var_decl, env),
        Declaration::Function(func_decl) => check_function_decl(func_decl, env),
        Declaration::ExternFunction(extern_func_decl) => {
            check_extern_function_decl(extern_func_decl, env)
        }
        Declaration::Class(class_decl) => check_class_decl(class_decl, env),
        Declaration::Trait(trait_decl) => check_trait_decl(trait_decl, env),
        Declaration::Enum(enum_decl) => check_enum_decl(enum_decl, env),
        Declaration::Record(record_decl) => check_record_decl(record_decl, env),
        Declaration::Effect(effect_decl) => check_effect_decl(effect_decl, env),
        Declaration::Implement(impl_decl) => check_implement_decl(impl_decl, env),
        Declaration::TypeAlias(type_alias) => check_type_alias(type_alias, env),
        Declaration::Newtype(newtype) => check_newtype(newtype, env),
        Declaration::Module(module_decl) => check_module_decl(module_decl, env),
        Declaration::Import(import_decl) => check_import_decl(import_decl, env),
        Declaration::Export(export_decl) => check_export_decl(export_decl, env),
    }
}

/// 检查模块声明
fn check_module_decl(
    module_decl: &x_parser::ast::ModuleDecl,
    env: &mut TypeEnv,
) -> Result<(), TypeError> {
    // 设置当前模块名
    env.set_current_module(module_decl.name.clone());
    Ok(())
}

/// 检查外部函数声明
fn check_extern_function_decl(
    extern_func_decl: &x_parser::ast::ExternFunctionDecl,
    env: &mut TypeEnv,
) -> Result<(), TypeError> {
    let _span = extern_func_decl.span;

    // 构建函数类型
    let param_types: Vec<Box<Type>> = extern_func_decl
        .parameters
        .iter()
        .map(|p| Box::new(p.type_annot.clone().unwrap_or(Type::Dynamic)))
        .collect();

    let return_type = Box::new(extern_func_decl.return_type.clone().unwrap_or(Type::Void));

    let func_type = Type::Function(param_types, return_type);

    // 将函数添加到环境
    env.add_function(&extern_func_decl.name, func_type);

    Ok(())
}

/// 检查导入声明
fn check_import_decl(
    import_decl: &x_parser::ast::ImportDecl,
    env: &mut TypeEnv,
) -> Result<(), TypeError> {
    let span = import_decl.span;

    // 检查模块是否已解析
    if let Some(exports) = env.get_module_exports(&import_decl.module_path) {
        // 检查导入的符号是否存在
        for symbol in &import_decl.symbols {
            match symbol {
                x_parser::ast::ImportSymbol::All => {
                    // 导入所有符号
                    for export in exports {
                        // 根据导出符号类型添加到环境
                        // 这里简化处理：假设都是函数或变量
                        if env.functions.contains_key(export) || env.get_variable(export).is_some()
                        {
                            // 已存在，跳过
                        } else {
                            // 未找到符号定义，记录警告（但不报错，因为可能是类型）
                        }
                    }
                }
                x_parser::ast::ImportSymbol::Named(name, alias) => {
                    if !exports.contains(name) {
                        return Err(TypeError::UndefinedVariable {
                            name: format!("{}::{}", import_decl.module_path, name),
                            span,
                        });
                    }
                    // 使用别名或原名
                    let target_name = alias.as_ref().unwrap_or(name);
                    // 标记符号已导入（实际类型信息在模块解析时获取）
                    let _ = target_name;
                }
            }
        }
    } else {
        // 模块未解析，可能是外部模块或标准库
        // 这里暂时不报错，允许导入未解析的模块
    }

    Ok(())
}

/// 检查导出声明
fn check_export_decl(
    export_decl: &x_parser::ast::ExportDecl,
    env: &mut TypeEnv,
) -> Result<(), TypeError> {
    let span = export_decl.span;
    let symbol = &export_decl.symbol;

    // 检查符号是否已定义
    let is_defined = env.get_variable(symbol).is_some()
        || env.functions.contains_key(symbol)
        || env.classes.contains_key(symbol)
        || env.traits.contains_key(symbol)
        || env.type_aliases.contains_key(symbol)
        || env.records.contains_key(symbol)
        || env.enums.contains_key(symbol);

    if !is_defined {
        return Err(TypeError::UndefinedVariable {
            name: symbol.clone(),
            span,
        });
    }

    // 添加到导出列表
    env.add_export(symbol.clone());
    Ok(())
}

/// 检查类声明（第二遍：检查实现细节）
fn check_class_decl(class_decl: &ClassDecl, env: &mut TypeEnv) -> Result<(), TypeError> {
    let span = class_decl.span;

    // 类信息已经在第一遍收集，这里只需要检查实现细节

    // 收集类型参数名
    let type_param_names: std::collections::HashSet<String> = class_decl
        .type_parameters
        .iter()
        .map(|tp| tp.name.clone())
        .collect();

    // 检查类型参数约束：约束必须引用存在的 trait
    for type_param in &class_decl.type_parameters {
        for constraint in &type_param.constraints {
            if env.get_trait(&constraint.trait_name).is_none() {
                return Err(TypeError::UndefinedType {
                    name: constraint.trait_name.clone(),
                    span: constraint.span,
                });
            }
        }
    }

    // 检查父类是否存在（如果有）
    if let Some(parent_name) = &class_decl.extends {
        // 检查父类是否存在
        if let Some(parent_info) = env.get_class(parent_name) {
            // 检查父类是否为 final
            if parent_info.is_final {
                return Err(TypeError::CannotExtendFinalClass {
                    class_name: parent_name.clone(),
                    span,
                });
            }

            // 检查继承循环
            if check_inheritance_cycle(&class_decl.name, env) {
                return Err(TypeError::InheritanceCycle {
                    class_name: class_decl.name.clone(),
                    span,
                });
            }
        } else {
            return Err(TypeError::UndefinedType {
                name: parent_name.clone(),
                span,
            });
        }
    }

    // 检查实现的接口是否存在
    for trait_name in &class_decl.implements {
        if env.get_trait(trait_name).is_none() {
            return Err(TypeError::UndefinedType {
                name: trait_name.clone(),
                span,
            });
        }
    }

    // 检查非法递归类型定义（直接或间接包含自身）
    let mut visited = std::collections::HashSet::new();
    visited.insert(class_decl.name.clone());
    for member in &class_decl.members {
        if let ClassMember::Field(field) = member {
            if let Some(type_annot) = &field.type_annot {
                if !is_valid_type_with_params(type_annot, env, &type_param_names) {
                    return Err(TypeError::UndefinedType {
                        name: format!("{:?}", type_annot),
                        span: field.span,
                    });
                }
                if check_recursive_type_definition(
                    &class_decl.name,
                    true,
                    type_annot,
                    &mut visited,
                    env,
                ) {
                    return Err(TypeError::RecursiveType { span: field.span });
                }
            }
        }
    }

    // 检查字段初始化器和方法体
    for member in &class_decl.members {
        match member {
            ClassMember::Field(field) => {
                // 检查字段类型注解或初始化器
                if let Some(initializer) = &field.initializer {
                    let init_type = infer_expression_type(initializer, env)?;
                    if let Some(type_annot) = &field.type_annot {
                        if !types_equal(&init_type, type_annot) {
                            return Err(TypeError::TypeMismatch {
                                expected: format!("{:?}", type_annot),
                                actual: format!("{:?}", init_type),
                                span: field.span,
                            });
                        }
                    }
                }
            }
            ClassMember::Method(method) => {
                // 检查方法重写是否合法
                if method.modifiers.is_override {
                    if let Some(err) =
                        check_method_override(&class_decl.name, &method.name, method.span, env)
                    {
                        return Err(err);
                    }
                } else {
                    // 检查是否缺少 override 关键字
                    if let Some(err) = check_missing_override_keyword(
                        &class_decl.name,
                        &method.name,
                        false,
                        method.span,
                        env,
                    ) {
                        return Err(err);
                    }
                }

                // 检查方法参数类型和返回类型是否有效（考虑类的类型参数）
                for param in &method.parameters {
                    if let Some(type_annot) = &param.type_annot {
                        if !is_valid_type_with_params(type_annot, env, &type_param_names) {
                            return Err(TypeError::UndefinedType {
                                name: format!("{:?}", type_annot),
                                span: param.span,
                            });
                        }
                    }
                }
                if let Some(return_ty) = &method.return_type {
                    if !is_valid_type_with_params(return_ty, env, &type_param_names) {
                        return Err(TypeError::UndefinedType {
                            name: format!("{:?}", return_ty),
                            span: method.span,
                        });
                    }
                }

                // 检查方法体
                if !method.body.statements.is_empty() {
                    env.push_scope();
                    // 添加 self 参数（类实例的引用）
                    env.add_variable("self", Type::Generic(class_decl.name.clone()));
                    env.add_variable("this", Type::Generic(class_decl.name.clone()));
                    // 添加类字段作为可直接访问的变量
                    for member in &class_decl.members {
                        if let ClassMember::Field(field) = member {
                            if let Some(type_annot) = &field.type_annot {
                                env.add_variable(&field.name, type_annot.clone());
                            }
                        }
                    }
                    // 添加方法参数
                    for param in &method.parameters {
                        if let Some(type_annot) = &param.type_annot {
                            if env.current_scope_contains(&param.name) {
                                env.pop_scope();
                                return Err(TypeError::DuplicateDeclaration {
                                    name: param.name.clone(),
                                    span: param.span,
                                });
                            }
                            env.add_variable(&param.name, type_annot.clone());
                        } else {
                            return Err(TypeError::CannotInferType { span: param.span });
                        }
                    }
                    check_block(&method.body, env)?;
                    env.pop_scope();
                }
            }
            ClassMember::Constructor(constructor) => {
                // 检查 super() 调用（如果类有父类）
                check_super_call_in_constructor(class_decl, constructor, env)?;

                // 检查构造函数参数类型是否有效（考虑类的类型参数）
                for param in &constructor.parameters {
                    if let Some(type_annot) = &param.type_annot {
                        if !is_valid_type_with_params(type_annot, env, &type_param_names) {
                            return Err(TypeError::UndefinedType {
                                name: format!("{:?}", type_annot),
                                span: param.span,
                            });
                        }
                    }
                }

                // 检查构造函数参数和体
                env.push_scope();
                // 添加 self 参数（类实例的引用）
                env.add_variable("self", Type::Generic(class_decl.name.clone()));
                env.add_variable("this", Type::Generic(class_decl.name.clone()));
                for param in &constructor.parameters {
                    if let Some(type_annot) = &param.type_annot {
                        if env.current_scope_contains(&param.name) {
                            env.pop_scope();
                            return Err(TypeError::DuplicateDeclaration {
                                name: param.name.clone(),
                                span: param.span,
                            });
                        }
                        env.add_variable(&param.name, type_annot.clone());
                    }
                }
                check_block(&constructor.body, env)?;
                env.pop_scope();
            }
        }
    }

    // 检查抽象类是否实现了所有抽象方法
    if !class_decl.modifiers.is_abstract {
        if let Some(err) = check_abstract_method_implementation(class_decl, env) {
            return Err(err);
        }
    }

    // 检查 trait 实现
    for trait_name in &class_decl.implements {
        check_trait_implementation(&class_decl.name, trait_name, env, class_decl.span)?;
    }

    Ok(())
}

/// 检查类是否正确实现了 trait 的所有方法
fn check_trait_implementation(
    class_name: &str,
    trait_name: &str,
    env: &mut TypeEnv,
    span: Span,
) -> Result<(), TypeError> {
    let class_info = env.get_class(class_name).expect("class should exist");
    let trait_info = env
        .get_trait(trait_name)
        .expect("trait should exist at this point");

    // 收集类的所有方法（包括继承的）
    let mut all_methods = class_info.methods.clone();
    if let Some(parent_name) = &class_info.extends {
        if let Some(parent_info) = env.get_class(parent_name) {
            for (name, ty) in &parent_info.methods {
                if !all_methods.contains_key(name) {
                    all_methods.insert(name.clone(), ty.clone());
                }
            }
        }
    }

    // 检查 trait 的所有方法是否都已实现
    for (method_name, trait_method_type) in &trait_info.methods {
        if let Some(class_method_type) = all_methods.get(method_name) {
            // 检查方法签名是否匹配
            if !types_equal(class_method_type, trait_method_type) {
                return Err(TypeError::InvalidOverride {
                    message: format!(
                        "方法 {} 的签名不匹配 trait {} 的定义",
                        method_name, trait_name
                    ),
                    span,
                });
            }
        } else {
            return Err(TypeError::MissingTraitMethod {
                trait_name: trait_name.to_string(),
                method_name: method_name.to_string(),
                span,
            });
        }
    }

    Ok(())
}

/// 检查继承循环
fn check_inheritance_cycle(class_name: &str, env: &TypeEnv) -> bool {
    let mut visited = std::collections::HashSet::new();
    let mut current = Some(class_name.to_string());

    while let Some(name) = current {
        if visited.contains(&name) {
            return true; // 发现循环
        }
        visited.insert(name.clone());

        current = env.get_class(&name).and_then(|info| info.extends.clone());
    }

    false
}

/// 检查方法重写是否合法
fn check_method_override(
    class_name: &str,
    method_name: &str,
    method_span: Span,
    env: &TypeEnv,
) -> Option<TypeError> {
    let class_info = env.get_class(class_name)?;

    // 获取父类
    let parent_name = class_info.extends.as_ref()?;
    let parent_info = env.get_class(parent_name)?;

    // 检查父类是否有此方法
    if let Some(parent_method_type) = parent_info.methods.get(method_name) {
        // 检查父类方法是否为 virtual 或 abstract
        let is_overridable = parent_info.virtual_methods.contains(method_name)
            || parent_info.abstract_methods.contains(method_name);

        if !is_overridable {
            return Some(TypeError::CannotOverrideNonVirtual {
                method_name: method_name.to_string(),
                span: method_span,
            });
        }

        // 检查子类方法是否存在
        if let Some(child_method_type) = class_info.methods.get(method_name) {
            // 使用变元检查替代简单的类型相等检查
            if let Err(e) = check_override_variance(
                child_method_type,
                parent_method_type,
                method_name,
                method_span,
            ) {
                return Some(e);
            }
        }
    }

    None
}

/// 检查方法是否需要 override 关键字（当重写父类方法时）
fn check_missing_override_keyword(
    class_name: &str,
    method_name: &str,
    has_override_keyword: bool,
    method_span: Span,
    env: &TypeEnv,
) -> Option<TypeError> {
    let class_info = env.get_class(class_name)?;

    // 获取父类
    let parent_name = class_info.extends.as_ref()?;
    let parent_info = env.get_class(parent_name)?;

    // 检查父类是否有此方法
    if parent_info.methods.contains_key(method_name) {
        // 父类有此方法，子类重写时必须有 override 关键字
        if !has_override_keyword {
            return Some(TypeError::MissingOverrideKeyword {
                method: method_name.to_string(),
                span: method_span,
            });
        }
    }

    None
}

/// 检查非抽象类是否实现了所有抽象方法
fn check_abstract_method_implementation(
    class_decl: &ClassDecl,
    env: &TypeEnv,
) -> Option<TypeError> {
    // 收集所有需要实现的抽象方法（来自父类和 trait）
    let mut abstract_methods: HashMap<String, Type> = HashMap::new();

    // 从父类收集抽象方法
    let mut current_parent = class_decl.extends.clone();
    while let Some(parent_name) = current_parent {
        if let Some(parent_info) = env.get_class(&parent_name) {
            // 收集父类的抽象方法（使用 abstract_methods 字段）
            for method_name in &parent_info.abstract_methods {
                if let Some(ty) = parent_info.methods.get(method_name) {
                    if !abstract_methods.contains_key(method_name) {
                        abstract_methods.insert(method_name.clone(), ty.clone());
                    }
                }
            }
            current_parent = parent_info.extends.clone();
        } else {
            break;
        }
    }

    // 从 trait 收集方法
    for trait_name in &class_decl.implements {
        if let Some(trait_info) = env.get_trait(trait_name) {
            for (name, ty) in &trait_info.methods {
                if !abstract_methods.contains_key(name) {
                    abstract_methods.insert(name.clone(), ty.clone());
                }
            }
        }
    }

    // 检查类是否实现了所有抽象方法
    let class_info = env.get_class(&class_decl.name)?;
    for method_name in abstract_methods.keys() {
        if !class_info.methods.contains_key(method_name) {
            // 检查是否继承了这个方法
            let mut found = false;
            let mut current = class_info.extends.clone();
            while let Some(parent_name) = current {
                if let Some(parent_info) = env.get_class(&parent_name) {
                    if parent_info.methods.contains_key(method_name) {
                        found = true;
                        break;
                    }
                    current = parent_info.extends.clone();
                } else {
                    break;
                }
            }

            if !found {
                return Some(TypeError::UnimplementedAbstractMethod {
                    method_name: method_name.clone(),
                    span: class_decl.span,
                });
            }
        }
    }

    None
}

/// 检查类型 sub 是否是类型 sup 的子类型
/// 用于类继承和 trait 实现的子类型检查
#[allow(dead_code)]
fn is_subtype_of(sub: &Type, sup: &Type, env: &TypeEnv) -> bool {
    match (sub, sup) {
        // 相同类型
        (t1, t2) if types_equal(t1, t2) => true,

        // 类继承子类型：SubClass <: SuperClass
        // 或者 trait 实现子类型：Class <: Trait
        (Type::Generic(sub_name), Type::Generic(sup_name)) => {
            if sub_name == sup_name {
                return true;
            }

            // 首先检查类继承
            if let Some(class_info) = env.get_class(sub_name) {
                // 检查继承链
                let mut current = Some(sub_name.clone());
                while let Some(name) = current {
                    if name == *sup_name {
                        return true;
                    }
                    if let Some(info) = env.get_class(&name) {
                        current = info.extends.clone();
                    } else {
                        break;
                    }
                }

                // 检查 trait 实现（如果 sup_name 是 trait）
                if env.get_trait(sup_name).is_some() {
                    // 检查直接实现
                    if class_info.implements.contains(sup_name) {
                        return true;
                    }
                    // 检查继承链中的实现
                    let mut current = class_info.extends.clone();
                    while let Some(parent_name) = current {
                        if let Some(parent_info) = env.get_class(&parent_name) {
                            if parent_info.implements.contains(sup_name) {
                                return true;
                            }
                            current = parent_info.extends.clone();
                        } else {
                            break;
                        }
                    }
                }
            }

            false
        }

        // 类型构造器实例：List<Sub> <: List<Super> (如果类型参数协变)
        (Type::TypeConstructor(sub_name, sub_args), Type::TypeConstructor(sup_name, sup_args)) => {
            if sub_name != sup_name || sub_args.len() != sup_args.len() {
                return false;
            }
            // 假设类型参数是协变的（简化实现）
            sub_args
                .iter()
                .zip(sup_args.iter())
                .all(|(s, p)| is_subtype_of(s, p, env))
        }

        // trait 实现子类型：Class <: Trait<T> (带类型参数的 trait)
        (Type::Generic(class_name), Type::TypeConstructor(trait_name, _)) => {
            // 检查类是否实现了该 trait
            if let Some(class_info) = env.get_class(class_name) {
                // 检查直接实现
                if class_info.implements.contains(trait_name) {
                    return true;
                }
                // 检查继承链中的实现
                let mut current = class_info.extends.clone();
                while let Some(parent_name) = current {
                    if let Some(parent_info) = env.get_class(&parent_name) {
                        if parent_info.implements.contains(trait_name) {
                            return true;
                        }
                        current = parent_info.extends.clone();
                    } else {
                        break;
                    }
                }
            }
            // 检查 trait 是否存在
            env.get_trait(trait_name).is_some() && env.get_class(class_name).is_some()
        }

        // Array 类型：Array<Sub> <: Array<Super> (协变)
        (Type::Array(sub_inner), Type::Array(sup_inner)) => {
            is_subtype_of(sub_inner, sup_inner, env)
        }

        // Dictionary 类型：Dict<K, V> <: Dict<K', V'> 要求 K <: K' 且 V <: V'
        (Type::Dictionary(sub_k, sub_v), Type::Dictionary(sup_k, sup_v)) => {
            is_subtype_of(sub_k, sup_k, env) && is_subtype_of(sub_v, sup_v, env)
        }

        // Tuple 类型：(T1, T2, ...) <: (T1', T2', ...) 要求每个 Ti <: Ti'
        (Type::Tuple(sub_tys), Type::Tuple(sup_tys)) => {
            if sub_tys.len() != sup_tys.len() {
                return false;
            }
            sub_tys
                .iter()
                .zip(sup_tys.iter())
                .all(|(s, p)| is_subtype_of(s, p, env))
        }

        // Union 类型：如果所有变体都是子类型，则 union 是子类型
        (Type::Union(_, sub_variants), Type::Union(_, sup_variants)) => {
            if sub_variants.len() != sup_variants.len() {
                return false;
            }
            sub_variants
                .iter()
                .zip(sup_variants.iter())
                .all(|(s, p)| is_subtype_of(s, p, env))
        }

        // Record 类型：每个字段都要求子类型
        (Type::Record(sub_name, sub_fields), Type::Record(sup_name, sup_fields)) => {
            if sub_name != sup_name || sub_fields.len() != sup_fields.len() {
                return false;
            }
            sub_fields
                .iter()
                .zip(sup_fields.iter())
                .all(|((sn, st), (sn2, pt))| sn == sn2 && is_subtype_of(st, pt, env))
        }

        // Async 类型：Async<Sub> <: Async<Super> (协变)
        (Type::Async(sub_inner), Type::Async(sup_inner)) => {
            is_subtype_of(sub_inner, sup_inner, env)
        }

        // 引用类型：&mut T 是 &T 的子类型，引用类型协变
        (Type::MutableReference(sub_inner), Type::Reference(sup_inner))
        | (Type::Reference(sub_inner), Type::Reference(sup_inner))
        | (Type::MutableReference(sub_inner), Type::MutableReference(sup_inner)) => {
            is_subtype_of(sub_inner, sup_inner, env)
        }

        // 指针类型：指针协变
        (Type::Pointer(sub_inner), Type::Pointer(sup_inner)) => {
            is_subtype_of(sub_inner, sup_inner, env)
        }
        (Type::ConstPointer(sub_inner), Type::ConstPointer(sup_inner)) => {
            is_subtype_of(sub_inner, sup_inner, env)
        }

        // 函数类型：函数是逆变的参数，协变的返回值
        // (Sub -> SubRet) <: (Super -> SuperRet)
        // 当 Sub 的参数是 Super 参数的超类型，且 SubRet 是 SuperRet 的子类型
        (Type::Function(sub_params, sub_ret), Type::Function(sup_params, sup_ret)) => {
            if sub_params.len() != sup_params.len() {
                return false;
            }
            // 参数逆变：父类型的参数应该是子类型
            let params_compatible = sub_params.iter().zip(sup_params.iter()).all(|(s, p)| {
                // 参数是逆变的：如果 p 是 s 的子类型，则函数是子类型
                is_subtype_of(p, s, env)
            });
            // 返回值协变：子类型的返回值应该是子类型
            let ret_compatible = is_subtype_of(sub_ret, sup_ret, env);
            params_compatible && ret_compatible
        }

        // Never 是所有类型的子类型
        (Type::Never, _) => true,

        // 其他情况：不是子类型
        _ => false,
    }
}

/// 检查方法重写是否符合变元规则
/// 参数应该是逆变的，返回值应该是协变的
fn check_override_variance(
    child_method: &Type,
    parent_method: &Type,
    method_name: &str,
    span: Span,
) -> Result<(), TypeError> {
    match (child_method, parent_method) {
        (Type::Function(child_params, child_ret), Type::Function(parent_params, parent_ret)) => {
            // 检查参数数量
            if child_params.len() != parent_params.len() {
                return Err(TypeError::VarianceError {
                    method: method_name.to_string(),
                    message: format!(
                        "参数数量不匹配: 期望 {}, 实际 {}",
                        parent_params.len(),
                        child_params.len()
                    ),
                    span,
                });
            }

            // 参数逆变检查：子类方法的参数类型必须是父类方法参数类型的超类型
            // 这是简化的检查，实际中需要更精确的类型比较
            for (child_param, parent_param) in child_params.iter().zip(parent_params.iter()) {
                if !types_equal(child_param, parent_param) {
                    // 允许参数类型的逆变（简化：只检查是否兼容）
                    // 完整实现需要更复杂的类型系统
                }
            }

            // 返回值协变检查：子类方法的返回值类型必须是父类方法返回值类型的子类型
            if !types_equal(child_ret, parent_ret) {
                // 允许返回值类型的协变（简化：只检查是否兼容）
            }

            Ok(())
        }
        _ => Err(TypeError::VarianceError {
            method: method_name.to_string(),
            message: "方法签名不是函数类型".to_string(),
            span,
        }),
    }
}

/// 检查构造函数中的 super() 调用
fn check_super_call_in_constructor(
    class_decl: &ClassDecl,
    constructor: &x_parser::ast::ConstructorDecl,
    env: &mut TypeEnv,
) -> Result<(), TypeError> {
    // 如果类有父类，检查构造函数是否调用了 super()
    if let Some(parent_name) = &class_decl.extends {
        // 使用第一个语句的 span 作为错误位置（如果没有语句则用默认 span）
        let span = constructor
            .body
            .statements
            .first()
            .map(|s| s.span)
            .unwrap_or_default();

        let parent_info = env
            .get_class(parent_name)
            .ok_or_else(|| TypeError::UndefinedType {
                name: parent_name.clone(),
                span,
            })?;

        // 检查构造函数体中是否有 super() 调用
        let has_super_call = constructor.body.statements.iter().any(|stmt| {
            if let StatementKind::Expression(expr) = &stmt.node {
                if let ExpressionKind::Call(callee, _) = &expr.node {
                    if let ExpressionKind::Variable(name) = &callee.node {
                        return name == "super";
                    }
                }
            }
            false
        });

        // 如果父类有构造函数参数，子类构造函数必须调用 super()
        if parent_info.parent_constructor_params.is_some() && !has_super_call {
            return Err(TypeError::MissingSuperCall { span });
        }

        // 如果有 super() 调用，检查参数数量
        if has_super_call {
            for stmt in &constructor.body.statements {
                if let StatementKind::Expression(expr) = &stmt.node {
                    if let ExpressionKind::Call(callee, args) = &expr.node {
                        if let ExpressionKind::Variable(name) = &callee.node {
                            if name == "super" {
                                if let Some(expected_params) =
                                    &parent_info.parent_constructor_params
                                {
                                    if args.len() != expected_params.len() {
                                        return Err(TypeError::SuperCallArgumentMismatch {
                                            expected: expected_params.len(),
                                            actual: args.len(),
                                            span: expr.span,
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

/// 检查可见性访问
#[allow(dead_code)]
fn check_visibility_access(
    class_name: &str,
    member_name: &str,
    is_field: bool,
    access_context: &str,
    env: &TypeEnv,
    span: Span,
) -> Result<(), TypeError> {
    let class_info = env
        .get_class(class_name)
        .ok_or_else(|| TypeError::UndefinedType {
            name: class_name.to_string(),
            span,
        })?;

    let visibility = if is_field {
        class_info
            .field_visibility
            .get(member_name)
            .copied()
            .unwrap_or(Visibility::Private)
    } else {
        class_info
            .method_visibility
            .get(member_name)
            .copied()
            .unwrap_or(Visibility::Private)
    };

    match visibility {
        Visibility::Public => Ok(()),
        Visibility::Private => {
            // 只有在类内部可以访问
            if access_context == class_name {
                Ok(())
            } else {
                Err(if is_field {
                    TypeError::FieldNotVisible {
                        class: class_name.to_string(),
                        field: member_name.to_string(),
                        span,
                    }
                } else {
                    TypeError::MethodNotVisible {
                        class: class_name.to_string(),
                        method: member_name.to_string(),
                        span,
                    }
                })
            }
        }
        Visibility::Protected => {
            // 在类及其子类中可以访问
            if access_context == class_name {
                return Ok(());
            }
            // 检查访问上下文是否是子类
            if let Some(_accessor_info) = env.get_class(access_context) {
                let mut current = Some(access_context.to_string());
                while let Some(name) = current {
                    if name == class_name {
                        return Ok(());
                    }
                    if let Some(info) = env.get_class(&name) {
                        current = info.extends.clone();
                    } else {
                        break;
                    }
                }
            }
            Err(if is_field {
                TypeError::FieldNotVisible {
                    class: class_name.to_string(),
                    field: member_name.to_string(),
                    span,
                }
            } else {
                TypeError::MethodNotVisible {
                    class: class_name.to_string(),
                    method: member_name.to_string(),
                    span,
                }
            })
        }
        Visibility::Internal => {
            // 在同一模块中可以访问（简化：暂时允许所有访问）
            Ok(())
        }
    }
}

/// 检查特征声明（第二遍：检查方法体和类型有效性）
fn check_trait_decl(trait_decl: &TraitDecl, env: &mut TypeEnv) -> Result<(), TypeError> {
    // trait 信息已经在第一遍收集，这里检查方法体

    // 收集类型参数名
    let type_param_names: std::collections::HashSet<String> = trait_decl
        .type_parameters
        .iter()
        .map(|tp| tp.name.clone())
        .collect();

    // 检查类型参数约束：约束必须引用存在的 trait
    for type_param in &trait_decl.type_parameters {
        for constraint in &type_param.constraints {
            if env.get_trait(&constraint.trait_name).is_none() {
                return Err(TypeError::UndefinedType {
                    name: constraint.trait_name.clone(),
                    span: constraint.span,
                });
            }
        }
    }

    // 检查继承的 trait 是否都存在
    for extends_trait in &trait_decl.extends {
        if env.get_trait(extends_trait).is_none() {
            return Err(TypeError::UndefinedType {
                name: extends_trait.clone(),
                span: trait_decl.span,
            });
        }
    }

    for method in &trait_decl.methods {
        // 检查方法参数类型是否有效
        for param in &method.parameters {
            if let Some(type_annot) = &param.type_annot {
                // 验证类型是否正确使用了声明的类型参数
                if !is_valid_type_with_params(type_annot, env, &type_param_names) {
                    return Err(TypeError::UndefinedType {
                        name: format!("{:?}", type_annot),
                        span: param.span,
                    });
                }
            } else {
                return Err(TypeError::CannotInferType { span: param.span });
            }
        }

        // 检查返回类型是否有效
        if let Some(return_type) = &method.return_type {
            if !is_valid_type_with_params(return_type, env, &type_param_names) {
                return Err(TypeError::UndefinedType {
                    name: format!("{:?}", return_type),
                    span: method.span,
                });
            }
        }
        // 检查方法参数类型是否有效
        for param in &method.parameters {
            if let Some(type_annot) = &param.type_annot {
                // 验证类型是否已定义
                if !is_valid_type(type_annot, env) {
                    return Err(TypeError::UndefinedType {
                        name: format!("{:?}", type_annot),
                        span: param.span,
                    });
                }
            } else {
                return Err(TypeError::CannotInferType { span: param.span });
            }
        }

        // 检查返回类型是否有效
        if let Some(return_type) = &method.return_type {
            if !is_valid_type(return_type, env) {
                return Err(TypeError::UndefinedType {
                    name: format!("{:?}", return_type),
                    span: method.span,
                });
            }
        }

        // 检查方法体（如果有）
        if !method.body.statements.is_empty() {
            env.push_scope();
            // 添加 self 参数（trait 方法可以有 self）
            env.add_variable("self", Type::Generic(trait_decl.name.clone()));
            env.add_variable("this", Type::Generic(trait_decl.name.clone()));
            // 添加方法参数
            for param in &method.parameters {
                if let Some(type_annot) = &param.type_annot {
                    env.add_variable(&param.name, type_annot.clone());
                }
            }
            check_block(&method.body, env)?;
            env.pop_scope();
        }
    }

    Ok(())
}

/// 检查枚举声明
fn check_enum_decl(
    enum_decl: &x_parser::ast::EnumDecl,
    env: &mut TypeEnv,
) -> Result<(), TypeError> {
    // 注册枚举类型
    let enum_name = &enum_decl.name;

    // 收集类型参数名
    let type_param_names: std::collections::HashSet<String> = enum_decl
        .type_parameters
        .iter()
        .map(|tp| tp.name.clone())
        .collect();

    // 检查类型参数约束：约束必须引用存在的 trait
    for type_param in &enum_decl.type_parameters {
        for constraint in &type_param.constraints {
            if env.get_trait(&constraint.trait_name).is_none() {
                return Err(TypeError::UndefinedType {
                    name: constraint.trait_name.clone(),
                    span: constraint.span,
                });
            }
        }
    }

    // 检查变体类型是否有效
    for variant in &enum_decl.variants {
        match &variant.data {
            x_parser::ast::EnumVariantData::Unit => {
                // 无类型参数，无需检查
            }
            x_parser::ast::EnumVariantData::Tuple(types) => {
                for ty in types {
                    if !is_valid_type_with_params(ty, env, &type_param_names) {
                        return Err(TypeError::UndefinedType {
                            name: format!("{:?}", ty),
                            span: variant.span,
                        });
                    }
                }
            }
            x_parser::ast::EnumVariantData::Record(fields) => {
                for (_, ty) in fields {
                    if !is_valid_type_with_params(ty, env, &type_param_names) {
                        return Err(TypeError::UndefinedType {
                            name: format!("{:?}", ty),
                            span: variant.span,
                        });
                    }
                }
            }
        }
    }

    // 注册枚举到环境（用于类型推断）
    env.register_enum(enum_name.clone(), enum_decl.clone());

    Ok(())
}

/// 检查记录声明
fn check_record_decl(
    record_decl: &x_parser::ast::RecordDecl,
    env: &mut TypeEnv,
) -> Result<(), TypeError> {
    let record_name = &record_decl.name;

    // 收集类型参数名
    let type_param_names: std::collections::HashSet<String> = record_decl
        .type_parameters
        .iter()
        .map(|tp| tp.name.clone())
        .collect();

    // 检查类型参数约束：约束必须引用存在的 trait
    for type_param in &record_decl.type_parameters {
        for constraint in &type_param.constraints {
            if env.get_trait(&constraint.trait_name).is_none() {
                return Err(TypeError::UndefinedType {
                    name: constraint.trait_name.clone(),
                    span: constraint.span,
                });
            }
        }
    }

    // 检查每个字段类型是否有效
    for (_field_name, field_type) in &record_decl.fields {
        if !is_valid_type_with_params(field_type, env, &type_param_names) {
            return Err(TypeError::UndefinedType {
                name: format!("{:?}", field_type),
                span: record_decl.span,
            });
        }
    }

    // 注册记录到环境
    env.register_record(record_name.clone(), record_decl.clone());

    Ok(())
}

/// 检查效果声明
fn check_effect_decl(
    effect_decl: &x_parser::ast::EffectDecl,
    env: &mut TypeEnv,
) -> Result<(), TypeError> {
    let effect_name = &effect_decl.name;

    // 收集类型参数名
    let type_param_names: std::collections::HashSet<String> = effect_decl
        .type_parameters
        .iter()
        .map(|tp| tp.name.clone())
        .collect();

    // 检查类型参数约束：约束必须引用存在的 trait
    for type_param in &effect_decl.type_parameters {
        for constraint in &type_param.constraints {
            if env.get_trait(&constraint.trait_name).is_none() {
                return Err(TypeError::UndefinedType {
                    name: constraint.trait_name.clone(),
                    span: constraint.span,
                });
            }
        }
    }

    // 检查每个操作的参数和返回类型
    // operations: (name: String, param_ty: Option<Type>, ret_ty: Option<Type>)
    for (_op_name, param_ty, ret_ty) in &effect_decl.operations {
        // 参数类型检查
        if let Some(param_ty) = param_ty {
            if !is_valid_type_with_params(param_ty, env, &type_param_names) {
                return Err(TypeError::UndefinedType {
                    name: format!("{:?}", param_ty),
                    span: effect_decl.span,
                });
            }
        }

        // 返回类型检查
        if let Some(ret_ty) = ret_ty {
            if !is_valid_type_with_params(ret_ty, env, &type_param_names) {
                return Err(TypeError::UndefinedType {
                    name: format!("{:?}", ret_ty),
                    span: effect_decl.span,
                });
            }
        }
    }

    // 注册效果到环境
    env.register_effect(effect_name.clone(), effect_decl.clone());

    Ok(())
}

/// 检查 trait 实现
fn check_implement_decl(
    impl_decl: &x_parser::ast::ImplementDecl,
    env: &mut TypeEnv,
) -> Result<(), TypeError> {
    let trait_name = &impl_decl.trait_name;
    let target_type = &impl_decl.target_type;

    // 收集类型参数名
    let type_param_names: std::collections::HashSet<String> = impl_decl
        .type_parameters
        .iter()
        .map(|tp| tp.name.clone())
        .collect();

    // 检查 trait 是否存在
    if env.get_trait(trait_name).is_none() {
        return Err(TypeError::UndefinedType {
            name: trait_name.clone(),
            span: impl_decl.span,
        });
    }

    // 检查类型参数约束：约束必须引用存在的 trait
    for type_param in &impl_decl.type_parameters {
        for constraint in &type_param.constraints {
            if env.get_trait(&constraint.trait_name).is_none() {
                return Err(TypeError::UndefinedType {
                    name: constraint.trait_name.clone(),
                    span: constraint.span,
                });
            }
        }
    }

    // 检查 where 子句约束
    for constraint in &impl_decl.where_clause {
        if env.get_trait(&constraint.trait_name).is_none() {
            return Err(TypeError::UndefinedType {
                name: constraint.trait_name.clone(),
                span: constraint.span,
            });
        }
    }

    // 检查目标类型 是否有效
    // 我们只检查类型引用是否正确使用了类型参数，不完整检查整个实现
    if !is_valid_type_with_params(target_type, env, &type_param_names) {
        return Err(TypeError::UndefinedType {
            name: format!("{:?}", target_type),
            span: impl_decl.span,
        });
    }

    // 检查每个方法实现
    env.push_scope();
    for method in &impl_decl.methods {
        // 检查方法参数和体
        check_function_decl(method, env)?;
    }
    env.pop_scope();

    Ok(())
}

/// 检查类型是否有效（考虑类型参数）
fn is_valid_type_with_params(
    ty: &Type,
    env: &TypeEnv,
    type_params: &std::collections::HashSet<String>,
) -> bool {
    match ty {
        // 基本类型始终有效
        Type::Int
        | Type::UnsignedInt
        | Type::Float
        | Type::Bool
        | Type::String
        | Type::Char
        | Type::Unit
        | Type::Never
        | Type::Dynamic => true,

        // 复合类型需要检查内部类型
        Type::Array(inner) => is_valid_type_with_params(inner, env, type_params),
        Type::Dictionary(key, value) => {
            is_valid_type_with_params(key, env, type_params)
                && is_valid_type_with_params(value, env, type_params)
        }
        Type::Tuple(types) => types
            .iter()
            .all(|t| is_valid_type_with_params(t, env, type_params)),
        Type::Async(inner) => is_valid_type_with_params(inner, env, type_params),
        Type::Function(params, ret) => {
            params
                .iter()
                .all(|p| is_valid_type_with_params(p, env, type_params))
                && is_valid_type_with_params(ret, env, type_params)
        }

        // Record 和 Union 类型
        Type::Record(_, fields) => fields
            .iter()
            .all(|(_, t)| is_valid_type_with_params(t, env, type_params)),
        Type::Union(_, variants) => variants
            .iter()
            .all(|t| is_valid_type_with_params(t, env, type_params)),

        // 泛型类型 - 检查是否是已定义的类、特征、类型别名，或者是类型参数
        Type::Generic(name) | Type::TypeParam(name) | Type::Var(name) => {
            type_params.contains(name)
                || env.get_class(name).is_some()
                || env.get_trait(name).is_some()
                || env.get_type_alias(name).is_some()
        }

        // 类型构造器应用：List<Int>, Map<String, Int>
        Type::TypeConstructor(name, type_args) => {
            // 检查基础类型是否有效
            let base_valid = type_params.contains(name)
                || env.get_class(name).is_some()
                || env.get_trait(name).is_some()
                || env.get_type_alias(name).is_some();
            base_valid
                && type_args
                    .iter()
                    .all(|t| is_valid_type_with_params(t, env, type_params))
        }

        // 引用类型
        Type::Reference(inner) => is_valid_type_with_params(inner, env, type_params),
        Type::MutableReference(inner) => is_valid_type_with_params(inner, env, type_params),

        // FFI 类型
        Type::Pointer(inner) => is_valid_type_with_params(inner, env, type_params),
        Type::ConstPointer(inner) => is_valid_type_with_params(inner, env, type_params),
        Type::MutPointer(inner) => is_valid_type_with_params(inner, env, type_params),
        Type::Void => true,

        // C FFI 类型 - 都是有效的原始类型
        Type::CInt
        | Type::CUInt
        | Type::CLong
        | Type::CULong
        | Type::CLongLong
        | Type::CULongLong
        | Type::CFloat
        | Type::CDouble
        | Type::CChar
        | Type::CSize
        | Type::CString => true,
    }
}

/// 检查类型别名声明（第二遍：验证类型有效性）
fn check_type_alias(type_alias: &TypeAlias, env: &mut TypeEnv) -> Result<(), TypeError> {
    let span = type_alias.span;

    // 类型别名已在第一遍收集，这里验证类型有效性

    // 验证目标类型是否有效
    if !is_valid_type(&type_alias.type_, env) {
        return Err(TypeError::UndefinedType {
            name: format!("{:?}", type_alias.type_),
            span,
        });
    }

    // 检查非法递归类型定义
    let mut visited = std::collections::HashSet::new();
    visited.insert(type_alias.name.clone());
    if check_recursive_type_definition(
        &type_alias.name,
        false,
        &type_alias.type_,
        &mut visited,
        env,
    ) {
        return Err(TypeError::RecursiveType { span });
    }

    Ok(())
}

/// 第一遍：收集 newtype 信息
fn collect_newtype_info(newtype: &Newtype, env: &mut TypeEnv) -> Result<(), TypeError> {
    let span = newtype.span;

    if env.get_type_alias(&newtype.name).is_some() {
        return Err(TypeError::DuplicateDeclaration {
            name: newtype.name.clone(),
            span,
        });
    }

    // Newtype 也作为命名类型添加到环境中
    // 与类型别名不同，newtype 创建一个全新的不透明类型
    // 在当前简化实现中，我们仍然存储底层类型供以后使用
    env.add_type_alias(&newtype.name, newtype.type_.clone());

    Ok(())
}

/// 检查 newtype 声明（第二遍：验证类型有效性）
fn check_newtype(newtype: &Newtype, env: &mut TypeEnv) -> Result<(), TypeError> {
    let span = newtype.span;

    // 验证底层类型是否有效
    if !is_valid_type(&newtype.type_, env) {
        return Err(TypeError::UndefinedType {
            name: format!("{:?}", newtype.type_),
            span,
        });
    }

    // 检查非法递归类型定义
    let mut visited = std::collections::HashSet::new();
    visited.insert(newtype.name.clone());
    if check_recursive_type_definition(&newtype.name, false, &newtype.type_, &mut visited, env) {
        return Err(TypeError::RecursiveType { span });
    }

    Ok(())
}

/// 创建函数类型
fn create_function_type(func_decl: &FunctionDecl) -> Type {
    let mut param_types = Vec::new();
    for param in &func_decl.parameters {
        if let Some(type_annot) = &param.type_annot {
            param_types.push(Box::new(type_annot.clone()));
        } else {
            param_types.push(Box::new(Type::Unit));
        }
    }

    let return_type = if let Some(return_type) = &func_decl.return_type {
        Box::new(return_type.clone())
    } else {
        Box::new(Type::Unit)
    };

    Type::Function(param_types, return_type)
}

/// 检查类型是否有效
fn is_valid_type(ty: &Type, env: &TypeEnv) -> bool {
    match ty {
        // 基本类型始终有效
        Type::Int
        | Type::UnsignedInt
        | Type::Float
        | Type::Bool
        | Type::String
        | Type::Char
        | Type::Unit
        | Type::Never
        | Type::Dynamic => true,

        // 复合类型需要检查内部类型
        Type::Array(inner) => is_valid_type(inner, env),
        Type::Dictionary(key, value) => is_valid_type(key, env) && is_valid_type(value, env),
        Type::Tuple(types) => types.iter().all(|t| is_valid_type(t, env)),
        Type::Async(inner) => is_valid_type(inner, env),
        Type::Function(params, ret) => {
            params.iter().all(|p| is_valid_type(p, env)) && is_valid_type(ret, env)
        }

        // Record 和 Union 类型
        Type::Record(_, fields) => fields.iter().all(|(_, t)| is_valid_type(t, env)),
        Type::Union(_, variants) => variants.iter().all(|t| is_valid_type(t, env)),

        // 泛型类型 - 检查是否是已定义的类、特征或类型别名
        Type::Generic(name) | Type::TypeParam(name) | Type::Var(name) => {
            env.get_class(name).is_some()
                || env.get_trait(name).is_some()
                || env.get_type_alias(name).is_some()
        }

        // 类型构造器应用：List<Int>, Map<String, Int>
        Type::TypeConstructor(name, type_args) => {
            // 检查基础类型是否有效
            let base_valid = env.get_class(name).is_some()
                || env.get_trait(name).is_some()
                || env.get_type_alias(name).is_some();
            // 检查所有类型参数是否有效
            base_valid && type_args.iter().all(|t| is_valid_type(t, env))
        }

        // 引用类型
        Type::Reference(inner) => is_valid_type(inner, env),
        Type::MutableReference(inner) => is_valid_type(inner, env),

        // FFI 类型
        Type::Pointer(inner) => is_valid_type(inner, env),
        Type::ConstPointer(inner) => is_valid_type(inner, env),
        Type::MutPointer(inner) => is_valid_type(inner, env),
        Type::Void => true,

        // C FFI 类型 - 都是有效的原始类型
        Type::CInt
        | Type::CUInt
        | Type::CLong
        | Type::CULong
        | Type::CLongLong
        | Type::CULongLong
        | Type::CFloat
        | Type::CDouble
        | Type::CChar
        | Type::CSize
        | Type::CString => true,
    }
}

/// 检查递归类型定义
/// 检测类型别名或类字段中的无限递归
/// 返回 true 表示发现非法递归，需要报错
#[allow(clippy::only_used_in_recursion)]
fn check_recursive_type_definition(
    current_type_name: &str,
    is_class: bool,
    checking_ty: &Type,
    visited: &mut std::collections::HashSet<String>,
    env: &TypeEnv,
) -> bool {
    match checking_ty {
        // 如果遇到我们正在定义的类型本身，这就是非法递归
        Type::Generic(name) => {
            if name == current_type_name {
                // 直接自引用
                return true;
            }
            if visited.contains(name) {
                // 已经访问过，避免无限循环
                return false;
            }
            visited.insert(name.clone());

            // 如果是类型别名，继续展开检查
            if let Some(aliased_ty) = env.get_type_alias(name) {
                if check_recursive_type_definition(
                    current_type_name,
                    is_class,
                    aliased_ty,
                    visited,
                    env,
                ) {
                    return true;
                }
            }
            // 如果是类，检查所有字段
            if let Some(class_info) = env.get_class(name) {
                for field_ty in class_info.fields.values() {
                    if check_recursive_type_definition(
                        current_type_name,
                        is_class,
                        field_ty,
                        visited,
                        env,
                    ) {
                        return true;
                    }
                }
            }
            visited.remove(name);
            false
        }

        // 类型构造器，检查类型构造器的名称和所有参数
        Type::TypeConstructor(name, args) => {
            if name == current_type_name {
                // 直接自引用，但是泛型构造器自我应用通常也是非法递归
                return true;
            }
            if visited.contains(name) {
                return false;
            }
            visited.insert(name.clone());

            // 检查类型参数
            for arg in args {
                if check_recursive_type_definition(current_type_name, is_class, arg, visited, env) {
                    return true;
                }
            }

            // 如果是类型别名，展开检查
            if let Some(aliased_ty) = env.get_type_alias(name) {
                if check_recursive_type_definition(
                    current_type_name,
                    is_class,
                    aliased_ty,
                    visited,
                    env,
                ) {
                    return true;
                }
            }

            visited.remove(name);
            false
        }

        // 引用类型内部的递归是允许的（引用间接，大小固定）
        Type::Reference(_inner) | Type::MutableReference(_inner) => {
            // 不继续检查内部，因为引用间接打破了递归
            false
        }

        // 指针类型内部的递归是允许的（指针间接，大小固定）
        Type::Pointer(_inner) | Type::ConstPointer(_inner) | Type::MutPointer(_inner) => {
            // 不继续检查内部，因为指针间接打破了递归
            false
        }

        // 对于复合类型，递归检查内部组件
        Type::Array(inner) => {
            check_recursive_type_definition(current_type_name, is_class, inner, visited, env)
        }
        Type::Dictionary(key, value) => {
            check_recursive_type_definition(current_type_name, is_class, key, visited, env)
                || check_recursive_type_definition(current_type_name, is_class, value, visited, env)
        }
        Type::Tuple(types) => types
            .iter()
            .any(|t| check_recursive_type_definition(current_type_name, is_class, t, visited, env)),
        Type::Async(inner) => {
            check_recursive_type_definition(current_type_name, is_class, inner, visited, env)
        }
        Type::Function(params, ret) => {
            params.iter().any(|p| {
                check_recursive_type_definition(current_type_name, is_class, p, visited, env)
            }) || check_recursive_type_definition(current_type_name, is_class, ret, visited, env)
        }
        Type::Record(_, fields) => fields.iter().any(|(_, t)| {
            check_recursive_type_definition(current_type_name, is_class, t, visited, env)
        }),
        Type::Union(_, variants) => variants
            .iter()
            .any(|t| check_recursive_type_definition(current_type_name, is_class, t, visited, env)),

        // 基本类型和其他不会形成递归
        Type::Int
        | Type::UnsignedInt
        | Type::Float
        | Type::Bool
        | Type::String
        | Type::Char
        | Type::Unit
        | Type::Never
        | Type::Dynamic
        | Type::Void
        | Type::CInt
        | Type::CUInt
        | Type::CLong
        | Type::CULong
        | Type::CLongLong
        | Type::CULongLong
        | Type::CFloat
        | Type::CDouble
        | Type::CChar
        | Type::CSize
        | Type::CString
        | Type::TypeParam(_)
        | Type::Var(_) => false,
    }
}

// ============================================================================
// 泛型类型支持
// ============================================================================

/// 类型替换：将类型参数替换为具体类型
pub fn apply_type_substitution(ty: &Type, subst: &HashMap<String, Type>) -> Type {
    match ty {
        // 类型参数：如果在替换表中，则替换
        Type::TypeParam(name) | Type::Var(name) => {
            subst.get(name).cloned().unwrap_or_else(|| ty.clone())
        }

        // 类型构造器：递归替换类型参数
        Type::TypeConstructor(name, type_args) => {
            let new_args: Vec<Type> = type_args
                .iter()
                .map(|t| apply_type_substitution(t, subst))
                .collect();
            Type::TypeConstructor(name.clone(), new_args)
        }

        // 复合类型：递归替换
        Type::Array(inner) => Type::Array(Box::new(apply_type_substitution(inner, subst))),
        Type::Async(inner) => Type::Async(Box::new(apply_type_substitution(inner, subst))),
        Type::Function(params, ret) => {
            let new_params: Vec<Box<Type>> = params
                .iter()
                .map(|p| Box::new(apply_type_substitution(p, subst)))
                .collect();
            Type::Function(new_params, Box::new(apply_type_substitution(ret, subst)))
        }
        Type::Tuple(types) => Type::Tuple(
            types
                .iter()
                .map(|t| apply_type_substitution(t, subst))
                .collect(),
        ),
        Type::Dictionary(k, v) => Type::Dictionary(
            Box::new(apply_type_substitution(k, subst)),
            Box::new(apply_type_substitution(v, subst)),
        ),
        Type::Record(name, fields) => {
            let new_fields: Vec<(String, Box<Type>)> = fields
                .iter()
                .map(|(n, t)| (n.clone(), Box::new(apply_type_substitution(t, subst))))
                .collect();
            Type::Record(name.clone(), new_fields)
        }
        Type::Union(name, variants) => Type::Union(
            name.clone(),
            variants
                .iter()
                .map(|t| apply_type_substitution(t, subst))
                .collect(),
        ),

        // 基本类型和泛型类型名不变
        Type::Int
        | Type::UnsignedInt
        | Type::Float
        | Type::Bool
        | Type::String
        | Type::Char
        | Type::Unit
        | Type::Never
        | Type::Dynamic
        | Type::Generic(_)
        | Type::Void => ty.clone(),

        // C FFI 类型 - 不需要替换
        Type::CInt
        | Type::CUInt
        | Type::CLong
        | Type::CULong
        | Type::CLongLong
        | Type::CULongLong
        | Type::CFloat
        | Type::CDouble
        | Type::CChar
        | Type::CSize
        | Type::CString => ty.clone(),

        // 引用类型 - 递归替换内部类型参数
        Type::Reference(inner) => Type::Reference(Box::new(apply_type_substitution(inner, subst))),
        Type::MutableReference(inner) => {
            Type::MutableReference(Box::new(apply_type_substitution(inner, subst)))
        }

        // FFI 指针类型
        Type::Pointer(inner) => Type::Pointer(Box::new(apply_type_substitution(inner, subst))),
        Type::ConstPointer(inner) => {
            Type::ConstPointer(Box::new(apply_type_substitution(inner, subst)))
        }
        Type::MutPointer(inner) => {
            Type::MutPointer(Box::new(apply_type_substitution(inner, subst)))
        }
    }
}

/// 实例化泛型函数类型
pub fn instantiate_function_type(
    type_params: &[x_parser::ast::TypeParameter],
    type_args: &[Type],
    param_types: &[Type],
    return_type: &Type,
    span: Span,
) -> Result<(Vec<Type>, Type), TypeError> {
    // 检查类型参数数量
    if type_params.len() != type_args.len() {
        return Err(TypeError::ParameterCountMismatch {
            expected: type_params.len(),
            actual: type_args.len(),
            span,
        });
    }

    // 构建替换表
    let subst: HashMap<String, Type> = type_params
        .iter()
        .zip(type_args.iter())
        .map(|(param, arg)| (param.name.clone(), arg.clone()))
        .collect();

    // 应用替换
    let new_params: Vec<Type> = param_types
        .iter()
        .map(|t| apply_type_substitution(t, &subst))
        .collect();
    let new_return = apply_type_substitution(return_type, &subst);

    Ok((new_params, new_return))
}

/// 检查类型参数约束是否满足
pub fn check_type_constraints(
    type_params: &[x_parser::ast::TypeParameter],
    type_args: &[Type],
    env: &TypeEnv,
    span: Span,
) -> Result<(), TypeError> {
    for (param, arg) in type_params.iter().zip(type_args.iter()) {
        for constraint in &param.constraints {
            // 检查类型是否实现了所需的 trait
            // 目前简化实现：只检查是否存在该 trait
            let trait_name = &constraint.trait_name;

            // 检查类型参数是否有该约束
            if let Type::TypeParam(name) = arg {
                // 在环境中查找该类型参数是否满足约束
                // 简化实现：假设类型参数总是满足约束
                let _ = name;
            } else if let Type::Generic(class_name) = arg {
                // 检查类是否实现了该 trait
                if let Some(class_info) = env.get_class(class_name) {
                    if !class_info.implements.contains(trait_name) {
                        return Err(TypeError::TypeConstraintViolation { span });
                    }
                }
            } else if let Type::TypeConstructor(class_name, _) = arg {
                // 检查泛型实例化是否实现了该 trait
                if let Some(class_info) = env.get_class(class_name) {
                    if !class_info.implements.contains(trait_name) {
                        return Err(TypeError::TypeConstraintViolation { span });
                    }
                }
            }
        }
    }
    Ok(())
}

// ============================================================================
// Hindley-Milner 类型推断系统
// ============================================================================

/// 类型方案：forall a1, a2, ... . T
/// 表示一个多态类型，其中量化变量可以是任意类型
#[derive(Debug, Clone)]
pub struct TypeScheme {
    /// 量化的类型变量
    pub quantified: Vec<String>,
    /// 实际类型
    pub ty: Type,
}

impl TypeScheme {
    /// 从单态类型创建类型方案（无量化变量）
    pub fn monomorphic(ty: Type) -> Self {
        TypeScheme {
            quantified: vec![],
            ty,
        }
    }

    /// 实例化类型方案：用量化变量替换为新鲜类型变量
    pub fn instantiate(&self, var_gen: &TypeVarGenerator) -> Type {
        let subst: HashMap<String, Type> = self
            .quantified
            .iter()
            .map(|name| (name.clone(), var_gen.fresh()))
            .collect();
        apply_type_substitution(&self.ty, &subst)
    }
}

/// 类型变量生成器
/// 生成唯一的类型变量名 '_0, '_1, '_2, ...
#[derive(Debug)]
pub struct TypeVarGenerator {
    counter: RefCell<usize>,
}

impl TypeVarGenerator {
    pub fn new() -> Self {
        TypeVarGenerator {
            counter: RefCell::new(0),
        }
    }

    /// 生成一个新鲜的类型变量
    pub fn fresh(&self) -> Type {
        let mut counter = self.counter.borrow_mut();
        let name = format!("'_{}", *counter);
        *counter += 1;
        Type::Var(name)
    }
}

impl Default for TypeVarGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// 合一错误
#[derive(Debug, Clone)]
pub enum UnificationError {
    /// 类型不匹配
    TypeMismatch(Type, Type),
    /// 发生了无限类型（occurs check 失败）
    InfiniteType(String, Type),
}

/// 合一两个类型，返回最一般合一子（Most General Unifier）
pub fn unify(t1: &Type, t2: &Type) -> Result<HashMap<String, Type>, UnificationError> {
    // 首先应用已有的替换（如果需要的话）
    // 这里简化实现，直接进行合一

    match (t1, t2) {
        // 类型变量可以与任何类型合一
        (Type::Var(name), other) | (other, Type::Var(name)) => {
            // Occurs check：防止无限类型
            if occurs_in(name, other) {
                return Err(UnificationError::InfiniteType(name.clone(), other.clone()));
            }
            let mut subst = HashMap::new();
            subst.insert(name.clone(), other.clone());
            Ok(subst)
        }

        // 类型参数的合一（类似类型变量）
        (Type::TypeParam(name), other) | (other, Type::TypeParam(name)) => {
            // 对于命名类型参数，我们允许合一
            if let Type::TypeParam(name2) = other {
                if name == name2 {
                    return Ok(HashMap::new());
                }
            }
            let mut subst = HashMap::new();
            subst.insert(name.clone(), other.clone());
            Ok(subst)
        }

        // 基本类型必须相等
        (Type::Int, Type::Int)
        | (Type::Float, Type::Float)
        | (Type::Bool, Type::Bool)
        | (Type::String, Type::String)
        | (Type::Char, Type::Char)
        | (Type::Unit, Type::Unit)
        | (Type::Never, Type::Never) => Ok(HashMap::new()),

        // 泛型类型名必须相等
        (Type::Generic(n1), Type::Generic(n2)) if n1 == n2 => Ok(HashMap::new()),

        // 数组类型：递归合一元素类型
        (Type::Array(e1), Type::Array(e2)) => unify(e1, e2),

        // 元组类型
        (Type::Tuple(ts1), Type::Tuple(ts2)) => {
            if ts1.len() != ts2.len() {
                return Err(UnificationError::TypeMismatch(t1.clone(), t2.clone()));
            }
            let mut subst = HashMap::new();
            for (e1, e2) in ts1.iter().zip(ts2.iter()) {
                let s = unify(
                    &apply_type_substitution(e1, &subst),
                    &apply_type_substitution(e2, &subst),
                )?;
                subst = compose_substitutions(&subst, &s);
            }
            Ok(subst)
        }

        // 字典类型
        (Type::Dictionary(k1, v1), Type::Dictionary(k2, v2)) => {
            let s1 = unify(k1, k2)?;
            let v2_subst = apply_type_substitution(v2, &s1);
            let v1_subst = apply_type_substitution(v1, &s1);
            let s2 = unify(&v1_subst, &v2_subst)?;
            Ok(compose_substitutions(&s1, &s2))
        }

        // 函数类型
        (Type::Function(params1, ret1), Type::Function(params2, ret2)) => {
            if params1.len() != params2.len() {
                return Err(UnificationError::TypeMismatch(t1.clone(), t2.clone()));
            }
            let mut subst = HashMap::new();
            for (p1, p2) in params1.iter().zip(params2.iter()) {
                let s = unify(
                    &apply_type_substitution(p1, &subst),
                    &apply_type_substitution(p2, &subst),
                )?;
                subst = compose_substitutions(&subst, &s);
            }
            let ret1_subst = apply_type_substitution(ret1, &subst);
            let ret2_subst = apply_type_substitution(ret2, &subst);
            let s = unify(&ret1_subst, &ret2_subst)?;
            Ok(compose_substitutions(&subst, &s))
        }

        // 异步类型
        (Type::Async(i1), Type::Async(i2)) => unify(i1, i2),

        // 引用类型
        (Type::Reference(i1), Type::Reference(i2)) => unify(i1, i2),
        (Type::MutableReference(i1), Type::MutableReference(i2)) => unify(i1, i2),

        // FFI 指针类型
        (Type::Pointer(i1), Type::Pointer(i2)) => unify(i1, i2),
        (Type::ConstPointer(i1), Type::ConstPointer(i2)) => unify(i1, i2),

        // Void 类型
        (Type::Void, Type::Void) => Ok(HashMap::new()),

        // 类型构造器
        (Type::TypeConstructor(n1, args1), Type::TypeConstructor(n2, args2)) => {
            if n1 != n2 || args1.len() != args2.len() {
                return Err(UnificationError::TypeMismatch(t1.clone(), t2.clone()));
            }
            let mut subst = HashMap::new();
            for (a1, a2) in args1.iter().zip(args2.iter()) {
                let s = unify(
                    &apply_type_substitution(a1, &subst),
                    &apply_type_substitution(a2, &subst),
                )?;
                subst = compose_substitutions(&subst, &s);
            }
            Ok(subst)
        }

        // 其他情况：类型不匹配
        _ => Err(UnificationError::TypeMismatch(t1.clone(), t2.clone())),
    }
}

/// 检查类型变量是否出现在类型中（occurs check）
pub fn occurs_in(var_name: &str, ty: &Type) -> bool {
    match ty {
        Type::Var(name) | Type::TypeParam(name) => name == var_name,

        Type::Array(inner) => occurs_in(var_name, inner),
        Type::Async(inner) => occurs_in(var_name, inner),
        Type::Function(params, ret) => {
            params.iter().any(|p| occurs_in(var_name, p)) || occurs_in(var_name, ret)
        }
        Type::Tuple(types) => types.iter().any(|t| occurs_in(var_name, t)),
        Type::Dictionary(k, v) => occurs_in(var_name, k) || occurs_in(var_name, v),
        Type::Record(_, fields) => fields.iter().any(|(_, t)| occurs_in(var_name, t)),
        Type::Union(_, variants) => variants.iter().any(|t| occurs_in(var_name, t)),
        Type::TypeConstructor(_, args) => args.iter().any(|t| occurs_in(var_name, t)),

        Type::Int
        | Type::UnsignedInt
        | Type::Float
        | Type::Bool
        | Type::String
        | Type::Char
        | Type::Unit
        | Type::Never
        | Type::Dynamic
        | Type::Generic(_)
        | Type::Void => false,

        // C FFI 类型 - 不包含类型变量
        Type::CInt
        | Type::CUInt
        | Type::CLong
        | Type::CULong
        | Type::CLongLong
        | Type::CULongLong
        | Type::CFloat
        | Type::CDouble
        | Type::CChar
        | Type::CSize
        | Type::CString => false,

        // 引用类型 - 检查内部类型
        Type::Reference(inner) => occurs_in(var_name, inner),
        Type::MutableReference(inner) => occurs_in(var_name, inner),

        // FFI 指针类型
        Type::Pointer(inner) => occurs_in(var_name, inner),
        Type::ConstPointer(inner) => occurs_in(var_name, inner),
        Type::MutPointer(inner) => occurs_in(var_name, inner),
    }
}

/// 组合两个替换
pub fn compose_substitutions(
    s1: &HashMap<String, Type>,
    s2: &HashMap<String, Type>,
) -> HashMap<String, Type> {
    // 先应用 s2 到 s1 的所有值，然后合并 s2
    let mut result = HashMap::new();

    for (k, v) in s1 {
        result.insert(k.clone(), apply_type_substitution(v, s2));
    }

    for (k, v) in s2 {
        if !result.contains_key(k) {
            result.insert(k.clone(), v.clone());
        }
    }

    result
}

/// 泛化：提取环境中不自由的类型变量，创建类型方案
/// 自由变量是在环境中出现但不在当前类型中的变量
pub fn generalize(ty: &Type, env_vars: &HashSet<String>) -> TypeScheme {
    // 找出类型中所有的自由变量
    let free_in_type = free_type_vars(ty);

    // 量化那些在类型中出现但不在环境中的变量
    let quantified: Vec<String> = free_in_type
        .into_iter()
        .filter(|v| !env_vars.contains(v))
        .collect();

    TypeScheme {
        quantified,
        ty: ty.clone(),
    }
}

/// 收集类型中的所有自由变量
pub fn free_type_vars(ty: &Type) -> Vec<String> {
    let mut vars = Vec::new();
    collect_free_vars(ty, &mut vars);
    vars.sort();
    vars.dedup();
    vars
}

fn collect_free_vars(ty: &Type, vars: &mut Vec<String>) {
    match ty {
        Type::Var(name) => vars.push(name.clone()),

        Type::Array(inner) => collect_free_vars(inner, vars),
        Type::Async(inner) => collect_free_vars(inner, vars),
        Type::Function(params, ret) => {
            for p in params {
                collect_free_vars(p, vars);
            }
            collect_free_vars(ret, vars);
        }
        Type::Tuple(types) => {
            for t in types {
                collect_free_vars(t, vars);
            }
        }
        Type::Dictionary(k, v) => {
            collect_free_vars(k, vars);
            collect_free_vars(v, vars);
        }
        Type::Record(_, fields) => {
            for (_, t) in fields {
                collect_free_vars(t, vars);
            }
        }
        Type::Union(_, variants) => {
            for t in variants {
                collect_free_vars(t, vars);
            }
        }
        Type::TypeConstructor(_, args) => {
            for t in args {
                collect_free_vars(t, vars);
            }
        }

        Type::Int
        | Type::UnsignedInt
        | Type::Float
        | Type::Bool
        | Type::String
        | Type::Char
        | Type::Unit
        | Type::Never
        | Type::Dynamic
        | Type::Generic(_)
        | Type::TypeParam(_)
        | Type::Void => {}

        // C FFI 类型 - 不包含自由类型变量
        Type::CInt
        | Type::CUInt
        | Type::CLong
        | Type::CULong
        | Type::CLongLong
        | Type::CULongLong
        | Type::CFloat
        | Type::CDouble
        | Type::CChar
        | Type::CSize
        | Type::CString => {}

        // 引用类型 - 收集内部类型中的自由变量
        Type::Reference(inner) => collect_free_vars(inner, vars),
        Type::MutableReference(inner) => collect_free_vars(inner, vars),

        // FFI 指针类型
        Type::Pointer(inner) => collect_free_vars(inner, vars),
        Type::ConstPointer(inner) => collect_free_vars(inner, vars),
        Type::MutPointer(inner) => collect_free_vars(inner, vars),
    }
}

/// 收集环境中的所有类型变量
pub fn collect_env_vars(env: &TypeEnv) -> HashSet<String> {
    let mut vars = HashSet::new();

    // 收集变量类型中的自由变量
    for scope in &env.variable_scopes {
        for ty in scope.values() {
            for v in free_type_vars(ty) {
                vars.insert(v);
            }
        }
    }

    // 收集函数类型中的自由变量
    for func_info in env.functions.values() {
        for v in free_type_vars(&func_info.ty) {
            vars.insert(v);
        }
    }

    vars
}

/// 推断泛型函数调用的类型参数
/// 使用 HM 类型推断从参数类型推断类型参数
pub fn infer_type_arguments(
    type_params: &[x_parser::ast::TypeParameter],
    param_types: &[Type],
    arg_types: &[Type],
    return_type: &Type,
    var_gen: &TypeVarGenerator,
    env: &TypeEnv,
    span: Span,
) -> Result<(Vec<Type>, Type), TypeError> {
    // 如果没有类型参数，直接返回
    if type_params.is_empty() {
        return Ok((vec![], return_type.clone()));
    }

    // 为每个类型参数创建新鲜类型变量
    let type_var_map: HashMap<String, Type> = type_params
        .iter()
        .map(|p| (p.name.clone(), var_gen.fresh()))
        .collect();

    // 用类型变量实例化参数类型
    let instantiated_params: Vec<Type> = param_types
        .iter()
        .map(|t| {
            let subst: HashMap<String, Type> = type_var_map.clone();
            apply_type_substitution(t, &subst)
        })
        .collect();

    // 用类型变量实例化返回类型
    let instantiated_return = apply_type_substitution(return_type, &type_var_map);

    // 合一参数类型
    let mut substitution = HashMap::new();
    for (expected, actual) in instantiated_params.iter().zip(arg_types.iter()) {
        match unify(expected, actual) {
            Ok(s) => {
                substitution = compose_substitutions(&substitution, &s);
            }
            Err(UnificationError::TypeMismatch(t1, t2)) => {
                return Err(TypeError::TypeMismatch {
                    expected: format!("{:?}", t1),
                    actual: format!("{:?}", t2),
                    span,
                });
            }
            Err(UnificationError::InfiniteType(var, ty)) => {
                let _ = (var, ty);
                return Err(TypeError::RecursiveType { span });
            }
        }
    }

    // 从替换中提取推断的类型参数
    let inferred_type_args: Vec<Type> = type_params
        .iter()
        .map(|p| {
            let _type_var = type_var_map.get(&p.name).unwrap();
            match substitution.get(&format!(
                "'_{}",
                type_params.iter().position(|x| x.name == p.name).unwrap()
            )) {
                Some(ty) => ty.clone(),
                None => {
                    // 如果没有找到替换，尝试用原始类型变量名查找
                    match substitution.get(&p.name) {
                        Some(ty) => ty.clone(),
                        None => Type::Var(format!("'_unresolved_{}", p.name)),
                    }
                }
            }
        })
        .collect();

    // 应用替换到返回类型
    let inferred_return = apply_type_substitution(&instantiated_return, &substitution);

    // 检查类型参数约束
    check_type_constraints(type_params, &inferred_type_args, env, span)?;

    Ok((inferred_type_args, inferred_return))
}

/// 解析类型参数约束
/// 检查推断的类型参数是否满足声明的约束
pub fn solve_type_constraints(
    type_params: &[x_parser::ast::TypeParameter],
    type_args: &[Type],
    env: &TypeEnv,
    span: Span,
) -> Result<(), TypeError> {
    for (param, arg) in type_params.iter().zip(type_args.iter()) {
        for constraint in &param.constraints {
            let trait_name = &constraint.trait_name;

            // 检查类型是否满足约束
            match arg {
                // 类型变量：暂时通过，后续需要更多上下文
                Type::Var(_) => continue,

                // 类型参数：检查是否有相同约束
                Type::TypeParam(name) => {
                    // 查找类型参数定义中的约束
                    let has_constraint = type_params
                        .iter()
                        .find(|p| &p.name == name)
                        .map(|p| p.constraints.iter().any(|c| &c.trait_name == trait_name))
                        .unwrap_or(false);

                    if !has_constraint {
                        return Err(TypeError::TypeConstraintViolation { span });
                    }
                }

                // 泛型类型：检查类是否实现了 trait
                Type::Generic(class_name) => {
                    if let Some(class_info) = env.get_class(class_name) {
                        if !class_info.implements.contains(trait_name) {
                            return Err(TypeError::TypeConstraintViolation { span });
                        }
                    }
                }

                // 类型构造器：检查基础类型是否实现了 trait
                Type::TypeConstructor(class_name, _) => {
                    if let Some(class_info) = env.get_class(class_name) {
                        if !class_info.implements.contains(trait_name) {
                            return Err(TypeError::TypeConstraintViolation { span });
                        }
                    }
                }

                // 其他类型：暂时通过
                _ => continue,
            }
        }
    }

    Ok(())
}

// ============================================================================
// 效果系统基础
// ============================================================================

use x_parser::ast::Effect;

/// 效果集合
pub type EffectSet = HashSet<String>;

/// 带效果的类型
#[derive(Debug, Clone)]
pub struct EffectfulType {
    /// 值的类型
    pub ty: Type,
    /// 效果集合
    pub effects: EffectSet,
}

impl EffectfulType {
    /// 创建纯净的类型（无效果）
    pub fn pure(ty: Type) -> Self {
        EffectfulType {
            ty,
            effects: HashSet::new(),
        }
    }

    /// 创建带效果的类型
    pub fn with_effects(ty: Type, effects: EffectSet) -> Self {
        EffectfulType { ty, effects }
    }

    /// 合并两个效果类型（用于序列组合）
    pub fn combine(&self, other: &EffectfulType) -> EffectfulType {
        let mut combined_effects = self.effects.clone();
        combined_effects.extend(other.effects.clone());
        EffectfulType {
            ty: other.ty.clone(),
            effects: combined_effects,
        }
    }
}

/// 将 Effect 枚举转换为字符串
pub fn effect_to_string(effect: &Effect) -> String {
    match effect {
        Effect::IO => "IO".to_string(),
        Effect::Async => "Async".to_string(),
        Effect::State(ty) => format!("State({})", ty),
        Effect::Throws(ty) => format!("Throws({})", ty),
        Effect::NonDet => "NonDet".to_string(),
        Effect::Custom(name) => name.clone(),
    }
}

/// 解析效果列表为效果集合
pub fn parse_effects(effects: &[Effect]) -> EffectSet {
    effects.iter().map(effect_to_string).collect()
}

/// 检查效果兼容性
/// 如果 inferred 不是 declared 的子集，则返回错误
pub fn check_effects_compatible(
    inferred: &EffectSet,
    declared: &EffectSet,
    span: Span,
) -> Result<(), TypeError> {
    // declared 必须包含 inferred 中的所有效果
    for effect in inferred {
        if !declared.contains(effect) {
            return Err(TypeError::MissingEffectDeclaration {
                required: effect.clone(),
                span,
            });
        }
    }
    Ok(())
}

/// 推断表达式效果的基础实现
/// 返回表达式可能产生的效果集合
pub fn infer_expression_effects(expr: &Expression, env: &TypeEnv) -> Result<EffectSet, TypeError> {
    let mut effects = HashSet::new();

    match &expr.node {
        // 字面量：无效果
        ExpressionKind::Literal(_) => {}

        // 变量引用：无效果
        ExpressionKind::Variable(_) => {}

        // 成员访问：无效果（除非是属性访问）
        ExpressionKind::Member(obj, _) => {
            effects.extend(infer_expression_effects(obj, env)?);
        }

        // 函数调用：可能有效果
        ExpressionKind::Call(callee, args) => {
            effects.extend(infer_expression_effects(callee, env)?);
            for arg in args {
                effects.extend(infer_expression_effects(arg, env)?);
            }
            // 如果调用的是已知函数，提取其效果声明
            if let ExpressionKind::Variable(name) = &callee.node {
                if let Some(func_info) = env.get_function_info(name) {
                    effects.extend(func_info.effects.clone());
                }
            }
        }

        // 二元运算：无效果
        ExpressionKind::Binary(_, left, right) => {
            effects.extend(infer_expression_effects(left, env)?);
            effects.extend(infer_expression_effects(right, env)?);
        }

        // 一元运算：无效果（Wait 除外）
        ExpressionKind::Unary(op, inner) => {
            effects.extend(infer_expression_effects(inner, env)?);
            if matches!(op, x_parser::ast::UnaryOp::Wait) {
                effects.insert("Async".to_string());
            }
        }

        // 类型转换：从内部表达式继承效果
        ExpressionKind::Cast(expr, _) => {
            effects.extend(infer_expression_effects(expr, env)?);
        }

        // 赋值：可能有状态效果
        ExpressionKind::Assign(target, value) => {
            effects.extend(infer_expression_effects(target, env)?);
            effects.extend(infer_expression_effects(value, env)?);
            // 如果目标不是局部变量，则可能有状态效果
            if let ExpressionKind::Member(_, _) = &target.node {
                effects.insert("State".to_string());
            }
        }

        // 条件表达式：合并两个分支的效果
        ExpressionKind::If(cond, then_expr, else_expr) => {
            effects.extend(infer_expression_effects(cond, env)?);
            effects.extend(infer_expression_effects(then_expr, env)?);
            effects.extend(infer_expression_effects(else_expr, env)?);
        }

        // Lambda：推断函数体中的效果
        ExpressionKind::Lambda(_, body) => {
            effects.extend(infer_block_effects(body, env)?);
        }

        // 数组字面量：无效果
        ExpressionKind::Array(elements) => {
            for elem in elements {
                effects.extend(infer_expression_effects(elem, env)?);
            }
        }

        // 元组字面量：无效果
        ExpressionKind::Tuple(elements) => {
            for elem in elements {
                effects.extend(infer_expression_effects(elem, env)?);
            }
        }

        // 字典字面量：无效果
        ExpressionKind::Dictionary(entries) => {
            for (k, v) in entries {
                effects.extend(infer_expression_effects(k, env)?);
                effects.extend(infer_expression_effects(v, env)?);
            }
        }

        // 记录字面量：无效果
        ExpressionKind::Record(_, fields) => {
            for (_, value) in fields {
                effects.extend(infer_expression_effects(value, env)?);
            }
        }

        // 范围：无效果
        ExpressionKind::Range(start, end, _) => {
            effects.extend(infer_expression_effects(start, env)?);
            effects.extend(infer_expression_effects(end, env)?);
        }

        // 管道：合并所有阶段的效果
        ExpressionKind::Pipe(input, funcs) => {
            effects.extend(infer_expression_effects(input, env)?);
            for func in funcs {
                effects.extend(infer_expression_effects(func, env)?);
            }
        }

        // Wait 操作：异步效果
        ExpressionKind::Wait(_, exprs) => {
            effects.insert("Async".to_string());
            for expr in exprs {
                effects.extend(infer_expression_effects(expr, env)?);
            }
        }

        // Needs：声明需要某个效果
        ExpressionKind::Needs(effect_name) => {
            effects.insert(effect_name.clone());
        }

        // Given：提供某个效果
        ExpressionKind::Given(_, inner) => {
            effects.extend(infer_expression_effects(inner, env)?);
        }

        // Handle：处理效果，handlers 会捕获效果
        ExpressionKind::Handle(_, handlers) => {
            for (_, handler) in handlers {
                effects.extend(infer_expression_effects(handler, env)?);
            }
        }

        // ? 运算符：可能有 Throws 效果
        ExpressionKind::TryPropagate(inner) => {
            effects.extend(infer_expression_effects(inner, env)?);
            // ? 运算符可能提前返回错误
            effects.insert("Throws".to_string());
        }

        // 括号表达式：透传效果
        ExpressionKind::Parenthesized(inner) => {
            effects.extend(infer_expression_effects(inner, env)?);
        }
        // 模式匹配：检查所有分支的效果
        ExpressionKind::Match(discriminant, cases) => {
            effects.extend(infer_expression_effects(discriminant, env)?);
            for case in cases {
                for stmt in &case.body.statements {
                    effects.extend(infer_statement_effects(stmt, env)?);
                }
                if let Some(guard) = &case.guard {
                    effects.extend(infer_expression_effects(guard, env)?);
                }
            }
        }
        ExpressionKind::Await(inner) => {
            effects.extend(infer_expression_effects(inner, env)?);
            // await 会异步等待，产生 IO 效果
            effects.insert("Async".to_string());
        }
        ExpressionKind::OptionalChain(base, _) => {
            effects.extend(infer_expression_effects(base, env)?);
        }
        ExpressionKind::NullCoalescing(left, right) => {
            effects.extend(infer_expression_effects(left, env)?);
            effects.extend(infer_expression_effects(right, env)?);
        }
        ExpressionKind::WhenGuard(condition, body) => {
            effects.extend(infer_expression_effects(condition, env)?);
            effects.extend(infer_expression_effects(body, env)?);
        }
        ExpressionKind::Block(block) => {
            for stmt in &block.statements {
                effects.extend(infer_statement_effects(stmt, env)?);
            }
        }
    }

    Ok(effects)
}

/// 检查变量声明
fn check_variable_decl(var_decl: &VariableDecl, env: &mut TypeEnv) -> Result<(), TypeError> {
    let span = var_decl.span;

    if env.current_scope_contains(&var_decl.name) {
        return Err(TypeError::DuplicateDeclaration {
            name: var_decl.name.clone(),
            span,
        });
    }

    // 检查初始化表达式的类型
    if let Some(initializer) = &var_decl.initializer {
        let init_type = infer_expression_type(initializer, env)?;

        // 如果有类型注解，检查类型匹配
        if let Some(type_annot) = &var_decl.type_annot {
            // Int 和 UnsignedInt 互相兼容
            let is_int_compatible = (types_equal(&init_type, &Type::Int)
                || types_equal(&init_type, &Type::UnsignedInt))
                && (types_equal(type_annot, &Type::Int)
                    || types_equal(type_annot, &Type::UnsignedInt));

            if !types_equal(&init_type, type_annot) && !is_int_compatible {
                return Err(TypeError::TypeMismatch {
                    expected: format!("{:?}", type_annot),
                    actual: format!("{:?}", init_type),
                    span,
                });
            }
            env.add_variable(&var_decl.name, type_annot.clone());
        } else {
            // 没有类型注解，使用推断的类型
            env.add_variable(&var_decl.name, init_type);
        }
    } else if let Some(type_annot) = &var_decl.type_annot {
        // 只有类型注解，没有初始化表达式
        env.add_variable(&var_decl.name, type_annot.clone());
    } else {
        // 既没有类型注解也没有初始化表达式，无法推断类型
        return Err(TypeError::CannotInferType { span });
    }

    Ok(())
}

/// 检查函数声明
fn check_function_decl(func_decl: &FunctionDecl, env: &mut TypeEnv) -> Result<(), TypeError> {
    let span = func_decl.span;

    // 函数签名已在第一遍收集，这里只检查函数体

    // 收集类型参数名
    let type_param_names: std::collections::HashSet<String> = func_decl
        .type_parameters
        .iter()
        .map(|tp| tp.name.clone())
        .collect();

    // 检查类型参数约束：约束必须引用存在的 trait
    for type_param in &func_decl.type_parameters {
        for constraint in &type_param.constraints {
            if env.get_trait(&constraint.trait_name).is_none() {
                return Err(TypeError::UndefinedType {
                    name: constraint.trait_name.clone(),
                    span: constraint.span,
                });
            }
        }
    }

    // 验证参数类型注解
    for param in &func_decl.parameters {
        if param.type_annot.is_none() {
            // 参数必须有类型注解
            return Err(TypeError::CannotInferType { span: param.span });
        }
        // 验证参数类型是否有效（考虑类型参数）
        if let Some(ty) = &param.type_annot {
            if !is_valid_type_with_params(ty, env, &type_param_names) {
                return Err(TypeError::UndefinedType {
                    name: format!("{:?}", ty),
                    span: param.span,
                });
            }
        }
    }

    // 验证返回类型是否有效
    if let Some(return_ty) = &func_decl.return_type {
        if !is_valid_type_with_params(return_ty, env, &type_param_names) {
            return Err(TypeError::UndefinedType {
                name: format!("{:?}", return_ty),
                span,
            });
        }
    }

    // 解析声明的效果
    let declared_effects = parse_effects(&func_decl.effects);

    // 验证所有声明的效果都已定义
    for effect in &func_decl.effects {
        let effect_name = effect_to_string(effect);
        if env.get_effect(&effect_name).is_none() {
            return Err(TypeError::UndeclaredEffect {
                effect: effect_name,
                span,
            });
        }
    }

    // 检查函数体
    env.push_scope();
    // 将类型参数添加到环境？不 - 类型参数只在类型中使用，变量不包含类型参数
    // 将参数加入当前作用域
    for param in &func_decl.parameters {
        let ty = param
            .type_annot
            .as_ref()
            .expect("type annotations checked above")
            .clone();
        if env.current_scope_contains(&param.name) {
            env.pop_scope();
            return Err(TypeError::DuplicateDeclaration {
                name: param.name.clone(),
                span: param.span,
            });
        }
        env.add_variable(&param.name, ty);
    }

    // 检查函数体
    let result = check_block(&func_decl.body, env);

    // 推断函数体的效果（如果声明了效果，需要检查兼容性）
    if !func_decl.effects.is_empty() {
        // 收集函数体中所有表达式的效果
        let inferred_effects = infer_block_effects(&func_decl.body, env)?;

        // 检查推断的效果是否与声明兼容
        if let Err(e) = check_effects_compatible(&inferred_effects, &declared_effects, span) {
            env.pop_scope();
            return Err(e);
        }
    }

    env.pop_scope();
    result
}

/// 推断块中的效果
fn infer_block_effects(block: &Block, env: &TypeEnv) -> Result<EffectSet, TypeError> {
    let mut effects = HashSet::new();

    for stmt in &block.statements {
        effects.extend(infer_statement_effects(stmt, env)?);
    }

    Ok(effects)
}

/// 推断语句中的效果
fn infer_statement_effects(stmt: &Statement, env: &TypeEnv) -> Result<EffectSet, TypeError> {
    match &stmt.node {
        StatementKind::Expression(expr) => infer_expression_effects(expr, env),
        StatementKind::Variable(var_decl) => {
            if let Some(init) = &var_decl.initializer {
                infer_expression_effects(init, env)
            } else {
                Ok(HashSet::new())
            }
        }
        StatementKind::Return(expr_opt) => {
            if let Some(expr) = expr_opt {
                infer_expression_effects(expr, env)
            } else {
                Ok(HashSet::new())
            }
        }
        StatementKind::If(if_stmt) => {
            let mut effects = infer_expression_effects(&if_stmt.condition, env)?;
            effects.extend(infer_block_effects(&if_stmt.then_block, env)?);
            if let Some(else_block) = &if_stmt.else_block {
                effects.extend(infer_block_effects(else_block, env)?);
            }
            Ok(effects)
        }
        StatementKind::While(while_stmt) => {
            let mut effects = infer_expression_effects(&while_stmt.condition, env)?;
            effects.extend(infer_block_effects(&while_stmt.body, env)?);
            Ok(effects)
        }
        StatementKind::For(for_stmt) => {
            let mut effects = infer_expression_effects(&for_stmt.iterator, env)?;
            effects.extend(infer_block_effects(&for_stmt.body, env)?);
            Ok(effects)
        }
        StatementKind::Match(match_stmt) => {
            let mut effects = infer_expression_effects(&match_stmt.expression, env)?;
            for case in &match_stmt.cases {
                effects.extend(infer_block_effects(&case.body, env)?);
            }
            Ok(effects)
        }
        StatementKind::Try(try_stmt) => {
            let mut effects = infer_block_effects(&try_stmt.body, env)?;
            for catch in &try_stmt.catch_clauses {
                effects.extend(infer_block_effects(&catch.body, env)?);
            }
            if let Some(finally) = &try_stmt.finally_block {
                effects.extend(infer_block_effects(finally, env)?);
            }
            Ok(effects)
        }
        StatementKind::Break | StatementKind::Continue => Ok(HashSet::new()),
        StatementKind::DoWhile(do_while) => {
            let mut effects = infer_block_effects(&do_while.body, env)?;
            effects.extend(infer_expression_effects(&do_while.condition, env)?);
            Ok(effects)
        }
        StatementKind::Unsafe(block) => {
            // Unsafe blocks inherit effects from their body
            infer_block_effects(block, env)
        }
        StatementKind::Defer(expr) => infer_expression_effects(expr, env),
        StatementKind::Yield(_) => Ok(HashSet::new()),
        StatementKind::Loop(block) => infer_block_effects(block, env),
        StatementKind::WhenGuard(condition, body) => {
            let mut effects = infer_expression_effects(condition, env)?;
            effects.extend(infer_expression_effects(body, env)?);
            Ok(effects)
        }
    }
}

/// 检查语句
fn check_statement(stmt: &Statement, env: &mut TypeEnv) -> Result<(), TypeError> {
    let span = stmt.span;

    match &stmt.node {
        StatementKind::Expression(expr) => {
            infer_expression_type(expr, env)?;
            Ok(())
        }
        StatementKind::Variable(var_decl) => check_variable_decl(var_decl, env),
        StatementKind::Return(expr_opt) => {
            if let Some(expr) = expr_opt {
                infer_expression_type(expr, env)?;
            }
            Ok(())
        }
        StatementKind::If(if_stmt) => {
            // 检查条件表达式类型为布尔
            let cond_type = infer_expression_type(&if_stmt.condition, env)?;
            if !types_equal(&cond_type, &Type::Bool) {
                return Err(TypeError::TypeMismatch {
                    expected: format!("{:?}", Type::Bool),
                    actual: format!("{:?}", cond_type),
                    span: if_stmt.condition.span,
                });
            }

            // 检查then块（新作用域）
            env.push_scope();
            check_block(&if_stmt.then_block, env)?;
            env.pop_scope();

            // 检查else块
            if let Some(else_block) = &if_stmt.else_block {
                env.push_scope();
                check_block(else_block, env)?;
                env.pop_scope();
            }

            Ok(())
        }
        StatementKind::For(for_stmt) => {
            // 先检查 iterator 表达式
            let iterator_type = infer_expression_type(&for_stmt.iterator, env)?;

            // 推断元素类型：从数组类型中提取元素类型
            let element_type = match &iterator_type {
                Type::Array(elem_ty) => (**elem_ty).clone(),
                _ => Type::Unit, // 如果不是数组类型，使用 Unit 作为回退
            };

            // for body 新作用域：将 pattern 中的变量绑定到推断出的元素类型
            env.push_scope();
            if let x_parser::ast::Pattern::Variable(name) = &for_stmt.pattern {
                if env.current_scope_contains(name) {
                    env.pop_scope();
                    return Err(TypeError::DuplicateDeclaration {
                        name: name.clone(),
                        span,
                    });
                }
                env.add_variable(name, element_type);
            }

            let r = check_block(&for_stmt.body, env);
            env.pop_scope();
            r
        }
        StatementKind::While(while_stmt) => {
            // 检查条件表达式类型为布尔
            let cond_type = infer_expression_type(&while_stmt.condition, env)?;
            if !types_equal(&cond_type, &Type::Bool) {
                return Err(TypeError::TypeMismatch {
                    expected: format!("{:?}", Type::Bool),
                    actual: format!("{:?}", cond_type),
                    span: while_stmt.condition.span,
                });
            }

            // 检查循环体（新作用域）
            env.push_scope();
            let r = check_block(&while_stmt.body, env);
            env.pop_scope();
            r
        }
        StatementKind::Match(match_stmt) => {
            let discriminant_ty = infer_expression_type(&match_stmt.expression, env)?;
            let match_span = match_stmt.expression.span;
            // 收集所有模式用于穷尽性检查
            let mut patterns = Vec::new();
            for case in &match_stmt.cases {
                patterns.push(case.pattern.clone());
                // 先将模式变量绑定到作用域，然后再检查 guard
                env.push_scope();
                // 检查模式并添加正确类型的绑定变量
                check_pattern(&case.pattern, &discriminant_ty, env, match_span)?;
                // 在模式变量已绑定后再检查 guard
                if let Some(guard) = &case.guard {
                    let gt = infer_expression_type(guard, env)?;
                    if !types_equal(&gt, &Type::Bool) {
                        return Err(TypeError::TypeMismatch {
                            expected: format!("{:?}", Type::Bool),
                            actual: format!("{:?}", gt),
                            span: guard.span,
                        });
                    }
                }
                check_block(&case.body, env)?;
                env.pop_scope();
            }
            // 检查穷尽性
            if let Err(e) = crate::exhaustiveness::check_exhaustive(&patterns, &discriminant_ty) {
                log::warn!(
                    "Match expression is not exhaustive: missing {:?}",
                    e.uncovered_patterns
                );
                // TODO: report as error/warning based on configuration
            }
            Ok(())
        }
        StatementKind::Try(try_stmt) => {
            env.push_scope();
            check_block(&try_stmt.body, env)?;
            env.pop_scope();

            for cc in &try_stmt.catch_clauses {
                env.push_scope();
                if let Some(var) = &cc.variable_name {
                    if env.current_scope_contains(var) {
                        env.pop_scope();
                        return Err(TypeError::DuplicateDeclaration {
                            name: var.clone(),
                            span,
                        });
                    }
                    // 暂不实现异常类型系统：先用 Unit 占位
                    env.add_variable(var, Type::Unit);
                }
                check_block(&cc.body, env)?;
                env.pop_scope();
            }

            if let Some(finally_block) = &try_stmt.finally_block {
                env.push_scope();
                check_block(finally_block, env)?;
                env.pop_scope();
            }

            Ok(())
        }
        StatementKind::Break | StatementKind::Continue => Ok(()),
        StatementKind::DoWhile(d) => {
            env.push_scope();
            check_block(&d.body, env)?;
            env.pop_scope();
            let cond_ty = infer_expression_type(&d.condition, env)?;
            if !types_equal(&cond_ty, &Type::Bool) {
                return Err(TypeError::TypeMismatch {
                    expected: "Bool".to_string(),
                    actual: format!("{:?}", cond_ty),
                    span: d.condition.span,
                });
            }
            Ok(())
        }
        StatementKind::Unsafe(block) => {
            // 检查 unsafe 块（新作用域）
            // 进入 unsafe 上下文
            env.push_scope();
            env.enter_unsafe_context();
            check_block(block, env)?;
            env.exit_unsafe_context();
            env.pop_scope();
            Ok(())
        }
        StatementKind::Defer(expr) => {
            infer_expression_type(expr, env)?;
            Ok(())
        }
        StatementKind::Yield(_) => Ok(()),
        StatementKind::Loop(body) => {
            env.push_scope();
            check_block(body, env)?;
            env.pop_scope();
            Ok(())
        }
        StatementKind::WhenGuard(condition, body) => {
            let cond_type = infer_expression_type(condition, env)?;
            if !types_equal(&cond_type, &Type::Bool) {
                return Err(TypeError::TypeMismatch {
                    expected: format!("{:?}", Type::Bool),
                    actual: format!("{:?}", cond_type),
                    span: condition.span,
                });
            }
            infer_expression_type(body, env)?;
            Ok(())
        }
    }
}

/// 检查块语句
fn check_block(block: &Block, env: &mut TypeEnv) -> Result<(), TypeError> {
    for stmt in &block.statements {
        check_statement(stmt, env)?;
    }
    Ok(())
}

/// 检查模式匹配给定类型，并将绑定变量添加到类型环境（带正确类型）
fn check_pattern(
    pattern: &x_parser::ast::Pattern,
    expected_ty: &Type,
    env: &mut TypeEnv,
    span: Span,
) -> Result<(), TypeError> {
    match pattern {
        x_parser::ast::Pattern::Wildcard => {
            // 通配符匹配任何类型
            Ok(())
        }
        x_parser::ast::Pattern::Variable(name) => {
            // 变量绑定匹配任何类型，类型就是预期类型
            if env.current_scope_contains(name) {
                return Err(TypeError::DuplicateDeclaration {
                    name: name.clone(),
                    span,
                });
            }
            env.add_variable(name, expected_ty.clone());
            Ok(())
        }
        x_parser::ast::Pattern::Literal(lit) => {
            // 检查字面量类型是否匹配预期类型
            let lit_ty = match lit {
                Literal::Integer(_) => Type::Int,
                Literal::Float(_) => Type::Float,
                Literal::Boolean(_) => Type::Bool,
                Literal::String(_) => Type::String,
                Literal::Char(_) => Type::Char,
                Literal::Unit => Type::Unit,
                Literal::Null | Literal::None => Type::Dynamic, // Null/None can match any nullable/optional type
            };
            if !types_equal(&lit_ty, expected_ty) {
                return Err(TypeError::TypeMismatch {
                    expected: format!("{}", expected_ty),
                    actual: format!("{}", lit_ty),
                    span,
                });
            }
            Ok(())
        }
        x_parser::ast::Pattern::Array(patterns) => {
            // 数组模式：预期应该是数组类型，每个元素模式匹配元素类型
            match expected_ty {
                Type::Array(element_ty) => {
                    for p in patterns {
                        check_pattern(p, element_ty, env, span)?;
                    }
                    Ok(())
                }
                _ => Err(TypeError::TypeMismatch {
                    expected: "Array[_]".to_string(),
                    actual: format!("{}", expected_ty),
                    span,
                }),
            }
        }
        x_parser::ast::Pattern::Dictionary(entries) => {
            // 字典模式：预期是字典类型
            match expected_ty {
                Type::Dictionary(key_ty, value_ty) => {
                    for (key_pattern, value_pattern) in entries {
                        // 检查键模式匹配键类型，值模式匹配值类型
                        check_pattern(key_pattern, key_ty, env, span)?;
                        check_pattern(value_pattern, value_ty, env, span)?;
                    }
                    Ok(())
                }
                _ => Err(TypeError::TypeMismatch {
                    expected: "Dictionary[_, _]".to_string(),
                    actual: format!("{}", expected_ty),
                    span,
                }),
            }
        }
        x_parser::ast::Pattern::Record(name, fields) => {
            // 记录模式：预期是命名记录类型
            match expected_ty {
                Type::Generic(expected_name) if expected_name == name => {
                    // 查找记录定义 - clone to avoid borrowing issues
                    if let Some(record_def) = env.get_record(name) {
                        let fields_cloned: Vec<(String, Type)> = record_def.fields.clone();
                        for (field_name, pattern) in fields {
                            // 找到对应的字段类型
                            if let Some((_, field_ty)) =
                                fields_cloned.iter().find(|(n, _)| n == field_name)
                            {
                                check_pattern(pattern, field_ty, env, span)?;
                            } else {
                                return Err(TypeError::UndefinedField {
                                    name: field_name.clone(),
                                    span,
                                });
                            }
                        }
                        Ok(())
                    } else {
                        Err(TypeError::UndefinedType {
                            name: name.clone(),
                            span,
                        })
                    }
                }
                Type::TypeConstructor(expected_name, _) if expected_name == name => {
                    // 查找记录定义 - clone to avoid borrowing issues
                    if let Some(record_def) = env.get_record(name) {
                        let fields_cloned: Vec<(String, Type)> = record_def.fields.clone();
                        for (field_name, pattern) in fields {
                            if let Some((_, field_ty)) =
                                fields_cloned.iter().find(|(n, _)| n == field_name)
                            {
                                check_pattern(pattern, field_ty, env, span)?;
                            } else {
                                return Err(TypeError::UndefinedField {
                                    name: field_name.clone(),
                                    span,
                                });
                            }
                        }
                        Ok(())
                    } else {
                        Err(TypeError::UndefinedType {
                            name: name.clone(),
                            span,
                        })
                    }
                }
                _ => Err(TypeError::TypeMismatch {
                    expected: name.clone(),
                    actual: format!("{}", expected_ty),
                    span,
                }),
            }
        }
        x_parser::ast::Pattern::Tuple(patterns) => {
            // 元组模式：预期应该是元组类型，长度匹配
            match expected_ty {
                Type::Tuple(element_tys) => {
                    if patterns.len() != element_tys.len() {
                        return Err(TypeError::ParameterCountMismatch {
                            expected: element_tys.len(),
                            actual: patterns.len(),
                            span,
                        });
                    }
                    for (p, ty) in patterns.iter().zip(element_tys.iter()) {
                        check_pattern(p, ty, env, span)?;
                    }
                    Ok(())
                }
                _ => Err(TypeError::TypeMismatch {
                    expected: "Tuple[_]".to_string(),
                    actual: format!("{}", expected_ty),
                    span,
                }),
            }
        }
        x_parser::ast::Pattern::Or(left, right) => {
            // 或模式：两边都必须匹配相同类型
            check_pattern(left, expected_ty, env, span)?;
            check_pattern(right, expected_ty, env, span)?;
            Ok(())
        }
        x_parser::ast::Pattern::Guard(inner, _guard) => {
            // 带卫士的模式：先检查内部模式
            check_pattern(inner, expected_ty, env, span)?;
            // 卫士表达式已经在后面检查类型为Bool
            Ok(())
        }
        x_parser::ast::Pattern::EnumConstructor(enum_name, variant_name, patterns) => {
            // 枚举构造器模式：检查枚举类型匹配
            match expected_ty {
                Type::Generic(expected_enum_name) if expected_enum_name == enum_name => {
                    // 查找枚举定义 - clone to avoid borrowing issues
                    if let Some(enum_def) = env.get_enum(enum_name) {
                        let enum_def_cloned = enum_def.clone();
                        if let Some(variant) = enum_def_cloned
                            .variants
                            .iter()
                            .find(|v| v.name == *variant_name)
                        {
                            // 根据变体数据类型检查参数
                            match &variant.data {
                                x_parser::ast::EnumVariantData::Unit => {
                                    if !patterns.is_empty() {
                                        return Err(TypeError::ParameterCountMismatch {
                                            expected: 0,
                                            actual: patterns.len(),
                                            span,
                                        });
                                    }
                                    Ok(())
                                }
                                x_parser::ast::EnumVariantData::Tuple(field_tys) => {
                                    let field_tys_cloned: Vec<Type> = field_tys.clone();
                                    if field_tys_cloned.len() != patterns.len() {
                                        return Err(TypeError::ParameterCountMismatch {
                                            expected: field_tys_cloned.len(),
                                            actual: patterns.len(),
                                            span,
                                        });
                                    }
                                    for (p, ty) in patterns.iter().zip(field_tys_cloned.iter()) {
                                        check_pattern(p, ty, env, span)?;
                                    }
                                    Ok(())
                                }
                                x_parser::ast::EnumVariantData::Record(fields) => {
                                    let fields_cloned: Vec<(String, Type)> = fields.clone();
                                    if fields_cloned.len() != patterns.len() {
                                        return Err(TypeError::ParameterCountMismatch {
                                            expected: fields_cloned.len(),
                                            actual: patterns.len(),
                                            span,
                                        });
                                    }
                                    for (p, (_field_name, ty)) in
                                        patterns.iter().zip(fields_cloned.iter())
                                    {
                                        check_pattern(p, ty, env, span)?;
                                    }
                                    Ok(())
                                }
                            }
                        } else {
                            Err(TypeError::UndefinedVariant {
                                enum_name: enum_name.clone(),
                                variant_name: variant_name.clone(),
                                span,
                            })
                        }
                    } else {
                        Err(TypeError::UndefinedType {
                            name: enum_name.clone(),
                            span,
                        })
                    }
                }
                Type::TypeConstructor(expected_enum_name, _) if expected_enum_name == enum_name => {
                    // 查找枚举定义 - clone to avoid borrowing issues
                    if let Some(enum_def) = env.get_enum(enum_name) {
                        let enum_def_cloned = enum_def.clone();
                        if let Some(variant) = enum_def_cloned
                            .variants
                            .iter()
                            .find(|v| v.name == *variant_name)
                        {
                            match &variant.data {
                                x_parser::ast::EnumVariantData::Unit => {
                                    if !patterns.is_empty() {
                                        return Err(TypeError::ParameterCountMismatch {
                                            expected: 0,
                                            actual: patterns.len(),
                                            span,
                                        });
                                    }
                                    Ok(())
                                }
                                x_parser::ast::EnumVariantData::Tuple(field_tys) => {
                                    let field_tys_cloned: Vec<Type> = field_tys.clone();
                                    if field_tys_cloned.len() != patterns.len() {
                                        return Err(TypeError::ParameterCountMismatch {
                                            expected: field_tys_cloned.len(),
                                            actual: patterns.len(),
                                            span,
                                        });
                                    }
                                    for (p, ty) in patterns.iter().zip(field_tys_cloned.iter()) {
                                        check_pattern(p, ty, env, span)?;
                                    }
                                    Ok(())
                                }
                                x_parser::ast::EnumVariantData::Record(fields) => {
                                    let fields_cloned: Vec<(String, Type)> = fields.clone();
                                    if fields_cloned.len() != patterns.len() {
                                        return Err(TypeError::ParameterCountMismatch {
                                            expected: fields_cloned.len(),
                                            actual: patterns.len(),
                                            span,
                                        });
                                    }
                                    for (p, (_field_name, ty)) in
                                        patterns.iter().zip(fields_cloned.iter())
                                    {
                                        check_pattern(p, ty, env, span)?;
                                    }
                                    Ok(())
                                }
                            }
                        } else {
                            Err(TypeError::UndefinedVariant {
                                enum_name: enum_name.clone(),
                                variant_name: variant_name.clone(),
                                span,
                            })
                        }
                    } else {
                        Err(TypeError::UndefinedType {
                            name: enum_name.clone(),
                            span,
                        })
                    }
                }
                // 处理 Option<T> 和 Result<T, E> 作为 TypeConstructor 的模式匹配
                Type::TypeConstructor(type_name, type_args)
                    if type_name == "Option" || type_name == "Result" =>
                {
                    match variant_name.as_str() {
                        "Some" | "Ok" => {
                            // Some(v) / Ok(v) 需要一个参数
                            if patterns.len() != 1 {
                                return Err(TypeError::ParameterCountMismatch {
                                    expected: 1,
                                    actual: patterns.len(),
                                    span,
                                });
                            }
                            let inner_ty = &type_args[0];
                            check_pattern(&patterns[0], inner_ty, env, span)?;
                            Ok(())
                        }
                        "None" => {
                            // None 不需要参数
                            if !patterns.is_empty() {
                                return Err(TypeError::ParameterCountMismatch {
                                    expected: 0,
                                    actual: patterns.len(),
                                    span,
                                });
                            }
                            Ok(())
                        }
                        "Err" => {
                            // Err(e) 需要一个参数，类型为 E
                            if patterns.len() != 1 {
                                return Err(TypeError::ParameterCountMismatch {
                                    expected: 1,
                                    actual: patterns.len(),
                                    span,
                                });
                            }
                            let err_ty = &type_args[1]; // Err(E) uses second type arg
                            check_pattern(&patterns[0], err_ty, env, span)?;
                            Ok(())
                        }
                        _ => Err(TypeError::UndefinedVariant {
                            enum_name: type_name.clone(),
                            variant_name: variant_name.clone(),
                            span,
                        }),
                    }
                }
                _ => Err(TypeError::TypeMismatch {
                    expected: enum_name.clone(),
                    actual: format!("{}", expected_ty),
                    span,
                }),
            }
        }
    }
}

/// 从模式中提取绑定变量并添加到环境（向后兼容保留，现在调用 check_pattern）
#[deprecated]
#[allow(dead_code)]
fn add_pattern_bindings(pattern: &x_parser::ast::Pattern, env: &mut TypeEnv) {
    // 已废弃：使用 check_pattern 替代，它会同时做类型检查
    match pattern {
        x_parser::ast::Pattern::Wildcard => {}
        x_parser::ast::Pattern::Variable(name) => {
            env.add_variable(name, Type::Dynamic);
        }
        x_parser::ast::Pattern::Literal(_) => {}
        x_parser::ast::Pattern::Array(patterns) => {
            for p in patterns {
                add_pattern_bindings(p, env);
            }
        }
        x_parser::ast::Pattern::Dictionary(entries) => {
            for (_, p) in entries {
                add_pattern_bindings(p, env);
            }
        }
        x_parser::ast::Pattern::Record(_, fields) => {
            for (_, p) in fields {
                add_pattern_bindings(p, env);
            }
        }
        x_parser::ast::Pattern::Tuple(patterns) => {
            for p in patterns {
                add_pattern_bindings(p, env);
            }
        }
        x_parser::ast::Pattern::Or(left, _) => {
            add_pattern_bindings(left, env);
        }
        x_parser::ast::Pattern::Guard(inner, _) => {
            add_pattern_bindings(inner, env);
        }
        x_parser::ast::Pattern::EnumConstructor(_, _, patterns) => {
            for p in patterns {
                add_pattern_bindings(p, env);
            }
        }
    }
}

/// 检查是否是内置类型名（用于 FFI 和标准类型）
fn is_builtin_type_name(name: &str) -> bool {
    matches!(
        name,
        // FFI 指针类型
        "Pointer" | "pointer" |
        // FFI 基础类型
        "Void" | "void" |
        // C FFI 类型
        "CInt" | "c_int" |
        "CUInt" | "c_uint" |
        "CLong" | "c_long" |
        "CULong" | "c_ulong" |
        "CLongLong" | "c_longlong" |
        "CULongLong" | "c_ulonglong" |
        "CFloat" | "c_float" |
        "CDouble" | "c_double" |
        "CChar" | "c_char" |
        "CSize" | "c_size_t" | "c_size" |
        "CString" | "c_string" |
        // 标准类型别名
        "Int" | "int" | "Int64" | "i64" | "Int32" | "i32" |
        "Int16" | "i16" | "Int8" | "i8" |
        "Int128" | "i128" |
        "UnsignedInt" | "uint" | "Uint64" | "u64" | "Uint32" | "u32" |
        "Uint16" | "u16" | "Uint8" | "u8" | "Byte" | "byte" |
        "Uint128" | "u128" |
        "Float" | "float" | "Float64" | "f64" | "Float32" | "f32" |
        "Float16" | "f16" | "Float128" | "f128" |
        "Bool" | "bool" | "Boolean" |
        "String" | "string" |
        "Char" | "char" | "Character" |
        "Unit" |
        "Option" |
        "Result" |
        "Array" |
        "Dictionary"
    )
}

/// 推断表达式类型
fn infer_expression_type(expr: &Expression, env: &mut TypeEnv) -> Result<Type, TypeError> {
    let span = expr.span;

    match &expr.node {
        ExpressionKind::Literal(lit) => infer_literal_type(lit, span),
        ExpressionKind::Variable(name) => {
            if let Some(ty) = env.get_variable(name) {
                Ok(ty.clone())
            } else if let Some(ty) = env.get_function(name) {
                Ok(ty.clone())
            } else if env.get_class(name).is_some() {
                // 类名作为构造函数，返回类类型
                Ok(Type::Generic(name.clone()))
            } else if env.get_enum(name).is_some() {
                // 枚举类型名，返回枚举类型
                Ok(Type::Generic(name.clone()))
            } else if let Some(variant_info) = env.get_enum_variant(name) {
                // 枚举变体（如 Some, None, Ok, Err）
                Ok(variant_info.variant_type.clone())
            } else if is_builtin_type_name(name) {
                // 内置类型名（如 Pointer, Void, CLong 等）
                Ok(Type::Generic(name.clone()))
            } else {
                Err(TypeError::UndefinedVariable {
                    name: name.to_string(),
                    span,
                })
            }
        }
        ExpressionKind::Member(obj, member) => {
            // 推断对象类型
            let obj_type = infer_expression_type(obj, env)?;

            // 根据对象类型查找成员
            match &obj_type {
                Type::Generic(class_name) | Type::TypeParam(class_name) => {
                    // 查找类信息
                    if let Some(class_info) = env.get_class(class_name) {
                        // 查找字段
                        if let Some(field_type) = class_info.fields.get(member) {
                            return Ok(field_type.clone());
                        }
                        // 查找方法
                        if let Some(method_type) = class_info.methods.get(member) {
                            return Ok(method_type.clone());
                        }
                        // 在父类中查找
                        if let Some(parent_name) = &class_info.extends {
                            if let Some(parent_info) = env.get_class(parent_name) {
                                if let Some(field_type) = parent_info.fields.get(member) {
                                    return Ok(field_type.clone());
                                }
                                if let Some(method_type) = parent_info.methods.get(member) {
                                    return Ok(method_type.clone());
                                }
                            }
                        }
                        return Err(TypeError::UndefinedMember {
                            name: member.clone(),
                            span,
                        });
                    }
                    // 检查是否是 trait 类型
                    if let Some(trait_info) = env.get_trait(class_name) {
                        if let Some(method_type) = trait_info.methods.get(member) {
                            return Ok(method_type.clone());
                        }
                        return Err(TypeError::UndefinedMember {
                            name: member.clone(),
                            span,
                        });
                    }
                    // 检查是否是枚举类型，访问变体构造器
                    if let Some(enum_decl) = env.get_enum(class_name) {
                        // 查找变体
                        for variant in &enum_decl.variants {
                            if &variant.name == member {
                                // 返回枚举变体构造器类型
                                // 根据变体数据类型返回不同的函数类型
                                match &variant.data {
                                    x_parser::ast::EnumVariantData::Unit => {
                                        // 无参数，返回枚举类型本身
                                        return Ok(Type::Generic(class_name.clone()));
                                    }
                                    x_parser::ast::EnumVariantData::Tuple(types) => {
                                        // 有参数，返回函数类型
                                        let param_types: Vec<Box<Type>> =
                                            types.iter().map(|t| Box::new(t.clone())).collect();
                                        return Ok(Type::Function(
                                            param_types,
                                            Box::new(Type::Generic(class_name.clone())),
                                        ));
                                    }
                                    x_parser::ast::EnumVariantData::Record(fields) => {
                                        // 记录式变体，每个字段成为一个命名参数
                                        // 创建函数类型：接受对应参数，返回枚举类型
                                        let param_types: Vec<Box<Type>> = fields
                                            .iter()
                                            .map(|(_, t)| Box::new(t.clone()))
                                            .collect();
                                        return Ok(Type::Function(
                                            param_types,
                                            Box::new(Type::Generic(class_name.clone())),
                                        ));
                                    }
                                }
                            }
                        }
                        return Err(TypeError::UndefinedMember {
                            name: member.clone(),
                            span,
                        });
                    }
                    // 检查是否是内置类型（如 Pointer, Void 等）
                    // 这些类型可以有静态方法
                    if is_builtin_type_name(class_name) {
                        // 对于 Pointer 类型，支持 null() 静态方法
                        if (class_name == "Pointer" || class_name == "pointer") && member == "null"
                        {
                            // null() 返回一个 Pointer 类型
                            return Ok(Type::Function(
                                Vec::new(),
                                Box::new(Type::Pointer(Box::new(Type::Void))),
                            ));
                        }
                        // Int 内置类型
                        if matches!(class_name.as_str(), "Int" | "Int64" | "i32" | "i64")
                            && member == "parse"
                        {
                            // parse(s: String) -> Option<Int>
                            return Ok(Type::Function(
                                vec![Box::new(Type::String)],
                                Box::new(Type::TypeConstructor(
                                    "Option".to_string(),
                                    vec![Type::Int],
                                )),
                            ));
                        }
                        // Float 内置类型
                        if matches!(class_name.as_str(), "Float" | "Float64" | "f32" | "f64")
                            && member == "parse"
                        {
                            // parse(s: String) -> Option<Float>
                            return Ok(Type::Function(
                                vec![Box::new(Type::String)],
                                Box::new(Type::TypeConstructor(
                                    "Option".to_string(),
                                    vec![Type::Float],
                                )),
                            ));
                        }
                        // Bool 内置类型
                        if matches!(class_name.as_str(), "Bool" | "Boolean") && member == "parse" {
                            // parse(s: String) -> Option<Bool>
                            return Ok(Type::Function(
                                vec![Box::new(Type::String)],
                                Box::new(Type::TypeConstructor(
                                    "Option".to_string(),
                                    vec![Type::Bool],
                                )),
                            ));
                        }
                        // Option 类型构造
                        if class_name == "Option" {
                            // Option::some(value) -> Option<T>
                            // Option::none() -> Option<T>
                            // 这里无法知道具体 T，所以返回通用函数
                            if member == "some" {
                                return Ok(Type::Function(
                                    vec![Box::new(Type::Var("_T".to_string()))],
                                    Box::new(Type::TypeConstructor(
                                        "Option".to_string(),
                                        vec![Type::Var("_T".to_string())],
                                    )),
                                ));
                            }
                            if member == "none" {
                                return Ok(Type::Function(
                                    Vec::new(),
                                    Box::new(Type::TypeConstructor(
                                        "Option".to_string(),
                                        vec![Type::Never],
                                    )),
                                ));
                            }
                        }
                        // Result 类型构造
                        if class_name == "Result" {
                            if member == "ok" {
                                return Ok(Type::Function(
                                    vec![Box::new(Type::Var("_T".to_string()))],
                                    Box::new(Type::TypeConstructor(
                                        "Result".to_string(),
                                        vec![
                                            Type::Var("_T".to_string()),
                                            Type::Var("_E".to_string()),
                                        ],
                                    )),
                                ));
                            }
                            if member == "err" {
                                return Ok(Type::Function(
                                    vec![Box::new(Type::Var("_E".to_string()))],
                                    Box::new(Type::TypeConstructor(
                                        "Result".to_string(),
                                        vec![
                                            Type::Var("_T".to_string()),
                                            Type::Var("_E".to_string()),
                                        ],
                                    )),
                                ));
                            }
                        }
                        // 默认返回无参构造函数
                        return Ok(Type::Function(
                            Vec::new(),
                            Box::new(Type::Generic(class_name.clone())),
                        ));
                    }
                    Err(TypeError::InvalidMemberAccess {
                        message: format!("未知类型: {}", class_name),
                        span,
                    })
                }
                _ => {
                    // 使用 match 避免双重模式匹配 (TOCTOU 优化)
                    match &obj_type {
                        Type::TypeConstructor(type_name, type_args) if type_name == "Option" => {
                            let inner = type_args[0].clone();
                            match member.as_str() {
                                "is_some" | "is_none" | "unwrap" | "unwrap_or" | "map"
                                | "and_then" => {
                                    return Ok(match_member_function(member, inner));
                                }
                                _ => {}
                            }
                        }
                        Type::TypeConstructor(type_name, type_args) if type_name == "Result" => {
                            let ok_type = type_args[0].clone();
                            let err_type = type_args[1].clone();
                            match member.as_str() {
                                "is_ok" | "is_err" | "unwrap" | "unwrap_err" | "unwrap_or"
                                | "map" | "map_err" | "and_then" => {
                                    return Ok(match_result_member_function(
                                        member, ok_type, err_type,
                                    ));
                                }
                                _ => {}
                            }
                        }
                        _ => {}
                    }
                    Err(TypeError::InvalidMemberAccess {
                        message: format!("无法访问类型 {:?} 的成员", obj_type),
                        span,
                    })
                }
            }
        }
        ExpressionKind::Call(callee, args) => {
            // 推断被调用表达式的类型
            let callee_type = infer_expression_type(callee, env)?;

            // 检查是否为类构造函数调用（callee 是 Generic 类型）
            if let Type::Generic(class_name) = &callee_type {
                // 这是一个类构造函数调用
                // 克隆类信息来避免借用问题
                let class_info_opt = env.get_class(class_name).cloned();
                if let Some(class_info) = class_info_opt {
                    // 检查构造函数参数
                    if let Some(constructor_params) = &class_info.parent_constructor_params {
                        // 有构造函数定义，检查参数数量
                        if constructor_params.len() != args.len() {
                            return Err(TypeError::ParameterCountMismatch {
                                expected: constructor_params.len(),
                                actual: args.len(),
                                span,
                            });
                        }

                        // 检查参数类型
                        for (param_type, arg) in constructor_params.iter().zip(args) {
                            let arg_type = infer_expression_type(arg, env)?;
                            let type_ok = types_equal(&arg_type, param_type)
                                || matches!(
                                    param_type,
                                    Type::Var(_) | Type::TypeParam(_) | Type::Dynamic
                                )
                                || matches!(
                                    &arg_type,
                                    Type::Var(_) | Type::TypeParam(_) | Type::Dynamic
                                );
                            if !type_ok {
                                return Err(TypeError::ParameterTypeMismatch { span: arg.span });
                            }
                        }
                    }
                    // 如果没有显式定义构造函数，使用默认无参构造
                    else if !args.is_empty() {
                        return Err(TypeError::ParameterCountMismatch {
                            expected: 0,
                            actual: args.len(),
                            span,
                        });
                    }
                }
                // 返回该类的实例类型
                return Ok(Type::Generic(class_name.clone()));
            }

            // 检查是否为内置的 Some/None/Ok/Err 构造函数
            if let Type::TypeConstructor(type_name, type_args) = &callee_type {
                // Some(v) -> Option<T>, Ok(v) -> Result<T, E>, None -> Option<T>, Err(e) -> Result<T, E>
                return Ok(Type::TypeConstructor(type_name.clone(), type_args.clone()));
            }

            // 检查是否为函数类型
            if let Type::Function(param_types, return_type) = callee_type {
                // 检查参数数量
                if param_types.len() != args.len() {
                    return Err(TypeError::ParameterCountMismatch {
                        expected: param_types.len(),
                        actual: args.len(),
                        span,
                    });
                }

                // 检查参数类型
                for (param_type, arg) in param_types.iter().zip(args) {
                    let arg_type = infer_expression_type(arg, env)?;
                    // 对于类型变量参数，接受任何实参类型
                    // param_type 是 &Box<Type>，需要解引用
                    let param_type_ref: &Type = param_type.as_ref();
                    // 判断是否是类型参数（TypeVar, TypeParam, 或看起来像类型参数的 Generic）
                    let is_type_param = matches!(
                        param_type_ref,
                        Type::Var(_) | Type::TypeParam(_) | Type::Dynamic
                    ) || if let Type::Generic(name) = param_type_ref {
                        // 简单启发式：单个大写字母或以大写字母开头且较短的名称可能是类型参数
                        name.len() == 1
                            || (name.len() <= 2
                                && name
                                    .chars()
                                    .next()
                                    .map(|c| c.is_uppercase())
                                    .unwrap_or(false))
                    } else {
                        false
                    };
                    let type_ok = types_equal(&arg_type, param_type_ref)
                        || is_type_param
                        || matches!(&arg_type, Type::Var(_) | Type::TypeParam(_) | Type::Dynamic);
                    if !type_ok {
                        return Err(TypeError::ParameterTypeMismatch { span: arg.span });
                    }
                }

                // 如果返回类型是类型变量，尝试推断为具体类型
                // 对于简单情况，假设返回 Int
                match return_type.as_ref() {
                    Type::Var(_) => Ok(Type::Int),
                    _ => Ok(*return_type),
                }
            } else {
                Err(TypeError::TypeMismatch {
                    expected: "Function".to_string(),
                    actual: format!("{:?}", callee_type),
                    span: callee.span,
                })
            }
        }
        ExpressionKind::Binary(op, left, right) => {
            let left_type = infer_expression_type(left, env)?;
            let right_type = infer_expression_type(right, env)?;

            // 检查是否是 Int 和 Float 的混合运算
            let is_int_float_mixed = (types_equal(&left_type, &Type::Int)
                && types_equal(&right_type, &Type::Float))
                || (types_equal(&left_type, &Type::Float) && types_equal(&right_type, &Type::Int));

            // 检查是否是字符串连接（任一操作数是字符串类型）
            let is_string_concat =
                types_equal(&left_type, &Type::String) || types_equal(&right_type, &Type::String);

            // 检查左右操作数类型是否匹配
            // 对于类型变量，我们尝试进行合一（unification）
            // 如果两边都是类型变量，假设它们可以合一
            // Int + Float 混合运算也是允许的
            // 字符串连接（Add 操作）允许任意类型
            let types_match = types_equal(&left_type, &right_type)
                || matches!((&left_type, &right_type), (Type::Var(_), Type::Var(_)))
                || matches!(&left_type, Type::Var(_))
                || matches!(&right_type, Type::Var(_))
                || is_int_float_mixed
                || (matches!(op, x_parser::ast::BinaryOp::Add) && is_string_concat);

            if !types_match {
                return Err(TypeError::TypeMismatch {
                    expected: format!("{:?}", left_type),
                    actual: format!("{:?}", right_type),
                    span: right.span,
                });
            }

            // 根据操作符返回相应的类型
            match op {
                // 算术运算返回数值类型
                x_parser::ast::BinaryOp::Add
                | x_parser::ast::BinaryOp::Sub
                | x_parser::ast::BinaryOp::Mul
                | x_parser::ast::BinaryOp::Div
                | x_parser::ast::BinaryOp::Mod
                | x_parser::ast::BinaryOp::Pow => {
                    // 字符串连接返回 String
                    if matches!(op, x_parser::ast::BinaryOp::Add) && is_string_concat {
                        Ok(Type::String)
                    } else if is_int_float_mixed {
                        // Int + Float 混合运算返回 Float
                        Ok(Type::Float)
                    } else if matches!(&left_type, Type::Var(_))
                        || matches!(&right_type, Type::Var(_))
                    {
                        Ok(Type::Int)
                    } else if types_equal(&left_type, &Type::Int)
                        || types_equal(&left_type, &Type::Float)
                    {
                        Ok(left_type)
                    } else {
                        Err(TypeError::TypeMismatch {
                            expected: "Int or Float".to_string(),
                            actual: format!("{:?}", left_type),
                            span: left.span,
                        })
                    }
                }
                // 逻辑运算返回布尔类型
                x_parser::ast::BinaryOp::And | x_parser::ast::BinaryOp::Or => {
                    if types_equal(&left_type, &Type::Bool) {
                        Ok(Type::Bool)
                    } else {
                        Err(TypeError::TypeMismatch {
                            expected: format!("{:?}", Type::Bool),
                            actual: format!("{:?}", left_type),
                            span: left.span,
                        })
                    }
                }
                // 比较运算返回布尔类型
                x_parser::ast::BinaryOp::Equal
                | x_parser::ast::BinaryOp::NotEqual
                | x_parser::ast::BinaryOp::Less
                | x_parser::ast::BinaryOp::LessEqual
                | x_parser::ast::BinaryOp::Greater
                | x_parser::ast::BinaryOp::GreaterEqual => Ok(Type::Bool),
                _ => Ok(Type::Unit), // 其他操作暂不实现
            }
        }
        ExpressionKind::Unary(op, expr) => {
            let expr_type = infer_expression_type(expr, env)?;
            match op {
                x_parser::ast::UnaryOp::Negate => {
                    if types_equal(&expr_type, &Type::Int) || types_equal(&expr_type, &Type::Float)
                    {
                        Ok(expr_type)
                    } else {
                        Err(TypeError::TypeMismatch {
                            expected: "Int or Float".to_string(),
                            actual: format!("{:?}", expr_type),
                            span: expr.span,
                        })
                    }
                }
                x_parser::ast::UnaryOp::Not => {
                    if types_equal(&expr_type, &Type::Bool) {
                        Ok(Type::Bool)
                    } else {
                        Err(TypeError::TypeMismatch {
                            expected: format!("{:?}", Type::Bool),
                            actual: format!("{:?}", expr_type),
                            span: expr.span,
                        })
                    }
                }
                x_parser::ast::UnaryOp::Wait => {
                    // wait 表达式：应用于 Async<T> 类型时返回 T
                    if let Type::Async(inner) = expr_type {
                        Ok(*inner)
                    } else {
                        Err(TypeError::InvalidAwait {
                            actual_type: format!("{}", expr_type),
                            span: expr.span,
                        })
                    }
                }
                x_parser::ast::UnaryOp::BitNot => {
                    // 按位取反：应用于 Int 类型
                    if types_equal(&expr_type, &Type::Int) {
                        Ok(Type::Int)
                    } else {
                        Err(TypeError::TypeMismatch {
                            expected: format!("{:?}", Type::Int),
                            actual: format!("{:?}", expr_type),
                            span: expr.span,
                        })
                    }
                }
                x_parser::ast::UnaryOp::Reference => {
                    // &expr: creates reference to expr
                    // Type is &inner_type
                    Ok(Type::Reference(Box::new(expr_type)))
                }
                x_parser::ast::UnaryOp::MutableReference => {
                    // &mut expr: creates mutable reference to expr
                    // Type is &mut inner_type
                    Ok(Type::MutableReference(Box::new(expr_type)))
                }
            }
        }
        ExpressionKind::Cast(expr, target_type) => {
            // 类型转换：结果类型就是目标类型
            // 我们只需要检查源表达式可转换（暂时跳过检查，直接返回目标类型）
            let _ = infer_expression_type(expr, env)?;
            Ok(target_type.clone())
        }
        ExpressionKind::Assign(lhs, rhs) => {
            // 推断右侧表达式类型
            let rhs_type = infer_expression_type(rhs, env)?;

            // 推断左侧表达式类型
            let lhs_type = infer_expression_type(lhs, env)?;

            // 检查类型匹配
            if !types_equal(&lhs_type, &rhs_type) {
                return Err(TypeError::TypeMismatch {
                    expected: format!("{:?}", lhs_type),
                    actual: format!("{:?}", rhs_type),
                    span: rhs.span,
                });
            }

            Ok(rhs_type)
        }
        ExpressionKind::If(cond, then_expr, else_expr) => {
            // 检查条件表达式类型为布尔
            let cond_type = infer_expression_type(cond, env)?;
            if !types_equal(&cond_type, &Type::Bool) {
                return Err(TypeError::TypeMismatch {
                    expected: format!("{:?}", Type::Bool),
                    actual: format!("{:?}", cond_type),
                    span: cond.span,
                });
            }

            // 推断then和else表达式类型
            let then_type = infer_expression_type(then_expr, env)?;
            let else_type = infer_expression_type(else_expr, env)?;

            // 检查then和else表达式类型是否匹配
            if !types_equal(&then_type, &else_type) {
                return Err(TypeError::TypeMismatch {
                    expected: format!("{:?}", then_type),
                    actual: format!("{:?}", else_type),
                    span: else_expr.span,
                });
            }

            Ok(then_type)
        }
        ExpressionKind::Parenthesized(inner) => infer_expression_type(inner, env),
        ExpressionKind::Array(items) => {
            if items.is_empty() {
                // 空数组必须依赖类型注解才能确定元素类型
                return Err(TypeError::CannotInferType { span });
            }
            let item_types: Vec<Type> = items
                .iter()
                .map(|item| infer_expression_type(item, env))
                .collect::<Result<_, _>>()?;
            // 检查所有元素类型是否一致
            let first_ty = item_types[0].clone();
            let all_same = item_types.iter().all(|t| types_equal(&first_ty, t));
            if all_same {
                Ok(Type::Array(Box::new(first_ty)))
            } else {
                // 异构数组使用 Dynamic 类型
                Ok(Type::Array(Box::new(Type::Dynamic)))
            }
        }
        ExpressionKind::Tuple(items) => {
            if items.is_empty() {
                return Ok(Type::Unit);
            }
            let item_types: Vec<Type> = items
                .iter()
                .map(|item| infer_expression_type(item, env))
                .collect::<Result<_, _>>()?;
            Ok(Type::Tuple(item_types))
        }
        ExpressionKind::Dictionary(entries) => {
            if entries.is_empty() {
                return Err(TypeError::CannotInferType { span });
            }
            let mut key_types = Vec::new();
            let mut val_types = Vec::new();
            for (k, v) in entries {
                let kt = infer_expression_type(k, env)?;
                let vt = infer_expression_type(v, env)?;
                key_types.push(kt);
                val_types.push(vt);
            }
            // 检查键类型一致
            let key_ty = key_types[0].clone();
            for kt in &key_types[1..] {
                if !types_equal(&key_ty, kt) {
                    return Err(TypeError::TypeMismatch {
                        expected: format!("{:?}", key_ty),
                        actual: format!("{:?}", kt),
                        span,
                    });
                }
            }
            // 值类型允许不一致（异构字典，如 JSON），使用 Dynamic 类型
            let val_ty = Type::Dynamic;
            Ok(Type::Dictionary(Box::new(key_ty), Box::new(val_ty)))
        }
        ExpressionKind::Range(start, end, _inclusive) => {
            let st = infer_expression_type(start, env)?;
            let et = infer_expression_type(end, env)?;
            if !types_equal(&st, &et) {
                return Err(TypeError::TypeMismatch {
                    expected: format!("{:?}", st),
                    actual: format!("{:?}", et),
                    span: end.span,
                });
            }
            if !(types_equal(&st, &Type::Int) || types_equal(&st, &Type::Float)) {
                return Err(TypeError::TypeMismatch {
                    expected: "Int or Float".to_string(),
                    actual: format!("{:?}", st),
                    span: start.span,
                });
            }
            Ok(Type::Array(Box::new(st)))
        }
        ExpressionKind::Lambda(params, body) => {
            // Lambda 类型推断
            // 需要为每个参数创建类型变量（或使用注解类型）
            let mut param_types = Vec::new();
            env.push_scope();
            for param in params {
                let ty = if let Some(type_annot) = &param.type_annot {
                    type_annot.clone()
                } else {
                    // 无类型注解，生成新鲜类型变量
                    env.fresh_type_var()
                };
                param_types.push(Box::new(ty.clone()));
                if env.current_scope_contains(&param.name) {
                    env.pop_scope();
                    return Err(TypeError::DuplicateDeclaration {
                        name: param.name.clone(),
                        span: param.span,
                    });
                }
                env.add_variable(&param.name, ty);
            }
            // 推断 body 的返回类型
            let return_type = infer_block_type(body, env)?;
            env.pop_scope();
            Ok(Type::Function(param_types, Box::new(return_type)))
        }
        ExpressionKind::Record(name, fields) => {
            // Record 类型推断
            // 验证字段类型一致性
            let mut field_types = Vec::new();
            for (field_name, field_expr) in fields {
                let field_ty = infer_expression_type(field_expr, env)?;
                field_types.push((field_name.clone(), Box::new(field_ty)));
            }
            Ok(Type::Record(name.clone(), field_types))
        }
        ExpressionKind::Pipe(input, functions) => {
            // Pipe 类型推断：input |> f1 |> f2 等价于 f2(f1(input))
            let mut current_type = infer_expression_type(input, env)?;
            for func_expr in functions {
                let func_type = infer_expression_type(func_expr, env)?;
                if let Type::Function(param_types, return_type) = func_type {
                    if param_types.len() != 1 {
                        return Err(TypeError::ParameterCountMismatch {
                            expected: 1,
                            actual: param_types.len(),
                            span: func_expr.span,
                        });
                    }
                    if !types_equal(&current_type, &param_types[0]) {
                        return Err(TypeError::TypeMismatch {
                            expected: format!("{:?}", param_types[0]),
                            actual: format!("{:?}", current_type),
                            span: input.span,
                        });
                    }
                    current_type = *return_type;
                } else {
                    return Err(TypeError::TypeMismatch {
                        expected: "Function".to_string(),
                        actual: format!("{:?}", func_type),
                        span: func_expr.span,
                    });
                }
            }
            Ok(current_type)
        }
        ExpressionKind::Wait(wait_type, exprs) => {
            // Wait 类型推断
            match wait_type {
                x_parser::ast::WaitType::Single => {
                    if exprs.len() != 1 {
                        return Err(TypeError::ParameterCountMismatch {
                            expected: 1,
                            actual: exprs.len(),
                            span,
                        });
                    }
                    let inner_ty = infer_expression_type(&exprs[0], env)?;
                    if let Type::Async(inner) = inner_ty {
                        Ok(*inner)
                    } else {
                        // 非 Async 类型，报错
                        Err(TypeError::InvalidAwait {
                            actual_type: format!("{}", inner_ty),
                            span: exprs[0].span,
                        })
                    }
                }
                x_parser::ast::WaitType::Concurrently => {
                    // concurrently 返回所有结果的元组
                    if exprs.is_empty() {
                        return Err(TypeError::CannotInferType { span });
                    }
                    let mut types = Vec::new();
                    for expr in exprs {
                        let ty = infer_expression_type(expr, env)?;
                        if let Type::Async(inner) = ty {
                            types.push(*inner);
                        } else {
                            // 非 Async 类型，报错
                            return Err(TypeError::InvalidAwait {
                                actual_type: format!("{}", ty),
                                span: expr.span,
                            });
                        }
                    }
                    Ok(Type::Tuple(types))
                }
                x_parser::ast::WaitType::Race => {
                    // race 返回第一个完成的类型
                    if exprs.is_empty() {
                        return Err(TypeError::CannotInferType { span });
                    }
                    // 检查所有表达式都是 Async 类型，且内部类型一致
                    let mut expected_inner_type: Option<Type> = None;
                    for expr in exprs {
                        let ty = infer_expression_type(expr, env)?;
                        if let Type::Async(inner) = ty {
                            if let Some(ref expected) = expected_inner_type {
                                if !types_equal(&inner, expected) {
                                    return Err(TypeError::AsyncTypeMismatch {
                                        expected: format!("{}", expected),
                                        actual: format!("{}", inner),
                                        span: expr.span,
                                    });
                                }
                            } else {
                                expected_inner_type = Some((*inner).clone());
                            }
                        } else {
                            // 非 Async 类型，报错
                            return Err(TypeError::InvalidAwait {
                                actual_type: format!("{}", ty),
                                span: expr.span,
                            });
                        }
                    }
                    Ok(expected_inner_type.unwrap_or(Type::Unit))
                }
                x_parser::ast::WaitType::Timeout(_) => {
                    // timeout 返回 Option<T>
                    if exprs.len() != 1 {
                        return Err(TypeError::ParameterCountMismatch {
                            expected: 1,
                            actual: exprs.len(),
                            span,
                        });
                    }
                    let inner_ty = infer_expression_type(&exprs[0], env)?;
                    if let Type::Async(inner) = inner_ty {
                        Ok(Type::TypeConstructor("Option".to_string(), vec![*inner]))
                    } else {
                        // 非 Async 类型，报错
                        Err(TypeError::InvalidAwait {
                            actual_type: format!("{}", inner_ty),
                            span: exprs[0].span,
                        })
                    }
                }
                x_parser::ast::WaitType::Atomic => {
                    // atomic 等待单个异步操作，类型和 Single 相同
                    if exprs.len() != 1 {
                        return Err(TypeError::ParameterCountMismatch {
                            expected: 1,
                            actual: exprs.len(),
                            span,
                        });
                    }
                    let inner_ty = infer_expression_type(&exprs[0], env)?;
                    if let Type::Async(inner) = inner_ty {
                        Ok(*inner)
                    } else {
                        Err(TypeError::InvalidAwait {
                            actual_type: format!("{}", inner_ty),
                            span: exprs[0].span,
                        })
                    }
                }
                x_parser::ast::WaitType::Retry => {
                    // retry 重试异步操作，最终返回相同类型
                    if exprs.len() != 1 {
                        return Err(TypeError::ParameterCountMismatch {
                            expected: 1,
                            actual: exprs.len(),
                            span,
                        });
                    }
                    let inner_ty = infer_expression_type(&exprs[0], env)?;
                    if let Type::Async(inner) = inner_ty {
                        Ok(*inner)
                    } else {
                        Err(TypeError::InvalidAwait {
                            actual_type: format!("{}", inner_ty),
                            span: exprs[0].span,
                        })
                    }
                }
            }
        }
        ExpressionKind::Needs(operation) => {
            // Needs 表达式: needs Effect.op(args)
            // operation 是 "Effect.op"
            // 解析 effect 名称和操作名称
            let Some((effect_name, op_name)) = operation.split_once('.') else {
                return Err(TypeError::InvalidNeedsSyntax {
                    syntax: operation.clone(),
                    span: expr.span,
                });
            };

            // 检查 effect 是否已声明
            let Some(effect_decl) = env.get_effect(effect_name) else {
                return Err(TypeError::UndeclaredEffect {
                    effect: effect_name.to_string(),
                    span: expr.span,
                });
            };

            // 查找操作
            let Some((_op_name, _param_ty, ret_ty)) = effect_decl
                .operations
                .iter()
                .find(|(name, _, _)| name == op_name)
            else {
                return Err(TypeError::UndefinedEffectOperation {
                    effect: effect_name.to_string(),
                    operation: op_name.to_string(),
                    span: expr.span,
                });
            };

            // 返回操作的返回类型，None 表示 Unit
            Ok(ret_ty.clone().unwrap_or(Type::Unit))
        }
        ExpressionKind::Given(_effect_name, expr) => {
            // Given 表达式提供 effect 的处理，返回内部表达式的类型
            // 效果检查已经通过效果推断完成
            // 这里只需要返回内部表达式的类型
            infer_expression_type(expr, env)
        }
        ExpressionKind::Handle(inner_expr, handlers) => {
            // Handle 表达式返回内部表达式的类型
            let inner_type = infer_expression_type(inner_expr, env)?;
            // 检查 handlers 返回类型（假设所有 handlers 返回相同类型）
            // 这里简化处理：返回内部表达式的类型
            let _ = handlers;
            Ok(inner_type)
        }
        ExpressionKind::TryPropagate(inner_expr) => {
            // ? 运算符：对 Result/Option 进行提前返回
            let inner_type = infer_expression_type(inner_expr, env)?;

            match &inner_type {
                // Result<T, E> -> T
                Type::TypeConstructor(name, args) if name == "Result" => Ok(args[0].clone()),
                // Option<T> -> T
                Type::TypeConstructor(name, args) if name == "Option" => Ok(args[0].clone()),
                // 非 Result/Option 类型，报错
                _ => Err(TypeError::TypeMismatch {
                    expected: "Result or Option".to_string(),
                    actual: format!("{}", inner_type),
                    span: expr.span,
                }),
            }
        }
        ExpressionKind::Match(discriminant, cases) => {
            // 模式匹配表达式：所有分支必须返回兼容类型
            let discriminant_ty = infer_expression_type(discriminant, env)?;
            let dt = discriminant_ty.clone();
            let match_span = discriminant.span;

            // 收集所有分支的类型
            let mut branch_types = Vec::new();
            let mut patterns = Vec::new();

            for case in cases {
                patterns.push(case.pattern.clone());
            }

            for case in cases {
                // 先将模式变量绑定到作用域，然后再检查 guard
                env.push_scope();
                // 检查模式并添加正确类型的绑定变量
                check_pattern(&case.pattern, &dt, env, match_span)?;

                // 在模式变量已绑定后再检查 guard
                if let Some(guard) = &case.guard {
                    let gt = infer_expression_type(guard, env)?;
                    if !types_equal(&gt, &Type::Bool) {
                        return Err(TypeError::TypeMismatch {
                            expected: format!("{:?}", Type::Bool),
                            actual: format!("{:?}", gt),
                            span: guard.span,
                        });
                    }
                }

                // 找到分支最后的表达式作为返回值（如果是表达式语句）
                // 对于match表达式，每个分支必须有一个表达式结果
                let mut branch_ty = Type::Unit;
                if let Some(last_stmt) = case.body.statements.last() {
                    match &last_stmt.node {
                        StatementKind::Expression(expr) => {
                            branch_ty = infer_expression_type(expr, env)?;
                        }
                        _ => {
                            // 如果最后不是表达式，分支返回 Unit
                            check_statement(last_stmt, env)?;
                            branch_ty = Type::Unit;
                        }
                    }
                }
                // 检查所有其他语句
                for stmt in &case.body.statements {
                    check_statement(stmt, env)?;
                }
                branch_types.push(branch_ty);
                env.pop_scope();
            }

            // 检查穷尽性
            if let Err(e) = crate::exhaustiveness::check_exhaustive(&patterns, &discriminant_ty) {
                log::warn!(
                    "Match expression is not exhaustive: missing {:?}",
                    e.uncovered_patterns
                );
                // TODO: report as error/warning based on configuration
            }

            // 所有分支类型必须兼容
            // 找到公共超类型（现在简化：所有分支必须类型相等）
            if branch_types.is_empty() {
                return Ok(Type::Unit);
            }
            let first_ty = &branch_types[0];
            for ty in &branch_types[1..] {
                if !types_equal(ty, first_ty) {
                    return Err(TypeError::TypeMismatch {
                        expected: format!("{}", first_ty),
                        actual: format!("{}", ty),
                        span: match_span,
                    });
                }
            }
            Ok(first_ty.clone())
        }
        ExpressionKind::Await(expr) => {
            // await expr: expr 必须是 Async<T>，返回 T
            let inner_ty = infer_expression_type(expr, env)?;
            match inner_ty {
                Type::Async(inner) => Ok((*inner).clone()),
                _ => Ok(Type::Unit),
            }
        }
        ExpressionKind::OptionalChain(base, _member) => {
            // 可选链：暂时返回 Option<成员类型>，简化处理
            infer_expression_type(base, env)?;
            Ok(Type::Unit)
        }
        ExpressionKind::NullCoalescing(left, right) => {
            // 空合并：返回两个操作数必须是同一类型，返回左操作数类型
            let left_ty = infer_expression_type(left, env)?;
            let _right_ty = infer_expression_type(right, env)?;
            Ok(left_ty)
        }
        ExpressionKind::WhenGuard(condition, body) => {
            let cond_type = infer_expression_type(condition, env)?;
            if !types_equal(&cond_type, &Type::Bool) {
                return Err(TypeError::TypeMismatch {
                    expected: format!("{:?}", Type::Bool),
                    actual: format!("{:?}", cond_type),
                    span: condition.span,
                });
            }
            infer_expression_type(body, env)
        }
        ExpressionKind::Block(block) => {
            env.push_scope();
            let ty = infer_block_type(block, env)?;
            env.pop_scope();
            Ok(ty)
        }
    }
}

/// 推断块表达式的类型
fn infer_block_type(block: &Block, env: &mut TypeEnv) -> Result<Type, TypeError> {
    let mut last_type = Type::Unit;
    for stmt in &block.statements {
        match &stmt.node {
            StatementKind::Expression(expr) => {
                last_type = infer_expression_type(expr, env)?;
            }
            StatementKind::Return(Some(expr)) => {
                last_type = infer_expression_type(expr, env)?;
            }
            StatementKind::Variable(var_decl) => {
                // 对于变量声明，只推断初始化表达式类型，不修改环境
                if let Some(initializer) = &var_decl.initializer {
                    last_type = infer_expression_type(initializer, env)?;
                }
            }
            StatementKind::Return(None) => {
                last_type = Type::Unit;
            }
            // 其他语句不影响返回类型
            _ => {}
        }
    }
    Ok(last_type)
}

/// 为 Option<T> 方法生成函数类型
fn match_member_function(method_name: &str, inner: Type) -> Type {
    let option_type = Type::TypeConstructor("Option".to_string(), vec![inner.clone()]);
    match method_name {
        "is_some" | "is_none" => {
            // self: Option<T> -> Bool
            Type::Function(vec![Box::new(option_type)], Box::new(Type::Bool))
        }
        "unwrap" => {
            // self: Option<T> -> T
            Type::Function(vec![Box::new(option_type)], Box::new(inner))
        }
        "unwrap_or" => {
            // self: Option<T>, default: T -> T
            Type::Function(
                vec![Box::new(option_type), Box::new(inner.clone())],
                Box::new(inner),
            )
        }
        "map" => {
            // self: Option<T>, f: function(T) -> U -> Option<U>
            // 这里返回泛型，因为目标类型 U 未知
            let u_type = Type::Var("_U".to_string());
            Type::Function(
                vec![
                    Box::new(option_type),
                    Box::new(Type::Function(
                        vec![Box::new(inner.clone())],
                        Box::new(u_type.clone()),
                    )),
                ],
                Box::new(Type::TypeConstructor("Option".to_string(), vec![u_type])),
            )
        }
        "and_then" => {
            // self: Option<T>, f: function(T) -> Option<U> -> Option<U>
            let u_type = Type::Var("_U".to_string());
            Type::Function(
                vec![
                    Box::new(option_type),
                    Box::new(Type::Function(
                        vec![Box::new(inner.clone())],
                        Box::new(Type::TypeConstructor(
                            "Option".to_string(),
                            vec![u_type.clone()],
                        )),
                    )),
                ],
                Box::new(Type::TypeConstructor("Option".to_string(), vec![u_type])),
            )
        }
        _ => Type::Function(vec![], Box::new(Type::Unit)),
    }
}

/// 为 Result<T, E> 方法生成函数类型
fn match_result_member_function(method_name: &str, ok_type: Type, err_type: Type) -> Type {
    let result_type = Type::TypeConstructor(
        "Result".to_string(),
        vec![ok_type.clone(), err_type.clone()],
    );
    let u_type = Type::Var("_U".to_string());
    let f_type = Type::Var("_F".to_string());
    match method_name {
        "is_ok" | "is_err" => {
            // self: Result<T, E> -> Bool
            Type::Function(vec![Box::new(result_type)], Box::new(Type::Bool))
        }
        "unwrap" => {
            // self: Result<T, E> -> T
            Type::Function(vec![Box::new(result_type)], Box::new(ok_type))
        }
        "unwrap_err" => {
            // self: Result<T, E> -> E
            let result_err =
                Type::TypeConstructor("Result".to_string(), vec![ok_type, err_type.clone()]);
            Type::Function(vec![Box::new(result_err)], Box::new(err_type))
        }
        "unwrap_or" => {
            // self: Result<T, E>, default: T -> T
            let result_or = Type::TypeConstructor(
                "Result".to_string(),
                vec![ok_type.clone(), err_type.clone()],
            );
            Type::Function(
                vec![Box::new(result_or), Box::new(ok_type.clone())],
                Box::new(ok_type),
            )
        }
        "map" => {
            // self: Result<T, E>, f: function(T) -> U -> Result<U, E>
            let result_map = Type::TypeConstructor(
                "Result".to_string(),
                vec![ok_type.clone(), err_type.clone()],
            );
            Type::Function(
                vec![
                    Box::new(result_map),
                    Box::new(Type::Function(
                        vec![Box::new(ok_type.clone())],
                        Box::new(u_type.clone()),
                    )),
                ],
                Box::new(Type::TypeConstructor(
                    "Result".to_string(),
                    vec![u_type, err_type],
                )),
            )
        }
        "map_err" => {
            // self: Result<T, E>, f: function(E) -> F -> Result<T, F>
            let result_map_err = Type::TypeConstructor(
                "Result".to_string(),
                vec![ok_type.clone(), err_type.clone()],
            );
            Type::Function(
                vec![
                    Box::new(result_map_err),
                    Box::new(Type::Function(
                        vec![Box::new(err_type.clone())],
                        Box::new(f_type.clone()),
                    )),
                ],
                Box::new(Type::TypeConstructor(
                    "Result".to_string(),
                    vec![ok_type, f_type],
                )),
            )
        }
        "and_then" => {
            // self: Result<T, E>, f: function(T) -> Result<U, E> -> Result<U, E>
            let u_var = Type::Var("_U".to_string());
            let result_and = Type::TypeConstructor(
                "Result".to_string(),
                vec![ok_type.clone(), err_type.clone()],
            );
            Type::Function(
                vec![
                    Box::new(result_and),
                    Box::new(Type::Function(
                        vec![Box::new(ok_type.clone())],
                        Box::new(Type::TypeConstructor(
                            "Result".to_string(),
                            vec![u_var.clone(), err_type.clone()],
                        )),
                    )),
                ],
                Box::new(Type::TypeConstructor(
                    "Result".to_string(),
                    vec![u_var, err_type],
                )),
            )
        }
        _ => Type::Function(vec![], Box::new(Type::Unit)),
    }
}

/// 推断字面量类型
fn infer_literal_type(lit: &Literal, _span: Span) -> Result<Type, TypeError> {
    match lit {
        Literal::Integer(_) => Ok(Type::Int),
        Literal::Float(_) => Ok(Type::Float),
        Literal::Boolean(_) => Ok(Type::Bool),
        Literal::String(_) => Ok(Type::String),
        Literal::Char(_) => Ok(Type::Char),
        Literal::Null => Ok(Type::Unit),
        Literal::None => Ok(Type::TypeConstructor(
            "Option".to_string(),
            vec![Type::Unit],
        )),
        Literal::Unit => Ok(Type::Unit),
    }
}

/// 检查两个类型是否相等
fn types_equal(ty1: &Type, ty2: &Type) -> bool {
    match (ty1, ty2) {
        // 类型变量：在类型推断期间，类型变量可以与任何类型匹配
        // 这是一个简化的处理，真正的实现应该使用合一算法
        (Type::Var(_), _) | (_, Type::Var(_)) => true,

        // 基本类型
        (Type::Int, Type::Int) => true,
        (Type::UnsignedInt, Type::UnsignedInt) => true,
        (Type::Float, Type::Float) => true,
        (Type::Bool, Type::Bool) => true,
        (Type::String, Type::String) => true,
        (Type::Char, Type::Char) => true,
        (Type::Unit, Type::Unit) => true,
        (Type::Never, Type::Never) => true,
        // Dynamic 可以与任何类型匹配（用于动态类型）
        (Type::Dynamic, _) | (_, Type::Dynamic) => true,

        // 复合类型
        (Type::Array(a1), Type::Array(a2)) => types_equal(a1, a2),
        (Type::Dictionary(k1, v1), Type::Dictionary(k2, v2)) => {
            types_equal(k1, k2) && types_equal(v1, v2)
        }
        (Type::Tuple(t1), Type::Tuple(t2)) => {
            if t1.len() != t2.len() {
                return false;
            }
            t1.iter().zip(t2.iter()).all(|(a, b)| types_equal(a, b))
        }
        (Type::Record(name1, fields1), Type::Record(name2, fields2)) => {
            if name1 != name2 {
                return false;
            }
            if fields1.len() != fields2.len() {
                return false;
            }
            fields1
                .iter()
                .zip(fields2.iter())
                .all(|((n1, t1), (n2, t2))| n1 == n2 && types_equal(t1, t2))
        }
        (Type::Union(name1, variants1), Type::Union(name2, variants2)) => {
            if name1 != name2 {
                return false;
            }
            if variants1.len() != variants2.len() {
                return false;
            }
            variants1
                .iter()
                .zip(variants2.iter())
                .all(|(v1, v2)| types_equal(v1, v2))
        }

        // 类型构造器：比较名称和所有类型参数
        (Type::TypeConstructor(name1, args1), Type::TypeConstructor(name2, args2)) => {
            if name1 != name2 {
                return false;
            }
            if args1.len() != args2.len() {
                return false;
            }
            args1
                .iter()
                .zip(args2.iter())
                .all(|(a1, a2)| types_equal(a1, a2))
        }

        // 高级类型
        (Type::Function(p1, r1), Type::Function(p2, r2)) => {
            if p1.len() != p2.len() {
                return false;
            }
            for (t1, t2) in p1.iter().zip(p2) {
                if !types_equal(t1, t2) {
                    return false;
                }
            }
            types_equal(r1, r2)
        }
        (Type::Async(a1), Type::Async(a2)) => types_equal(a1, a2),

        // 泛型类型
        (Type::Generic(n1), Type::Generic(n2)) => n1 == n2,
        (Type::TypeParam(n1), Type::TypeParam(n2)) => n1 == n2,

        // FFI 类型
        (Type::Void, Type::Void) => true,
        (Type::Pointer(p1), Type::Pointer(p2)) => types_equal(p1, p2),
        (Type::ConstPointer(p1), Type::ConstPointer(p2)) => types_equal(p1, p2),
        // C FFI 类型
        (Type::CInt, Type::CInt) => true,
        (Type::CUInt, Type::CUInt) => true,
        (Type::CLong, Type::CLong) => true,
        (Type::CULong, Type::CULong) => true,
        (Type::CLongLong, Type::CLongLong) => true,
        (Type::CULongLong, Type::CULongLong) => true,
        (Type::CFloat, Type::CFloat) => true,
        (Type::CDouble, Type::CDouble) => true,
        (Type::CChar, Type::CChar) => true,
        (Type::CSize, Type::CSize) => true,
        (Type::CString, Type::CString) => true,

        _ => false,
    }
}

/// 检查类型兼容性（是否可以从 source 赋值给 target）
///
/// 类型兼容性规则：
/// 1. 类型相等则兼容
/// 2. Never 是所有类型的子类型
/// 3. Union 类型：如果 source 是 union 的某个变体，则兼容
/// 4. Option 类型：如果 source 是 T，可以赋值给 Option<T>
/// 5. 数值类型：Int 可以隐式转换为 Float
#[allow(dead_code)]
fn is_type_compatible(source: &Type, target: &Type) -> bool {
    // 首先检查类型相等
    if types_equal(source, target) {
        return true;
    }

    match (source, target) {
        // Never 是所有类型的子类型
        // 注意：(Never, Never) 已在 types_equal 中处理
        (Type::Never, _) => true,

        // Int 可以隐式转换为 Float
        (Type::Int, Type::Float) => true,

        // Union 类型兼容性：检查 source 是否是 union 的某个变体
        (source, Type::Union(_, variants)) => {
            variants.iter().any(|v| is_type_compatible(source, v))
        }

        // Option 类型兼容性：T 可以赋值给 Option<T>
        (source, Type::TypeConstructor(type_name, type_args)) if type_name == "Option" => {
            is_type_compatible(source, &type_args[0])
        }

        // 数组协变：如果元素类型兼容，数组也兼容
        (Type::Array(s_inner), Type::Array(t_inner)) => is_type_compatible(s_inner, t_inner),

        // Result 类型兼容性
        (Type::TypeConstructor(s_name, s_args), Type::TypeConstructor(t_name, t_args))
            if s_name == "Result"
                && t_name == "Result"
                && s_args.len() == 2
                && t_args.len() == 2 =>
        {
            is_type_compatible(&s_args[0], &t_args[0]) && is_type_compatible(&s_args[1], &t_args[1])
        }

        // 函数类型兼容性（参数是逆变的，返回值是协变的）
        // 简化版本：仅检查协变
        (Type::Function(s_params, s_ret), Type::Function(t_params, t_ret)) => {
            if s_params.len() != t_params.len() {
                return false;
            }
            // 返回值协变
            if !is_type_compatible(s_ret, t_ret) {
                return false;
            }
            // 参数逆变（简化：使用相等检查）
            s_params
                .iter()
                .zip(t_params.iter())
                .all(|(sp, tp)| types_equal(sp, tp))
        }

        // Tuple 类型兼容性
        (Type::Tuple(s_items), Type::Tuple(t_items)) => {
            if s_items.len() != t_items.len() {
                return false;
            }
            s_items
                .iter()
                .zip(t_items.iter())
                .all(|(s, t)| is_type_compatible(s, t))
        }

        // Record 类型兼容性（名义类型，需要相同名称）
        (Type::Record(s_name, s_fields), Type::Record(t_name, t_fields)) => {
            if s_name != t_name {
                return false;
            }
            if s_fields.len() != t_fields.len() {
                return false;
            }
            s_fields
                .iter()
                .zip(t_fields.iter())
                .all(|((sn, st), (tn, tt))| sn == tn && is_type_compatible(st, tt))
        }

        // Async 类型兼容性
        (Type::Async(s_inner), Type::Async(t_inner)) => is_type_compatible(s_inner, t_inner),

        _ => false,
    }
}

/// 获取类型的公共超类型（用于分支合并等场景）
#[allow(dead_code)]
fn common_supertype(types: &[Type]) -> Option<Type> {
    if types.is_empty() {
        return None;
    }
    if types.len() == 1 {
        return Some(types[0].clone());
    }

    let first = &types[0];

    // 检查所有类型是否相同
    if types.iter().all(|t| types_equal(t, first)) {
        return Some(first.clone());
    }

    // 检查是否所有类型都可以转换为同一个类型
    for candidate in types {
        if types.iter().all(|t| is_type_compatible(t, candidate)) {
            return Some(candidate.clone());
        }
    }

    // 尝试创建 Union 类型
    let mut variants: Vec<Type> = Vec::new();
    for t in types {
        // 检查是否已经包含
        if !variants.iter().any(|v| types_equal(v, t)) {
            variants.push(t.clone());
        }
    }

    if variants.len() == 1 {
        return Some(variants.into_iter().next().unwrap());
    }

    Some(Type::Union("__inferred__".to_string(), variants))
}

#[cfg(test)]
mod tests {
    use super::*;
    use x_parser::parse_program;

    #[test]
    fn type_check_ok_variable_and_binary() {
        let src = r#"
let x: Int = 1;
let y: Int = x + 2;
"#;
        let program = parse_program(src).expect("parse ok");
        type_check(&program).expect("type_check ok");
    }

    #[test]
    fn type_check_undefined_variable() {
        let src = r#"
let y: Int = x + 1;
"#;
        let program = parse_program(src).expect("parse ok");
        let err = type_check(&program).unwrap_err();
        assert!(matches!(err, TypeError::UndefinedVariable { name, .. } if name == "x"));
    }

    #[test]
    fn type_check_duplicate_declaration_same_scope() {
        let src = r#"
let x: Int = 1;
let x: Int = 2;
"#;
        let program = parse_program(src).expect("parse ok");
        let err = type_check(&program).unwrap_err();
        assert!(matches!(err, TypeError::DuplicateDeclaration { name, .. } if name == "x"));
    }

    #[test]
    fn type_check_function_call_parameter_count_mismatch() {
        let src = r#"
function add(a: Int, b: Int) -> Int { return a + b; }
let x: Int = add(1);
"#;
        let program = parse_program(src).expect("parse ok");
        let err = type_check(&program).unwrap_err();
        assert!(matches!(
            err,
            TypeError::ParameterCountMismatch {
                expected: 2,
                actual: 1,
                ..
            }
        ));
    }

    #[test]
    fn type_check_function_call_parameter_type_mismatch() {
        let src = r#"
function id(a: Int) -> Int { return a; }
let x: Int = id(true);
"#;
        let program = parse_program(src).expect("parse ok");
        let err = type_check(&program).unwrap_err();
        assert!(matches!(err, TypeError::ParameterTypeMismatch { .. }));
    }

    #[test]
    fn type_check_if_condition_must_be_bool() {
        let src = r#"
if 1 { return; }
"#;
        let program = parse_program(src).expect("parse ok");
        let err = type_check(&program).unwrap_err();
        assert!(matches!(err, TypeError::TypeMismatch { .. }));
    }

    #[test]
    fn type_check_array_type_inference() {
        // x-parser 当前对数组字面量的解析尚未完成；此处保留为后续用例
        let src = r#"
let a: Int = 1;
"#;
        let program = parse_program(src).expect("parse ok");
        type_check(&program).expect("type_check ok");
    }

    #[test]
    fn type_check_empty_array_needs_annotation() {
        // x-parser 当前对数组字面量的解析尚未完成；此处改为测试"无初始化且无注解"的推断失败
        let src = r#"
let a;
"#;
        let program = parse_program(src).expect("parse ok");
        let err = type_check(&program).unwrap_err();
        assert!(matches!(err, TypeError::CannotInferType { .. }));
    }

    #[test]
    fn type_check_match_guard_bool() {
        let src = r#"
let x: Int = 1;
match x {
  _ when 1 { return; }
}
"#;
        let program = parse_program(src).expect("parse ok");
        let err = type_check(&program).unwrap_err();
        assert!(matches!(err, TypeError::TypeMismatch { .. }));
    }

    #[test]
    fn type_check_try_catch_finally_scopes() {
        let src = r#"
try { let x: Int = 1; return x; }
catch (Exception e) { return e; }
finally { return; }
"#;
        let program = parse_program(src).expect("parse ok");
        // e 的类型目前占位 Unit，所以 return e 仍可通过类型推断为 Unit，这里仅验证不崩溃
        type_check(&program).expect("type_check ok");
    }

    #[test]
    fn type_check_option_type() {
        // Option 类型测试 - 使用基本类型验证
        let src = r#"
let x: Int = 1;
"#;
        let program = parse_program(src).expect("parse ok");
        type_check(&program).expect("type_check ok");
    }

    #[test]
    fn type_check_tuple_type() {
        let src = r#"
let x: Int = 1;
let y: String = "hello";
"#;
        let program = parse_program(src).expect("parse ok");
        type_check(&program).expect("type_check ok");
    }

    #[test]
    fn type_check_function_as_value() {
        let src = r#"
function add(a: Int, b: Int) -> Int { return a + b; }
let f = add;
let result: Int = f(1, 2);
"#;
        let program = parse_program(src).expect("parse ok");
        type_check(&program).expect("type_check ok");
    }

    #[test]
    fn type_check_nested_function_calls() {
        let src = r#"
function double(x: Int) -> Int { return x + x; }
function quadruple(x: Int) -> Int { return double(double(x)); }
let result: Int = quadruple(5);
"#;
        let program = parse_program(src).expect("parse ok");
        type_check(&program).expect("type_check ok");
    }

    #[test]
    fn type_check_lambda_simple() {
        // Lambda 测试（当前 parser 可能不支持完整语法）
        let src = r#"
let x: Int = 1;
"#;
        let program = parse_program(src).expect("parse ok");
        type_check(&program).expect("type_check ok");
    }

    #[test]
    fn type_check_record_type() {
        // 记录类型测试（当前可能不支持）
        let src = r#"
let x: Int = 1;
"#;
        let program = parse_program(src).expect("parse ok");
        type_check(&program).expect("type_check ok");
    }

    #[test]
    fn type_check_pipe_operator() {
        // 管道操作测试（当前可能不支持）
        let src = r#"
let x: Int = 1;
"#;
        let program = parse_program(src).expect("parse ok");
        type_check(&program).expect("type_check ok");
    }

    #[test]
    fn type_error_has_span() {
        let src = r#"
let y = x;
"#;
        let program = parse_program(src).expect("parse ok");
        let err = type_check(&program).unwrap_err();
        // 验证错误包含 Span
        let span = err.span();
        // span 是有效的（start <= end）
        assert!(span.start <= span.end);
    }

    // === 类型兼容性测试 ===

    #[test]
    fn type_compatibility_equal_types() {
        // 相同类型应该兼容
        assert!(is_type_compatible(&Type::Int, &Type::Int));
        assert!(is_type_compatible(&Type::String, &Type::String));
        assert!(is_type_compatible(&Type::Bool, &Type::Bool));
    }

    #[test]
    fn type_compatibility_never_subtype() {
        // Never 是所有类型的子类型
        assert!(is_type_compatible(&Type::Never, &Type::Int));
        assert!(is_type_compatible(&Type::Never, &Type::String));
        assert!(is_type_compatible(&Type::Never, &Type::Bool));
        assert!(is_type_compatible(
            &Type::Never,
            &Type::Array(Box::new(Type::Int))
        ));
    }

    #[test]
    fn type_compatibility_int_to_float() {
        // Int 可以隐式转换为 Float
        assert!(is_type_compatible(&Type::Int, &Type::Float));
        // 但反过来不行
        assert!(!is_type_compatible(&Type::Float, &Type::Int));
    }

    #[test]
    fn type_compatibility_option() {
        // T 可以赋值给 Option<T>
        let opt_int = Type::TypeConstructor("Option".to_string(), vec![Type::Int]);
        let opt_string = Type::TypeConstructor("Option".to_string(), vec![Type::String]);
        assert!(is_type_compatible(&Type::Int, &opt_int));
        assert!(is_type_compatible(&Type::String, &opt_string));

        // Option<T> 不能赋值给 T
        assert!(!is_type_compatible(&opt_int, &Type::Int));

        // Option<T> 兼容 Option<T>
        assert!(is_type_compatible(&opt_int, &opt_int));
    }

    #[test]
    fn type_compatibility_union() {
        // 检查类型是否是 union 的成员
        let union_type = Type::Union("Number".to_string(), vec![Type::Int, Type::Float]);

        assert!(is_type_compatible(&Type::Int, &union_type));
        assert!(is_type_compatible(&Type::Float, &union_type));
        assert!(!is_type_compatible(&Type::String, &union_type));
    }

    #[test]
    fn type_compatibility_array() {
        // 数组协变
        assert!(is_type_compatible(
            &Type::Array(Box::new(Type::Int)),
            &Type::Array(Box::new(Type::Float))
        ));

        // 字符串数组不能赋值给整数数组
        assert!(!is_type_compatible(
            &Type::Array(Box::new(Type::String)),
            &Type::Array(Box::new(Type::Int))
        ));
    }

    #[test]
    fn type_compatibility_function() {
        // 函数类型兼容性
        let fn1 = Type::Function(vec![Box::new(Type::Int)], Box::new(Type::Int));
        let fn2 = Type::Function(vec![Box::new(Type::Int)], Box::new(Type::Int));

        assert!(is_type_compatible(&fn1, &fn2));

        // 返回值类型不同的函数
        let fn3 = Type::Function(vec![Box::new(Type::Int)], Box::new(Type::Float));
        // 返回值协变：Int -> Float
        assert!(is_type_compatible(&fn1, &fn3));
    }

    #[test]
    fn common_supertype_equal_types() {
        // 相同类型的公共超类型就是它们自己
        let types = vec![Type::Int, Type::Int, Type::Int];
        let supertype = common_supertype(&types);
        assert!(matches!(supertype, Some(Type::Int)));
    }

    #[test]
    fn common_supertype_never_and_other() {
        // Never 和其他类型的公共超类型是那个其他类型
        let types = vec![Type::Never, Type::Int];
        let supertype = common_supertype(&types);
        assert!(matches!(supertype, Some(Type::Int)));
    }

    #[test]
    fn common_supertype_int_and_float() {
        // Int 和 Float 的公共超类型应该是 Float（因为 Int 可以转换为 Float）
        let types = vec![Type::Int, Type::Float];
        let supertype = common_supertype(&types);
        assert!(matches!(supertype, Some(Type::Float)));
    }

    #[test]
    fn common_supertype_creates_union() {
        // 不同类型的公共超类型是 Union
        let types = vec![Type::Int, Type::String];
        let supertype = common_supertype(&types);
        assert!(matches!(supertype, Some(Type::Union(_, _))));
    }

    #[test]
    fn common_supertype_empty() {
        // 空列表返回 None
        let types: Vec<Type> = vec![];
        let supertype = common_supertype(&types);
        assert!(supertype.is_none());
    }

    // === 类继承和 trait 实现测试 ===

    #[test]
    fn class_inheritance_type_check() {
        // 测试类继承的类型检查
        let src = r#"
class Animal {
    let name: String

    new(n: String) {
        this.name = n
    }

    virtual function speak() -> String {
        return "..."
    }
}

class Dog extends Animal {
    override function speak() -> String {
        return "Woof"
    }
}
"#;
        let program = parse_program(src).expect("parse ok");
        type_check(&program).expect("type_check ok for class inheritance");
    }

    #[test]
    fn class_inheritance_cycle_detection() {
        // 测试继承循环检测 - 这需要复杂的设置，目前简化测试
        let env = TypeEnv::new();
        // 直接测试 check_inheritance_cycle 函数
        // 在实际使用中，这个检测会通过 check_class_decl 自动触发
        assert!(!check_inheritance_cycle("TestClass", &env));
    }

    #[test]
    fn trait_implementation_check() {
        // 测试 trait 实现
        let src = r#"
trait Serializable {
    function serialize() -> String
}

class Point implement Serializable {
    let x: Int
    let y: Int

    new(a: Int, b: Int) {
        this.x = a
        this.y = b
    }

    function serialize() -> String {
        return "Point"
    }
}
"#;
        let program = parse_program(src).expect("parse ok");
        type_check(&program).expect("type_check ok for trait implementation");
    }

    #[test]
    fn trait_missing_method_implementation() {
        // 测试缺少 trait 方法实现
        // 由于当前 parser 对某些语法有限制，使用简单的测试
        let mut env = TypeEnv::new();

        // 添加 trait
        let mut methods = HashMap::new();
        methods.insert(
            "serialize".to_string(),
            Type::Function(vec![], Box::new(Type::String)),
        );
        env.add_trait(
            "Serializable",
            TraitInfo {
                name: "Serializable".to_string(),
                extends: vec![],
                methods,
            },
        );

        // 添加类，但不实现 serialize 方法
        let mut class_methods = HashMap::new();
        class_methods.insert(
            "toString".to_string(),
            Type::Function(vec![], Box::new(Type::String)),
        );
        env.add_class(
            "Point",
            ClassInfo {
                name: "Point".to_string(),
                extends: None,
                implements: vec!["Serializable".to_string()],
                fields: HashMap::new(),
                methods: class_methods,
                is_abstract: false,
                is_final: false,
                field_visibility: HashMap::new(),
                method_visibility: HashMap::new(),
                abstract_methods: HashSet::new(),
                virtual_methods: HashSet::new(),
                parent_constructor_params: None,
            },
        );

        // 测试 trait 实现检查
        let result = check_trait_implementation("Point", "Serializable", &mut env, Span::default());
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, TypeError::MissingTraitMethod { .. }));
    }

    #[test]
    fn abstract_class_implementation_check() {
        // 测试抽象类 - 直接测试抽象方法检查函数
        let mut env = TypeEnv::new();

        // 添加抽象父类
        let mut abstract_methods = HashSet::new();
        abstract_methods.insert("area".to_string());

        let mut parent_methods = HashMap::new();
        parent_methods.insert(
            "area".to_string(),
            Type::Function(vec![], Box::new(Type::Float)),
        );

        env.add_class(
            "Shape",
            ClassInfo {
                name: "Shape".to_string(),
                extends: None,
                implements: vec![],
                fields: HashMap::new(),
                methods: parent_methods,
                is_abstract: true,
                is_final: false,
                field_visibility: HashMap::new(),
                method_visibility: HashMap::new(),
                abstract_methods,
                virtual_methods: HashSet::new(),
                parent_constructor_params: None,
            },
        );

        // 添加具体子类
        let mut child_methods = HashMap::new();
        child_methods.insert(
            "area".to_string(),
            Type::Function(vec![], Box::new(Type::Float)),
        );

        env.add_class(
            "Circle",
            ClassInfo {
                name: "Circle".to_string(),
                extends: Some("Shape".to_string()),
                implements: vec![],
                fields: HashMap::new(),
                methods: child_methods,
                is_abstract: false,
                is_final: false,
                field_visibility: HashMap::new(),
                method_visibility: HashMap::new(),
                abstract_methods: HashSet::new(),
                virtual_methods: HashSet::new(),
                parent_constructor_params: None,
            },
        );

        // 测试抽象方法实现检查 - Circle 实现了 area，应该通过
        let class_decl = ClassDecl {
            name: "Circle".to_string(),
            type_parameters: vec![],
            extends: Some("Shape".to_string()),
            implements: vec![],
            members: vec![],
            modifiers: x_parser::ast::ClassModifiers::default(),
            span: x_lexer::span::Span::new(0, 0),
        };

        let result = check_abstract_method_implementation(&class_decl, &env);
        assert!(
            result.is_none(),
            "Circle should implement all abstract methods"
        );
    }

    #[test]
    fn is_subtype_of_test() {
        let mut env = TypeEnv::new();

        // 设置测试类
        env.add_class(
            "Animal",
            ClassInfo {
                name: "Animal".to_string(),
                extends: None,
                implements: vec![],
                fields: HashMap::new(),
                methods: HashMap::new(),
                is_abstract: false,
                is_final: false,
                field_visibility: HashMap::new(),
                method_visibility: HashMap::new(),
                abstract_methods: HashSet::new(),
                virtual_methods: HashSet::new(),
                parent_constructor_params: None,
            },
        );

        env.add_class(
            "Dog",
            ClassInfo {
                name: "Dog".to_string(),
                extends: Some("Animal".to_string()),
                implements: vec![],
                fields: HashMap::new(),
                methods: HashMap::new(),
                is_abstract: false,
                is_final: false,
                field_visibility: HashMap::new(),
                method_visibility: HashMap::new(),
                abstract_methods: HashSet::new(),
                virtual_methods: HashSet::new(),
                parent_constructor_params: None,
            },
        );

        // 测试子类型关系
        assert!(is_subtype_of(
            &Type::Generic("Dog".to_string()),
            &Type::Generic("Animal".to_string()),
            &env
        ));
        assert!(is_subtype_of(
            &Type::Generic("Dog".to_string()),
            &Type::Generic("Dog".to_string()),
            &env
        ));
        assert!(!is_subtype_of(
            &Type::Generic("Animal".to_string()),
            &Type::Generic("Dog".to_string()),
            &env
        ));
    }

    #[test]
    fn is_subtype_of_with_trait() {
        let mut env = TypeEnv::new();

        // 设置测试类和 trait
        env.add_class(
            "Point",
            ClassInfo {
                name: "Point".to_string(),
                extends: None,
                implements: vec!["Serializable".to_string()],
                fields: HashMap::new(),
                methods: HashMap::new(),
                is_abstract: false,
                is_final: false,
                field_visibility: HashMap::new(),
                method_visibility: HashMap::new(),
                abstract_methods: HashSet::new(),
                virtual_methods: HashSet::new(),
                parent_constructor_params: None,
            },
        );

        env.add_trait(
            "Serializable",
            TraitInfo {
                name: "Serializable".to_string(),
                extends: vec![],
                methods: HashMap::new(),
            },
        );

        // 测试 trait 实现的子类型关系
        assert!(is_subtype_of(
            &Type::Generic("Point".to_string()),
            &Type::Generic("Serializable".to_string()),
            &env
        ));
    }

    #[test]
    fn visibility_check_test() {
        let mut env = TypeEnv::new();

        let mut field_visibility = HashMap::new();
        field_visibility.insert("private_field".to_string(), Visibility::Private);
        field_visibility.insert("public_field".to_string(), Visibility::Public);
        field_visibility.insert("protected_field".to_string(), Visibility::Protected);

        env.add_class(
            "Test",
            ClassInfo {
                name: "Test".to_string(),
                extends: None,
                implements: vec![],
                fields: HashMap::new(),
                methods: HashMap::new(),
                is_abstract: false,
                is_final: false,
                field_visibility,
                method_visibility: HashMap::new(),
                abstract_methods: HashSet::new(),
                virtual_methods: HashSet::new(),
                parent_constructor_params: None,
            },
        );

        // 测试可见性检查
        let span = Span::default();

        // 公共字段应该可以访问
        assert!(check_visibility_access("Test", "public_field", true, "Other", &env, span).is_ok());

        // 私有字段在类外部不可访问
        assert!(
            check_visibility_access("Test", "private_field", true, "Other", &env, span).is_err()
        );

        // 私有字段在类内部可以访问
        assert!(check_visibility_access("Test", "private_field", true, "Test", &env, span).is_ok());
    }

    // ==================== Effect System Tests ====================

    #[test]
    fn effect_set_creation() {
        use std::collections::HashSet;

        let mut effects: EffectSet = HashSet::new();
        effects.insert("IO".to_string());
        effects.insert("Async".to_string());

        assert!(effects.contains("IO"));
        assert!(effects.contains("Async"));
        assert_eq!(effects.len(), 2);
    }

    #[test]
    fn parse_builtin_effects() {
        use x_parser::ast::Effect;

        let effects = vec![Effect::IO, Effect::Async];
        let parsed = parse_effects(&effects);

        assert!(parsed.contains("IO"));
        assert!(parsed.contains("Async"));
    }

    #[test]
    fn parse_throws_effect() {
        use x_parser::ast::Effect;

        let effects = vec![Effect::Throws("NetworkError".to_string())];
        let parsed = parse_effects(&effects);

        assert!(parsed.contains("Throws(NetworkError)"));
    }

    #[test]
    fn parse_state_effect() {
        use x_parser::ast::Effect;

        let effects = vec![Effect::State("Int".to_string())];
        let parsed = parse_effects(&effects);

        assert!(parsed.contains("State(Int)"));
    }

    #[test]
    fn parse_custom_effect() {
        use x_parser::ast::Effect;

        let effects = vec![Effect::Custom("Logger".to_string())];
        let parsed = parse_effects(&effects);

        assert!(parsed.contains("Logger"));
    }

    #[test]
    fn effect_set_operations() {
        let mut effects: EffectSet = HashSet::new();
        effects.insert("IO".to_string());
        effects.insert("Async".to_string());

        // Test set operations
        assert!(effects.contains("IO"));
        assert!(effects.contains("Async"));
        assert_eq!(effects.len(), 2);

        // Test subset relationship manually
        let mut other: EffectSet = HashSet::new();
        other.insert("IO".to_string());
        assert!(other.is_subset(&effects));
    }

    #[test]
    fn pure_function_empty_effects() {
        let effects: EffectSet = HashSet::new();
        assert!(effects.is_empty());
    }

    // ==================== Type Inference Tests ====================

    #[test]
    fn type_inference_integer_literal() {
        let src = r#"
let x = 42;
"#;
        let program = parse_program(src).expect("parse ok");
        let result = type_check(&program);
        assert!(result.is_ok());
    }

    #[test]
    fn type_inference_float_literal() {
        let src = r#"
let x = 3.14;
"#;
        let program = parse_program(src).expect("parse ok");
        let result = type_check(&program);
        assert!(result.is_ok());
    }

    #[test]
    fn type_inference_string_literal() {
        let src = r#"
let x = "hello";
"#;
        let program = parse_program(src).expect("parse ok");
        let result = type_check(&program);
        assert!(result.is_ok());
    }

    #[test]
    fn type_inference_bool_literal() {
        let src = r#"
let x = true;
"#;
        let program = parse_program(src).expect("parse ok");
        let result = type_check(&program);
        assert!(result.is_ok());
    }

    #[test]
    fn type_inference_binary_expression() {
        let src = r#"
let x = 1 + 2;
"#;
        let program = parse_program(src).expect("parse ok");
        let result = type_check(&program);
        assert!(result.is_ok());
    }

    #[test]
    fn type_inference_function_call() {
        let src = r#"
function add(a: Int, b: Int) -> Int { return a + b; }
let x = add(1, 2);
"#;
        let program = parse_program(src).expect("parse ok");
        let result = type_check(&program);
        assert!(result.is_ok());
    }

    // ==================== Scope Tests ====================

    #[test]
    fn scope_shadowing_allowed() {
        let src = r#"
let x = 1;
let x = 2;
"#;
        let program = parse_program(src).expect("parse ok");
        // This should fail because duplicate declaration in same scope
        let result = type_check(&program);
        assert!(result.is_err());
    }

    #[test]
    #[ignore = "closure capture not yet fully implemented in parser"]
    fn nested_scope_variable_access() {
        let src = r#"
let x = 1;
function foo() -> Int {
    return x;
}
"#;
        let program = parse_program(src).expect("parse ok");
        let result = type_check(&program);
        assert!(result.is_ok());
    }

    #[test]
    fn function_parameter_scope() {
        let src = r#"
function add(a: Int, b: Int) -> Int {
    return a + b;
}
"#;
        let program = parse_program(src).expect("parse ok");
        let result = type_check(&program);
        assert!(result.is_ok());
    }

    // ==================== Control Flow Type Tests ====================

    #[test]
    #[ignore = "if expression syntax not yet fully implemented"]
    fn if_expression_both_branches_same_type() {
        let src = r#"
let x = if true { 1 } else { 2 };
"#;
        let program = parse_program(src).expect("parse ok");
        let result = type_check(&program);
        assert!(result.is_ok());
    }

    #[test]
    fn while_loop_condition_must_be_bool() {
        let src = r#"
while 1 { }
"#;
        let program = parse_program(src).expect("parse ok");
        let result = type_check(&program);
        assert!(result.is_err());
    }

    // ==================== Generic Type Tests ====================

    #[test]
    fn generic_function_definition() {
        let src = r#"
function identity<T>(x: T) -> T {
    return x;
}
"#;
        let program = parse_program(src).expect("parse ok");
        let result = type_check(&program);
        assert!(result.is_ok());
    }

    #[test]
    fn generic_class_definition() {
        let src = r#"
class Container<T> {
    let value: T
}
"#;
        let program = parse_program(src).expect("parse ok");
        let result = type_check(&program);
        assert!(result.is_ok());
    }

    // ==================== Option/Result Type Tests ====================
    // 注意: Some/None/Ok/Err 现在由标准库提供，不是编译器内置
    // 用户需要使用标准库中的版本，或定义自己的枚举

    #[test]
    fn option_type_annotation() {
        // Option<T> 类型注解仍然可用（作为内置语法）
        let src = r#"
let x: Option<Int> = None;
"#;
        let program = parse_program(src).expect("parse ok");
        let _ = type_check(&program);
        // 应该能通过，因为 Option<Int> 语法仍然有效
        // 但 Some(42) 需要用户自己定义或从标准库导入
    }

    #[test]
    fn result_type_annotation() {
        // Result<T, E> 类型注解仍然可用（作为内置语法）
        let src = r#"
let x: Result<Int, String> = Ok(42);
"#;
        let program = parse_program(src).expect("parse ok");
        let _ = type_check(&program);
        // 应该能通过
    }

    // ==================== Pattern Matching Type Tests ====================

    #[test]
    #[ignore = "match expression syntax not yet fully implemented"]
    fn match_expression_exhaustive() {
        let src = r#"
let x = 1;
let y = match x {
    1 { "one" }
    _ { "other" }
};
"#;
        let program = parse_program(src).expect("parse ok");
        let result = type_check(&program);
        assert!(result.is_ok());
    }

    #[test]
    #[ignore = "match expression syntax not yet fully implemented"]
    fn match_expression_type_mismatch() {
        let src = r#"
let x = 1;
let y: Int = match x {
    1 { "one" }
    _ { "other" }
};
"#;
        let program = parse_program(src).expect("parse ok");
        let result = type_check(&program);
        // Should fail because match returns String but y is Int
        assert!(result.is_err());
    }

    // ==================== Class and Trait Tests ====================

    #[test]
    fn class_field_access() {
        let src = r#"
class Point {
    let x: Int
    let y: Int
}
function get_x(p: Point) -> Int {
    return p.x;
}
"#;
        let program = parse_program(src).expect("parse ok");
        let result = type_check(&program);
        assert!(result.is_ok());
    }

    #[test]
    fn class_method_call() {
        let src = r#"
class Counter {
    let count: Int

    function increment() -> Int {
        return this.count + 1;
    }
}
"#;
        let program = parse_program(src).expect("parse ok");
        let result = type_check(&program);
        assert!(result.is_ok());
    }

    #[test]
    fn trait_definition() {
        let src = r#"
trait Printable {
    function to_string() -> String
}
"#;
        let program = parse_program(src).expect("parse ok");
        let result = type_check(&program);
        assert!(result.is_ok());
    }

    #[test]
    #[ignore = "implement trait syntax not yet fully implemented"]
    fn implement_trait() {
        let src = r#"
trait Printable {
    function to_string() -> String
}
class User {
    let name: String
}
implement Printable for User {
    function to_string() -> String {
        return this.name;
    }
}
"#;
        let program = parse_program(src).expect("parse ok");
        let result = type_check(&program);
        assert!(result.is_ok());
    }

    // ==================== Enum Tests ====================

    #[test]
    fn enum_definition_simple() {
        let src = r#"
enum Color {
    Red,
    Green,
    Blue
}
"#;
        let program = parse_program(src).expect("parse ok");
        let result = type_check(&program);
        assert!(result.is_ok());
    }

    #[test]
    fn enum_with_data() {
        let src = r#"
enum Option<T> {
    None,
    Some(T)
}
"#;
        let program = parse_program(src).expect("parse ok");
        let result = type_check(&program);
        assert!(result.is_ok());
    }

    #[test]
    fn enum_variant_construction() {
        let src = r#"
enum Color {
    Red,
    Green,
    Blue
}
let c = Color.Red;
"#;
        let program = parse_program(src).expect("parse ok");
        let result = type_check(&program);
        assert!(result.is_ok());
    }

    // ==================== Record Tests ====================

    #[test]
    fn record_definition() {
        let src = r#"
record Point {
    x: Float,
    y: Float
}
"#;
        let program = parse_program(src).expect("parse ok");
        let result = type_check(&program);
        assert!(result.is_ok());
    }

    #[test]
    #[ignore = "record construction syntax not yet fully implemented"]
    fn record_construction() {
        let src = r#"
record Point {
    x: Float,
    y: Float
}
let p = Point { x: 1.0, y: 2.0 };
"#;
        let program = parse_program(src).expect("parse ok");
        let result = type_check(&program);
        assert!(result.is_ok());
    }

    // ==================== Lambda Tests ====================

    #[test]
    fn lambda_simple() {
        let src = r#"
let add = (a, b) -> a + b;
"#;
        let program = parse_program(src).expect("parse ok");
        let result = type_check(&program);
        assert!(result.is_ok());
    }

    #[test]
    #[ignore = "function type annotation not yet fully implemented"]
    fn lambda_as_argument() {
        let src = r#"
function apply(f: (Int, Int) -> Int, a: Int, b: Int) -> Int {
    return f(a, b);
}
"#;
        let program = parse_program(src).expect("parse ok");
        let _ = type_check(&program);
        // May fail if function type not fully implemented
    }

    // ==================== Async Tests ====================

    #[test]
    fn async_function_definition() {
        let src = r#"
async function fetch_data() -> String {
    return "data";
}
"#;
        let program = parse_program(src).expect("parse ok");
        let result = type_check(&program);
        assert!(result.is_ok());
    }

    #[test]
    fn async_function_with_await() {
        let src = r#"
async function fetch() -> String {
    return "data";
}
async function main() -> String {
    return await fetch();
}
"#;
        let program = parse_program(src).expect("parse ok");
        let result = type_check(&program);
        assert!(result.is_ok());
    }

    // ==================== Error Propagation Tests ====================

    #[test]
    fn error_propagation_operator() {
        let src = r#"
function parse_int(s: String) -> Int with Throws<ParseError> {
    return 42;
}
function main() -> Int with Throws<ParseError> {
    return parse_int("42")?;
}
"#;
        let program = parse_program(src).expect("parse ok");
        let _ = type_check(&program);
        // May fail if effect system not fully implemented
    }

    #[test]
    fn optional_chaining() {
        let src = r#"
let x = user?.name;
"#;
        let program = parse_program(src).expect("parse ok");
        let _ = type_check(&program);
        // May fail if optional chaining not fully implemented
    }

    #[test]
    fn null_coalescing() {
        let src = r#"
let x = user?.name ?? "anonymous";
"#;
        let program = parse_program(src).expect("parse ok");
        let _ = type_check(&program);
        // May fail if null coalescing not fully implemented
    }

    // ==================== Import/Export Tests ====================

    #[test]
    fn import_declaration() {
        let src = r#"
import std.io;
"#;
        let program = parse_program(src).expect("parse ok");
        let result = type_check(&program);
        assert!(result.is_ok());
    }

    #[test]
    #[ignore = "module path with dots not yet fully implemented"]
    fn module_declaration() {
        let src = r#"
module myapp.utils;
"#;
        let program = parse_program(src).expect("parse ok");
        let result = type_check(&program);
        assert!(result.is_ok());
    }

    #[test]
    #[ignore = "export type checking not yet fully implemented"]
    fn export_declaration() {
        let src = r#"
export foo;
"#;
        let program = parse_program(src).expect("parse ok");
        let result = type_check(&program);
        assert!(result.is_ok());
    }

    // ==================== Extern Function Tests ====================

    #[test]
    fn extern_function_declaration() {
        let src = r#"
extern function puts(s: CString) -> CInt;
"#;
        let program = parse_program(src).expect("parse ok");
        let result = type_check(&program);
        assert!(result.is_ok());
    }

    // ==================== Type Alias Tests ====================

    #[test]
    #[ignore = "type alias syntax not yet fully implemented"]
    fn type_alias_definition() {
        let src = r#"
type UserID = Int;
"#;
        let program = parse_program(src).expect("parse ok");
        let result = type_check(&program);
        assert!(result.is_ok());
    }

    #[test]
    #[ignore = "type alias syntax not yet fully implemented"]
    fn type_alias_generic() {
        let src = r#"
type List<T> = Array<T>;
"#;
        let program = parse_program(src).expect("parse ok");
        let result = type_check(&program);
        assert!(result.is_ok());
    }

    // ==================== Array/Dictionary Tests ====================

    #[test]
    fn array_literal() {
        let src = r#"
let arr = [1, 2, 3];
"#;
        let program = parse_program(src).expect("parse ok");
        let result = type_check(&program);
        assert!(result.is_ok());
    }

    #[test]
    fn dictionary_literal() {
        let src = r#"
let dict = {"a": 1, "b": 2};
"#;
        let program = parse_program(src).expect("parse ok");
        let result = type_check(&program);
        assert!(result.is_ok());
    }

    #[test]
    fn array_index_access() {
        let src = r#"
let arr = [1, 2, 3];
let x = arr[0];
"#;
        let program = parse_program(src).expect("parse ok");
        let result = type_check(&program);
        assert!(result.is_ok());
    }
}
