// MIR (Middle Intermediate Representation)
//
// 中层中间表示，采用控制流图（CFG）形式
// 适合进行控制流分析、数据流分析和优化 Pass

use std::collections::HashMap;

/// 导入声明（从 HIR 保留到 MIR）
#[derive(Debug, Clone)]
pub struct Import {
    /// 模块路径
    pub module_path: String,
    /// 导入的符号列表：(name, alias)
    pub symbols: Vec<(String, Option<String>)>,
    /// 是否导入全部
    pub import_all: bool,
}

/// MIR 模块
#[derive(Debug, Clone)]
pub struct MirModule {
    /// 模块名
    pub name: String,
    /// 导入声明
    pub imports: Vec<Import>,
    /// 函数列表
    pub functions: Vec<MirFunction>,
    /// 全局变量
    pub globals: Vec<MirGlobal>,
}

/// 类型参数
#[derive(Debug, Clone)]
pub struct TypeParameter {
    /// 参数名
    pub name: String,
}

/// MIR 函数
#[derive(Debug, Clone)]
pub struct MirFunction {
    /// 函数名
    pub name: String,
    /// 类型参数（泛型）
    pub type_params: Vec<TypeParameter>,
    /// 参数
    pub parameters: Vec<MirParameter>,
    /// 返回类型
    pub return_type: MirType,
    /// 基本块列表
    pub blocks: Vec<MirBasicBlock>,
    /// 局部变量（ID -> 类型）
    pub locals: HashMap<MirLocalId, MirType>,
    /// 变量名 -> 局部变量 ID 映射
    pub name_to_local: HashMap<String, MirLocalId>,
    /// 是否是外部函数
    pub is_extern: bool,
}

/// MIR 参数
#[derive(Debug, Clone)]
pub struct MirParameter {
    /// 参数名
    pub name: String,
    /// 参数类型
    pub ty: MirType,
    /// 参数索引
    pub index: usize,
}

/// MIR 基本块
#[derive(Debug, Clone)]
pub struct MirBasicBlock {
    /// 块 ID
    pub id: MirBlockId,
    /// 指令列表
    pub instructions: Vec<MirInstruction>,
    /// 终止指令
    pub terminator: MirTerminator,
}

impl Default for MirBasicBlock {
    fn default() -> Self {
        Self {
            id: 0,
            instructions: Vec::new(),
            terminator: MirTerminator::Unreachable,
        }
    }
}

/// MIR 局部变量 ID
pub type MirLocalId = usize;

/// MIR 基本块 ID
pub type MirBlockId = usize;

/// MIR 类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MirType {
    /// 整数（指定位宽）
    Int(u32),
    /// 浮点数（指定位宽）
    Float(u32),
    /// 布尔值
    Bool,
    /// 字符串
    String,
    /// 字符
    Char,
    /// 单元类型
    Unit,
    /// 指针
    Pointer(Box<MirType>),
    /// 数组
    Array(Box<MirType>, usize),
    /// 结构体
    Struct(String, Vec<MirType>),
    /// 函数指针
    Function(Vec<MirType>, Box<MirType>),
    /// 未知类型
    Unknown,
}

/// MIR 指令
#[derive(Debug, Clone)]
pub enum MirInstruction {
    /// 赋值
    Assign { dest: MirLocalId, value: MirOperand },
    /// 二元运算
    BinaryOp {
        dest: MirLocalId,
        op: MirBinOp,
        left: MirOperand,
        right: MirOperand,
    },
    /// 一元运算
    UnaryOp {
        dest: MirLocalId,
        op: MirUnOp,
        operand: MirOperand,
    },
    /// 函数调用
    Call {
        dest: Option<MirLocalId>,
        func: MirOperand,
        args: Vec<MirOperand>,
    },
    /// 字段访问
    FieldAccess {
        dest: MirLocalId,
        object: MirOperand,
        field: String,
    },
    /// 数组访问
    ArrayAccess {
        dest: MirLocalId,
        array: MirOperand,
        index: MirOperand,
    },
    /// 分配内存
    Alloc {
        dest: MirLocalId,
        ty: MirType,
        size: usize,
    },
    /// 加载
    Load { dest: MirLocalId, ptr: MirOperand },
    /// 存储
    Store { ptr: MirOperand, value: MirOperand },
    /// 类型转换
    Cast {
        dest: MirLocalId,
        value: MirOperand,
        ty: MirType,
    },
    /// Perceus: 复制引用计数
    Dup { dest: MirLocalId, src: MirOperand },
    /// Perceus: 释放引用
    Drop { value: MirOperand },
    /// Perceus: 复用内存
    Reuse { dest: MirLocalId, src: MirOperand },
    /// When guard: early return guard
    WhenGuard {
        dest: MirLocalId,
        condition: MirOperand,
        body: MirOperand,
    },
}

/// MIR 操作数
#[derive(Debug, Clone)]
pub enum MirOperand {
    /// 局部变量
    Local(MirLocalId),
    /// 常量
    Constant(MirConstant),
    /// 参数
    Param(usize),
    /// 全局变量或函数引用
    Global(String),
}

/// MIR 常量
#[derive(Debug, Clone)]
pub enum MirConstant {
    /// 整数常量
    Int(i64),
    /// 浮点常量
    Float(f64),
    /// 布尔常量
    Bool(bool),
    /// 字符串常量
    String(String),
    /// 字符常量
    Char(char),
    /// 空指针
    Null,
    /// 单元值
    Unit,
}

impl PartialEq for MirConstant {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (MirConstant::Int(a), MirConstant::Int(b)) => a == b,
            (MirConstant::Float(a), MirConstant::Float(b)) => a.to_bits() == b.to_bits(),
            (MirConstant::Bool(a), MirConstant::Bool(b)) => a == b,
            (MirConstant::String(a), MirConstant::String(b)) => a == b,
            (MirConstant::Char(a), MirConstant::Char(b)) => a == b,
            (MirConstant::Null, MirConstant::Null) => true,
            (MirConstant::Unit, MirConstant::Unit) => true,
            _ => false,
        }
    }
}

impl Eq for MirConstant {}

/// MIR 二元运算符
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MirBinOp {
    // 算术运算
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    // 比较运算
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    // 逻辑运算
    And,
    Or,
    // 位运算
    BitAnd,
    BitOr,
    BitXor,
    Shl,
    Shr,
}

/// MIR 一元运算符
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MirUnOp {
    /// 负号
    Neg,
    /// 逻辑非
    Not,
    /// 位非
    BitNot,
    /// 引用
    Reference,
    /// 可变引用
    MutableReference,
}

/// MIR 终止指令
#[derive(Debug, Clone)]
pub enum MirTerminator {
    /// 无条件跳转
    Branch { target: MirBlockId },
    /// 条件跳转
    CondBranch {
        cond: MirOperand,
        then_block: MirBlockId,
        else_block: MirBlockId,
    },
    /// 返回
    Return { value: Option<MirOperand> },
    /// 不可达
    Unreachable,
    /// Switch
    Switch {
        value: MirOperand,
        cases: Vec<(MirConstant, MirBlockId)>,
        default: MirBlockId,
    },
}

/// MIR 全局变量
#[derive(Debug, Clone)]
pub struct MirGlobal {
    /// 名称
    pub name: String,
    /// 类型
    pub ty: MirType,
    /// 初始值
    pub initializer: Option<MirConstant>,
    /// 是否可变
    pub mutable: bool,
}

/// MIR 构建器
pub struct MirBuilder {
    module: MirModule,
    current_function: Option<usize>,
    current_block: Option<MirBlockId>,
    local_counter: MirLocalId,
    block_counter: MirBlockId,
}

impl MirBuilder {
    pub fn new(module_name: &str) -> Self {
        Self {
            module: MirModule {
                name: module_name.to_string(),
                imports: Vec::new(),
                functions: Vec::new(),
                globals: Vec::new(),
            },
            current_function: None,
            current_block: None,
            block_counter: 0,
            local_counter: 0,
        }
    }

    /// 创建新函数
    pub fn create_function(
        &mut self,
        name: &str,
        parameters: Vec<MirParameter>,
        return_type: MirType,
    ) -> usize {
        let func = MirFunction {
            name: name.to_string(),
            type_params: Vec::new(),
            parameters,
            return_type,
            blocks: Vec::new(),
            locals: HashMap::new(),
            name_to_local: HashMap::new(),
            is_extern: false,
        };
        self.module.functions.push(func);
        self.module.functions.len() - 1
    }

    /// 设置当前函数
    pub fn set_current_function(&mut self, index: usize) {
        self.current_function = Some(index);
        self.local_counter = 0;
        self.block_counter = 0;
    }

    /// 创建新基本块
    pub fn create_block(&mut self) -> MirBlockId {
        let id = self.block_counter;
        self.block_counter += 1;
        id
    }

    /// 添加基本块到当前函数
    pub fn add_block(&mut self, block: MirBasicBlock) {
        if let Some(func_idx) = self.current_function {
            self.module.functions[func_idx].blocks.push(block);
        }
    }

    /// 创建新局部变量
    pub fn create_local(&mut self, ty: MirType) -> MirLocalId {
        let id = self.local_counter;
        self.local_counter += 1;
        if let Some(func_idx) = self.current_function {
            self.module.functions[func_idx].locals.insert(id, ty);
        }
        id
    }

    /// 添加指令到当前块
    pub fn push_instruction(&mut self, instr: MirInstruction) {
        if let Some(func_idx) = self.current_function {
            if let Some(block_id) = self.current_block {
                if let Some(block) = self.module.functions[func_idx]
                    .blocks
                    .iter_mut()
                    .find(|b| b.id == block_id)
                {
                    block.instructions.push(instr);
                }
            }
        }
    }

    /// 设置当前基本块
    pub fn set_current_block(&mut self, block_id: MirBlockId) {
        self.current_block = Some(block_id);
    }

    /// 添加导入声明
    pub fn add_import(&mut self, import: Import) {
        self.module.imports.push(import);
    }

    /// 获取构建的模块
    pub fn build(self) -> MirModule {
        self.module
    }
}

impl MirModule {
    /// 创建空的 MIR 模块
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            imports: Vec::new(),
            functions: Vec::new(),
            globals: Vec::new(),
        }
    }
}

impl MirFunction {
    /// 获取入口块
    pub fn entry_block(&self) -> Option<&MirBasicBlock> {
        self.blocks.first()
    }

    /// 获取所有使用到的局部变量
    pub fn used_locals(&self) -> Vec<MirLocalId> {
        let mut locals = Vec::new();
        for block in &self.blocks {
            for instr in &block.instructions {
                match instr {
                    MirInstruction::Assign { dest, .. }
                    | MirInstruction::BinaryOp { dest, .. }
                    | MirInstruction::UnaryOp { dest, .. }
                    | MirInstruction::FieldAccess { dest, .. }
                    | MirInstruction::ArrayAccess { dest, .. }
                    | MirInstruction::Alloc { dest, .. }
                    | MirInstruction::Load { dest, .. }
                    | MirInstruction::Cast { dest, .. }
                    | MirInstruction::Dup { dest, .. }
                    | MirInstruction::Reuse { dest, .. }
                    | MirInstruction::WhenGuard { dest, .. } => {
                        locals.push(*dest);
                    }
                    MirInstruction::Call { dest, .. } => {
                        if let Some(d) = dest {
                            locals.push(*d);
                        }
                    }
                    MirInstruction::Store { .. } | MirInstruction::Drop { .. } => {}
                }
            }
        }
        locals
    }
}
