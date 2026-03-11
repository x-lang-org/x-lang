// Perceus内存管理库
//
// Perceus是一种编译期内存管理技术，通过静态分析确定：
// - 变量的所有权和生命周期
// - 需要插入dup/drop操作的位置
// - 内存复用机会

use std::collections::HashMap;

/// Perceus中间表示
#[derive(Debug, PartialEq, Clone)]
pub struct PerceusIR {
    /// 函数分析结果
    pub functions: Vec<FunctionAnalysis>,
    /// 全局内存操作
    pub global_ops: Vec<MemoryOp>,
    /// 复用分析结果
    pub reuse_analysis: ReuseAnalysis,
}

/// 函数分析结果
#[derive(Debug, PartialEq, Clone)]
pub struct FunctionAnalysis {
    /// 函数名
    pub name: String,
    /// 参数的所有权状态
    pub param_ownership: Vec<OwnershipFact>,
    /// 返回值的所有权状态
    pub return_ownership: OwnershipFact,
    /// 内存操作序列
    pub memory_ops: Vec<MemoryOp>,
    /// 控制流分析
    pub control_flow: ControlFlowAnalysis,
}

/// 所有权事实
#[derive(Debug, PartialEq, Clone)]
pub struct OwnershipFact {
    /// 变量名
    pub variable: String,
    /// 所有权状态
    pub state: OwnershipState,
    /// 类型信息
    pub ty: String,
}

/// 所有权状态
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum OwnershipState {
    /// 拥有所有权
    Owned,
    /// 借用
    Borrowed,
    /// 已移动
    Moved,
    /// 已复制
    Copied,
    /// 已释放
    Dropped,
}

/// 内存操作
#[derive(Debug, PartialEq, Clone)]
pub enum MemoryOp {
    /// 复制操作 (dup)
    Dup {
        variable: String,
        target: String,
        position: SourcePos,
    },
    /// 释放操作 (drop)
    Drop {
        variable: String,
        position: SourcePos,
    },
    /// 复用操作
    Reuse {
        from: String,
        to: String,
        position: SourcePos,
    },
    /// 分配操作
    Alloc {
        variable: String,
        size: usize,
        position: SourcePos,
    },
}

/// 源码位置
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct SourcePos {
    pub line: usize,
    pub column: usize,
}

/// 控制流分析
#[derive(Debug, PartialEq, Clone)]
pub struct ControlFlowAnalysis {
    /// 基本块
    pub blocks: Vec<BasicBlock>,
    /// 边（控制流转移）
    pub edges: Vec<(usize, usize)>,
}

/// 基本块
#[derive(Debug, PartialEq, Clone)]
pub struct BasicBlock {
    /// 块ID
    pub id: usize,
    /// 入口所有权状态
    pub entry_state: HashMap<String, OwnershipState>,
    /// 出口所有权状态
    pub exit_state: HashMap<String, OwnershipState>,
    /// 语句索引
    pub statements: Vec<usize>,
}

/// 复用分析结果
#[derive(Debug, PartialEq, Clone)]
pub struct ReuseAnalysis {
    /// 可复用的变量对
    pub reuse_pairs: Vec<ReusePair>,
    /// 预期节省的内存
    pub estimated_savings: usize,
}

/// 复用对
#[derive(Debug, PartialEq, Clone)]
pub struct ReusePair {
    /// 源变量（将要释放）
    pub source: String,
    /// 目标变量（将要分配）
    pub target: String,
    /// 复用位置
    pub position: SourcePos,
}

/// Perceus分析器
pub struct PerceusAnalyzer {
    /// 当前作用域的变量状态
    variables: HashMap<String, OwnershipState>,
    /// 需要追踪的变量类型
    variable_types: HashMap<String, String>,
    /// 已生成的内存操作
    memory_ops: Vec<MemoryOp>,
    /// 当前源码位置
    current_pos: SourcePos,
}

impl PerceusAnalyzer {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            variable_types: HashMap::new(),
            memory_ops: Vec::new(),
            current_pos: SourcePos { line: 1, column: 1 },
        }
    }

    /// 分析HIR并生成PerceusIR
    pub fn analyze(&mut self, hir: &x_hir::Hir) -> Result<PerceusIR, PerceusError> {
        let mut functions = Vec::new();

        // 分析每个函数声明
        for decl in &hir.declarations {
            if let x_hir::HirDeclaration::Function(func) = decl {
                let analysis = self.analyze_function(func)?;
                functions.push(analysis);
            }
        }

        // 构建复用分析
        let reuse_analysis = self.build_reuse_analysis();

        Ok(PerceusIR {
            functions,
            global_ops: self.memory_ops.clone(),
            reuse_analysis,
        })
    }

    /// 分析单个函数
    fn analyze_function(&mut self, func: &x_hir::HirFunctionDecl) -> Result<FunctionAnalysis, PerceusError> {
        // 重置状态
        self.variables.clear();
        self.variable_types.clear();
        self.memory_ops.clear();

        // 分析参数
        let mut param_ownership = Vec::new();
        for param in &func.parameters {
            let ty_str = self.type_to_string(&param.ty);
            self.variables.insert(param.name.clone(), OwnershipState::Owned);
            self.variable_types.insert(param.name.clone(), ty_str.clone());
            param_ownership.push(OwnershipFact {
                variable: param.name.clone(),
                state: OwnershipState::Owned,
                ty: ty_str,
            });
        }

        // 分析函数体
        self.analyze_block(&func.body)?;

        // 构建控制流分析
        let control_flow = self.build_control_flow(&func.body);

        // 确定返回值所有权
        let return_ownership = OwnershipFact {
            variable: "return".to_string(),
            state: OwnershipState::Owned,
            ty: self.type_to_string(&func.return_type),
        };

        Ok(FunctionAnalysis {
            name: func.name.clone(),
            param_ownership,
            return_ownership,
            memory_ops: self.memory_ops.clone(),
            control_flow,
        })
    }

    /// 分析代码块
    fn analyze_block(&mut self, block: &x_hir::HirBlock) -> Result<(), PerceusError> {
        for stmt in &block.statements {
            self.analyze_statement(stmt)?;
        }
        Ok(())
    }

    /// 分析语句
    fn analyze_statement(&mut self, stmt: &x_hir::HirStatement) -> Result<(), PerceusError> {
        match stmt {
            x_hir::HirStatement::Variable(var_decl) => {
                // 变量声明
                let ty_str = self.type_to_string(&var_decl.ty);
                self.variables.insert(var_decl.name.clone(), OwnershipState::Owned);
                self.variable_types.insert(var_decl.name.clone(), ty_str);

                // 如果有初始化表达式，分析它
                if let Some(init) = &var_decl.initializer {
                    self.analyze_expression(init)?;
                }
            }
            x_hir::HirStatement::Expression(expr) => {
                self.analyze_expression(expr)?;
            }
            x_hir::HirStatement::Return(opt_expr) => {
                if let Some(expr) = opt_expr {
                    self.analyze_expression(expr)?;
                    // 返回值移动所有权
                    self.transfer_ownership(expr, OwnershipState::Moved)?;
                }
            }
            x_hir::HirStatement::If(if_stmt) => {
                self.analyze_expression(&if_stmt.condition)?;
                self.analyze_block(&if_stmt.then_block)?;
                if let Some(else_block) = &if_stmt.else_block {
                    self.analyze_block(else_block)?;
                }
            }
            x_hir::HirStatement::While(while_stmt) => {
                self.analyze_expression(&while_stmt.condition)?;
                self.analyze_block(&while_stmt.body)?;
            }
            x_hir::HirStatement::For(for_stmt) => {
                self.analyze_expression(&for_stmt.iterator)?;
                // 循环变量
                if let x_hir::HirPattern::Variable(name) = &for_stmt.pattern {
                    self.variables.insert(name.clone(), OwnershipState::Owned);
                }
                self.analyze_block(&for_stmt.body)?;
            }
            x_hir::HirStatement::Match(match_stmt) => {
                self.analyze_expression(&match_stmt.expression)?;
                for case in &match_stmt.cases {
                    self.analyze_block(&case.body)?;
                }
            }
            x_hir::HirStatement::Try(try_stmt) => {
                self.analyze_block(&try_stmt.body)?;
                for catch in &try_stmt.catch_clauses {
                    if let Some(var_name) = &catch.variable_name {
                        self.variables.insert(var_name.clone(), OwnershipState::Owned);
                    }
                    self.analyze_block(&catch.body)?;
                }
                if let Some(finally) = &try_stmt.finally_block {
                    self.analyze_block(finally)?;
                }
            }
            x_hir::HirStatement::Break | x_hir::HirStatement::Continue => {
                // 控制流语句，无需特殊处理
            }
        }
        Ok(())
    }

    /// 分析表达式
    fn analyze_expression(&mut self, expr: &x_hir::HirExpression) -> Result<(), PerceusError> {
        match expr {
            x_hir::HirExpression::Variable(name) => {
                // 变量引用 - 检查是否需要dup
                if let Some(state) = self.variables.get_mut(name) {
                    if *state == OwnershipState::Owned {
                        // 第一次使用，移动所有权
                        *state = OwnershipState::Moved;
                    } else if *state == OwnershipState::Moved {
                        // 已移动，需要dup
                        self.memory_ops.push(MemoryOp::Dup {
                            variable: name.clone(),
                            target: format!("{}_dup", name),
                            position: self.current_pos,
                        });
                    }
                }
            }
            x_hir::HirExpression::Call(callee, args) => {
                self.analyze_expression(callee)?;
                for arg in args {
                    self.analyze_expression(arg)?;
                }
            }
            x_hir::HirExpression::Binary(_, left, right) => {
                self.analyze_expression(left)?;
                self.analyze_expression(right)?;
            }
            x_hir::HirExpression::Unary(_, expr) => {
                self.analyze_expression(expr)?;
            }
            x_hir::HirExpression::Member(obj, _) => {
                self.analyze_expression(obj)?;
            }
            x_hir::HirExpression::Assign(target, value) => {
                self.analyze_expression(value)?;
                self.analyze_expression(target)?;
            }
            x_hir::HirExpression::If(cond, then_e, else_e) => {
                self.analyze_expression(cond)?;
                self.analyze_expression(then_e)?;
                self.analyze_expression(else_e)?;
            }
            x_hir::HirExpression::Lambda(_, body) => {
                self.analyze_block(body)?;
            }
            x_hir::HirExpression::Array(items) => {
                for item in items {
                    self.analyze_expression(item)?;
                }
            }
            x_hir::HirExpression::Dictionary(entries) => {
                for (k, v) in entries {
                    self.analyze_expression(k)?;
                    self.analyze_expression(v)?;
                }
            }
            x_hir::HirExpression::Record(_, fields) => {
                for (_, value) in fields {
                    self.analyze_expression(value)?;
                }
            }
            x_hir::HirExpression::Range(start, end, _) => {
                self.analyze_expression(start)?;
                self.analyze_expression(end)?;
            }
            x_hir::HirExpression::Pipe(input, funcs) => {
                self.analyze_expression(input)?;
                for func in funcs {
                    self.analyze_expression(func)?;
                }
            }
            x_hir::HirExpression::Wait(_, exprs) => {
                for expr in exprs {
                    self.analyze_expression(expr)?;
                }
            }
            x_hir::HirExpression::Given(_, expr) => {
                self.analyze_expression(expr)?;
            }
            _ => {}
        }
        Ok(())
    }

    /// 转移所有权
    fn transfer_ownership(&mut self, expr: &x_hir::HirExpression, new_state: OwnershipState) -> Result<(), PerceusError> {
        if let x_hir::HirExpression::Variable(name) = expr {
            if let Some(state) = self.variables.get_mut(name) {
                *state = new_state;
            }
        }
        Ok(())
    }

    /// 构建控制流分析
    fn build_control_flow(&self, _block: &x_hir::HirBlock) -> ControlFlowAnalysis {
        // 简化版本：单块控制流
        ControlFlowAnalysis {
            blocks: vec![BasicBlock {
                id: 0,
                entry_state: HashMap::new(),
                exit_state: self.variables.clone(),
                statements: vec![0],
            }],
            edges: vec![],
        }
    }

    /// 构建复用分析
    fn build_reuse_analysis(&self) -> ReuseAnalysis {
        // 分析drop和alloc操作，寻找复用机会
        let mut reuse_pairs = Vec::new();
        let mut drops: Vec<&String> = Vec::new();
        let mut allocs: Vec<&String> = Vec::new();

        for op in &self.memory_ops {
            match op {
                MemoryOp::Drop { variable, .. } => drops.push(variable),
                MemoryOp::Alloc { variable, .. } => allocs.push(variable),
                _ => {}
            }
        }

        // 简单启发式：匹配drop后的alloc
        for (i, drop_var) in drops.iter().enumerate() {
            if i < allocs.len() {
                reuse_pairs.push(ReusePair {
                    source: (*drop_var).clone(),
                    target: allocs[i].clone(),
                    position: self.current_pos,
                });
            }
        }

        let estimated_savings = reuse_pairs.len() * 8; // 假设每个复用节省8字节
        ReuseAnalysis {
            reuse_pairs,
            estimated_savings,
        }
    }

    /// 类型转字符串
    fn type_to_string(&self, ty: &x_hir::HirType) -> String {
        match ty {
            x_hir::HirType::Int => "Int".to_string(),
            x_hir::HirType::Float => "Float".to_string(),
            x_hir::HirType::Bool => "Bool".to_string(),
            x_hir::HirType::String => "String".to_string(),
            x_hir::HirType::Char => "Char".to_string(),
            x_hir::HirType::Unit => "Unit".to_string(),
            x_hir::HirType::Never => "Never".to_string(),
            x_hir::HirType::Array(inner) => format!("Array<{}>", self.type_to_string(inner)),
            x_hir::HirType::Option(inner) => format!("Option<{}>", self.type_to_string(inner)),
            x_hir::HirType::Result(ok, err) => format!("Result<{}, {}>", self.type_to_string(ok), self.type_to_string(err)),
            _ => "Unknown".to_string(),
        }
    }
}

impl Default for PerceusAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// 对高级中间表示进行Perceus分析
pub fn analyze_hir(hir: &x_hir::Hir) -> Result<PerceusIR, PerceusError> {
    let mut analyzer = PerceusAnalyzer::new();
    analyzer.analyze(hir)
}

/// Perceus分析错误
#[derive(thiserror::Error, Debug)]
pub enum PerceusError {
    #[error("分析错误: {0}")]
    AnalysisError(String),
    #[error("所有权错误: {0}")]
    OwnershipError(String),
    #[error("未定义的变量: {0}")]
    UndefinedVariable(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use x_hir::{Hir, HirDeclaration, HirFunctionDecl, HirBlock, HirType, HirTypeEnv, HirParameter};

    fn create_test_function(name: &str, params: Vec<(&str, HirType)>) -> Hir {
        let parameters = params.into_iter().map(|(n, t)| HirParameter {
            name: n.to_string(),
            ty: t,
            default: None,
        }).collect();

        Hir {
            module_name: "test".to_string(),
            declarations: vec![HirDeclaration::Function(HirFunctionDecl {
                name: name.to_string(),
                parameters,
                return_type: HirType::Unit,
                body: HirBlock { statements: vec![] },
                is_async: false,
                effects: vec![],
            })],
            statements: vec![],
            type_env: HirTypeEnv {
                variables: HashMap::new(),
                functions: HashMap::new(),
                types: HashMap::new(),
            },
        }
    }

    #[test]
    fn analyze_hir_returns_ok_for_empty_hir() {
        let hir = Hir {
            module_name: "test".to_string(),
            declarations: vec![],
            statements: vec![],
            type_env: HirTypeEnv {
                variables: HashMap::new(),
                functions: HashMap::new(),
                types: HashMap::new(),
            },
        };
        let pir = analyze_hir(&hir).expect("analyze_hir");
        assert!(pir.functions.is_empty());
    }

    #[test]
    fn analyze_hir_extracts_function_info() {
        let hir = create_test_function("add", vec![
            ("x", HirType::Int),
            ("y", HirType::Int),
        ]);

        let pir = analyze_hir(&hir).expect("analyze_hir");
        assert_eq!(pir.functions.len(), 1);
        assert_eq!(pir.functions[0].name, "add");
        assert_eq!(pir.functions[0].param_ownership.len(), 2);
    }

    #[test]
    fn perceus_error_displays_message() {
        let e = PerceusError::AnalysisError("test message".to_string());
        assert!(e.to_string().contains("分析错误"));
        assert!(e.to_string().contains("test message"));
    }

    #[test]
    fn ownership_state_can_be_tracked() {
        let mut analyzer = PerceusAnalyzer::new();
        analyzer.variables.insert("x".to_string(), OwnershipState::Owned);

        assert_eq!(analyzer.variables.get("x"), Some(&OwnershipState::Owned));
    }

    #[test]
    fn memory_ops_can_be_recorded() {
        let mut analyzer = PerceusAnalyzer::new();
        analyzer.memory_ops.push(MemoryOp::Drop {
            variable: "x".to_string(),
            position: SourcePos { line: 1, column: 1 },
        });

        assert_eq!(analyzer.memory_ops.len(), 1);
    }

    #[test]
    fn reuse_analysis_identifies_opportunities() {
        let analyzer = PerceusAnalyzer::new();
        let reuse = analyzer.build_reuse_analysis();
        assert!(reuse.reuse_pairs.is_empty()); // 没有操作时为空
    }
}
