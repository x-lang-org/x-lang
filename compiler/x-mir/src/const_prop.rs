//! 常量传播优化
//!
//! 该模块实现稀疏条件常量传播算法，能够：
//! 1. 在控制流图中追踪变量的常量值
//! 2. 当变量值确定为常量时，替换所有使用
//! 3. 处理条件分支中的常量条件，消除不可达分支

use std::collections::{HashMap, HashSet};

use crate::{
    MirBinOp, MirConstant, MirFunction, MirInstruction, MirLocalId, MirModule, MirOperand,
    MirTerminator, MirUnOp,
};

/// 常量格值
#[derive(Debug, Clone, PartialEq)]
pub enum ConstantValue {
    /// 未定义（还未遇到定义）
    Undefined,
    /// 常量（值确定）
    Constant(MirConstant),
    /// 变量（值不确定）
    Variable,
}

impl ConstantValue {
    /// 格交操作：meet
    pub fn meet(&self, other: &Self) -> Self {
        match (self, other) {
            // undef ∧ x = x
            (ConstantValue::Undefined, other) => other.clone(),
            (other, ConstantValue::Undefined) => other.clone(),
            // 相同常量保持不变
            (ConstantValue::Constant(c1), ConstantValue::Constant(c2)) if c1 == c2 => {
                ConstantValue::Constant(c1.clone())
            }
            // 不同常量 → 变量
            (ConstantValue::Constant(_), ConstantValue::Constant(_)) => ConstantValue::Variable,
            // 任意与变量 → 变量
            (ConstantValue::Variable, _) | (_, ConstantValue::Variable) => ConstantValue::Variable,
        }
    }

    /// 是否为常量
    pub fn is_constant(&self) -> bool {
        matches!(self, ConstantValue::Constant(_))
    }

    /// 是否为未定义
    pub fn is_undefined(&self) -> bool {
        matches!(self, ConstantValue::Undefined)
    }
}

/// 基本块的常量信息
#[derive(Debug, Clone, Default)]
pub struct BlockConstInfo {
    /// 每个局部变量的当前常量值
    pub values: HashMap<MirLocalId, ConstantValue>,
    /// 是否已经处理过
    pub processed: bool,
}

/// 常量传播优化器
pub struct ConstantPropagation {
    /// 工作列表：需要重新处理的基本块ID
    work_list: Vec<usize>,
}

impl ConstantPropagation {
    pub fn new() -> Self {
        Self {
            work_list: Vec::new(),
        }
    }

    /// 对整个模块执行常量传播
    pub fn run_on_module(&mut self, module: &mut MirModule) {
        for func in &mut module.functions {
            if !func.is_extern && !func.blocks.is_empty() {
                self.run_on_function(func);
            }
        }
    }

    /// 对单个函数执行常量传播
    pub fn run_on_function(&mut self, func: &mut MirFunction) {
        // 初始化每个基本块的常量信息
        let mut block_info: HashMap<usize, BlockConstInfo> = func
            .blocks
            .iter()
            .map(|block| (block.id, BlockConstInfo::default()))
            .collect();

        // 入口块所有变量初始为未定义
        // 将入口块加入工作列表
        if let Some(entry_block) = func.blocks.first() {
            self.add_to_worklist(entry_block.id, &mut block_info);
        }

        // 处理工作列表
        while let Some(block_id) = self.work_list.pop() {
            let (changed, out_values) = self.process_block(block_id, func, &block_info);
            let info = block_info.get_mut(&block_id).unwrap();

            // 如果输出发生变化，将后继块加入工作列表
            if changed {
                // 更新当前块的值
                for (local, value) in &out_values {
                    info.values.insert(*local, value.clone());
                }
                info.processed = true;

                // 获取后继基本块
                let successors = self.get_successors(block_id, func);
                for succ_id in successors {
                    self.add_to_worklist(succ_id, &mut block_info);
                }
            }
        }

        // 现在所有常量值已知，进行替换
        self.replace_constants(func, &block_info);

        // 消除不可达代码
        self.eliminate_unreachable_blocks(func, &block_info);
    }

    /// 添加基本块到工作列表
    fn add_to_worklist(
        &mut self,
        block_id: usize,
        block_info: &mut HashMap<usize, BlockConstInfo>,
    ) {
        if !self.work_list.contains(&block_id) {
            block_info.entry(block_id).or_default().processed = false;
            self.work_list.push(block_id);
        }
    }

    /// 获取基本块的后继
    fn get_successors(&self, block_id: usize, func: &MirFunction) -> Vec<usize> {
        let Some(block) = func.blocks.iter().find(|b| b.id == block_id) else {
            return Vec::new();
        };

        match &block.terminator {
            MirTerminator::Branch { target } => vec![*target],
            MirTerminator::CondBranch {
                then_block,
                else_block,
                ..
            } => vec![*then_block, *else_block],
            MirTerminator::Switch { cases, default, .. } => {
                let mut succ = cases.iter().map(|(_, b)| *b).collect::<Vec<_>>();
                succ.push(*default);
                succ
            }
            MirTerminator::Return { .. } | MirTerminator::Unreachable => Vec::new(),
        }
    }

    /// 处理单个基本块，计算输出值
    fn process_block(
        &self,
        block_id: usize,
        func: &MirFunction,
        block_info: &HashMap<usize, BlockConstInfo>,
    ) -> (bool, HashMap<MirLocalId, ConstantValue>) {
        let block = func.blocks.iter().find(|b| b.id == block_id).unwrap();

        // 计算入口值：meet 所有前驱的出口值
        let mut current_values: HashMap<MirLocalId, ConstantValue> = HashMap::new();

        // 获取前驱
        let predecessors = self.get_predecessors(block_id, func);

        if predecessors.is_empty() {
            // 入口块，所有变量初始未定义
            for local in func.locals.keys() {
                current_values.insert(*local, ConstantValue::Undefined);
            }
        } else {
            // 对每个变量，meet 所有前驱的值
            for local in func.locals.keys() {
                let mut value = ConstantValue::Undefined;
                for pred_id in &predecessors {
                    if let Some(pred_info) = block_info.get(pred_id) {
                        if let Some(pred_value) = pred_info.values.get(local) {
                            value = value.meet(pred_value);
                        }
                    }
                }
                current_values.insert(*local, value);
            }
        }

        let old_values = current_values.clone();

        // 遍历指令，更新常量值
        for instr in &block.instructions {
            self.evaluate_instr(instr, &mut current_values);
        }

        // 检查是否变化
        let changed = current_values != old_values;
        (changed, current_values)
    }

    /// 获取前驱基本块
    fn get_predecessors(&self, block_id: usize, func: &MirFunction) -> Vec<usize> {
        let mut preds = Vec::new();
        for block in &func.blocks {
            let succs = self.get_successors(block.id, func);
            if succs.contains(&block_id) {
                preds.push(block.id);
            }
        }
        preds
    }

    /// 根据当前值计算指令结果
    fn evaluate_instr(
        &self,
        instr: &MirInstruction,
        current_values: &mut HashMap<MirLocalId, ConstantValue>,
    ) {
        match instr {
            MirInstruction::Assign { dest, value } => {
                let value = self.evaluate_operand(value, current_values);
                current_values.insert(*dest, value);
            }
            MirInstruction::BinaryOp {
                dest,
                op,
                left,
                right,
                ..
            } => {
                let left_val = self.evaluate_operand(left, current_values);
                let right_val = self.evaluate_operand(right, current_values);

                let result = self.evaluate_binary_op(*op, &left_val, &right_val);
                current_values.insert(*dest, result);
            }
            MirInstruction::UnaryOp {
                dest, op, operand, ..
            } => {
                let val = self.evaluate_operand(operand, current_values);
                let result = self.evaluate_unary_op(*op, &val);
                current_values.insert(*dest, result);
            }
            MirInstruction::Cast {
                dest, value, ty, ..
            } => {
                let val = self.evaluate_operand(value, current_values);
                let result = self.evaluate_cast(&val, ty);
                current_values.insert(*dest, result);
            }
            MirInstruction::Dup { dest, src } => {
                let src_val = self.evaluate_operand(src, current_values);
                current_values.insert(*dest, src_val);
            }
            MirInstruction::Reuse { dest, src } => {
                let src_val = self.evaluate_operand(src, current_values);
                current_values.insert(*dest, src_val);
            }
            // 这些指令不产生可常量传播的结果
            MirInstruction::Call { dest, .. } => {
                if let Some(dest) = dest {
                    current_values.insert(*dest, ConstantValue::Variable);
                }
            }
            MirInstruction::FieldAccess { dest, .. } => {
                current_values.insert(*dest, ConstantValue::Variable);
            }
            MirInstruction::ArrayAccess { dest, .. } => {
                current_values.insert(*dest, ConstantValue::Variable);
            }
            MirInstruction::Alloc { dest, .. } => {
                current_values.insert(*dest, ConstantValue::Variable);
            }
            MirInstruction::Load { dest, .. } => {
                current_values.insert(*dest, ConstantValue::Variable);
            }
            MirInstruction::Store { .. } => {
                // 不影响当前局部变量表
            }
            MirInstruction::Drop { .. } => {
                // 不产生新值
            }
            MirInstruction::WhenGuard { .. } => {
                // When guard - no constant propagation
            }
        }
    }

    /// 求值操作数
    fn evaluate_operand(
        &self,
        operand: &MirOperand,
        current_values: &HashMap<MirLocalId, ConstantValue>,
    ) -> ConstantValue {
        match operand {
            MirOperand::Local(id) => current_values
                .get(id)
                .cloned()
                .unwrap_or(ConstantValue::Undefined),
            MirOperand::Constant(c) => ConstantValue::Constant(c.clone()),
            MirOperand::Param(_) => ConstantValue::Variable,
            MirOperand::Global(_) => ConstantValue::Variable,
        }
    }

    /// 求值二元运算
    fn evaluate_binary_op(
        &self,
        op: MirBinOp,
        left: &ConstantValue,
        right: &ConstantValue,
    ) -> ConstantValue {
        let (ConstantValue::Constant(left_c), ConstantValue::Constant(right_c)) = (left, right)
        else {
            return ConstantValue::Variable;
        };

        // 都是常量，计算结果
        match (left_c, right_c) {
            (MirConstant::Int(l), MirConstant::Int(r)) => {
                let result = match op {
                    MirBinOp::Add => MirConstant::Int(l.wrapping_add(*r)),
                    MirBinOp::Sub => MirConstant::Int(l.wrapping_sub(*r)),
                    MirBinOp::Mul => MirConstant::Int(l.wrapping_mul(*r)),
                    MirBinOp::Div if *r != 0 => MirConstant::Int(l / *r),
                    MirBinOp::Mod if *r != 0 => MirConstant::Int(l % *r),
                    MirBinOp::Eq => MirConstant::Bool(l == r),
                    MirBinOp::Ne => MirConstant::Bool(l != r),
                    MirBinOp::Lt => MirConstant::Bool(l < r),
                    MirBinOp::Le => MirConstant::Bool(l <= r),
                    MirBinOp::Gt => MirConstant::Bool(l > r),
                    MirBinOp::Ge => MirConstant::Bool(l >= r),
                    MirBinOp::BitAnd => MirConstant::Int(l & r),
                    MirBinOp::BitOr => MirConstant::Int(l | r),
                    MirBinOp::BitXor => MirConstant::Int(l ^ r),
                    MirBinOp::Shl if *r >= 0 && *r < 64 => MirConstant::Int(l << r),
                    MirBinOp::Shr if *r >= 0 && *r < 64 => MirConstant::Int(l >> r),
                    MirBinOp::And => MirConstant::Bool(*l != 0 && *r != 0),
                    MirBinOp::Or => MirConstant::Bool(*l != 0 || *r != 0),
                    _ => return ConstantValue::Variable,
                };
                ConstantValue::Constant(result)
            }
            (MirConstant::Float(l), MirConstant::Float(r)) => {
                let result = match op {
                    MirBinOp::Add => MirConstant::Float(l + r),
                    MirBinOp::Sub => MirConstant::Float(l - r),
                    MirBinOp::Mul => MirConstant::Float(l * r),
                    MirBinOp::Div => MirConstant::Float(l / r),
                    MirBinOp::Eq => MirConstant::Bool(l == r),
                    MirBinOp::Ne => MirConstant::Bool(l != r),
                    MirBinOp::Lt => MirConstant::Bool(l < r),
                    MirBinOp::Le => MirConstant::Bool(l <= r),
                    MirBinOp::Gt => MirConstant::Bool(l > r),
                    MirBinOp::Ge => MirConstant::Bool(l >= r),
                    _ => return ConstantValue::Variable,
                };
                ConstantValue::Constant(result)
            }
            (MirConstant::Bool(l), MirConstant::Bool(r)) => {
                let result = match op {
                    MirBinOp::And => MirConstant::Bool(*l && *r),
                    MirBinOp::Or => MirConstant::Bool(*l || *r),
                    MirBinOp::Eq => MirConstant::Bool(l == r),
                    MirBinOp::Ne => MirConstant::Bool(l != r),
                    _ => return ConstantValue::Variable,
                };
                ConstantValue::Constant(result)
            }
            _ => ConstantValue::Variable,
        }
    }

    /// 求值一元运算
    fn evaluate_unary_op(&self, op: MirUnOp, operand: &ConstantValue) -> ConstantValue {
        let ConstantValue::Constant(val) = operand else {
            return ConstantValue::Variable;
        };

        match val {
            MirConstant::Int(v) => {
                let result = match op {
                    MirUnOp::Neg => MirConstant::Int(-v),
                    MirUnOp::BitNot => MirConstant::Int(!v),
                    MirUnOp::Not => MirConstant::Bool(*v == 0),
                    MirUnOp::Reference | MirUnOp::MutableReference => {
                        return ConstantValue::Variable;
                    }
                };
                ConstantValue::Constant(result)
            }
            MirConstant::Bool(v) => {
                let result = match op {
                    MirUnOp::Not => MirConstant::Bool(!v),
                    _ => return ConstantValue::Variable,
                };
                ConstantValue::Constant(result)
            }
            MirConstant::Float(v) => {
                let result = match op {
                    MirUnOp::Neg => MirConstant::Float(-v),
                    _ => return ConstantValue::Variable,
                };
                ConstantValue::Constant(result)
            }
            _ => ConstantValue::Variable,
        }
    }

    /// 求值类型转换
    fn evaluate_cast(&self, value: &ConstantValue, _ty: &crate::MirType) -> ConstantValue {
        // 简化处理：如果输入是常量，输出也是常量（类型转换由后端处理）
        // 如果需要更精确，可以在这里根据目标类型转换常量值
        value.clone()
    }

    /// 将推断出的常量替换回指令
    fn replace_constants(
        &self,
        func: &mut MirFunction,
        block_info: &HashMap<usize, BlockConstInfo>,
    ) {
        for block in &mut func.blocks {
            let Some(info) = block_info.get(&block.id) else {
                continue;
            };

            // 替换每条指令中的操作数
            for instr in &mut block.instructions {
                self.replace_instr(instr, info);
            }

            // 替换终止指令中的操作数
            self.replace_terminator(&mut block.terminator, info);
        }
    }

    /// 替换一条指令中的常量
    fn replace_instr(&self, instr: &mut MirInstruction, info: &BlockConstInfo) {
        match instr {
            MirInstruction::Assign { value, .. } => {
                self.replace_operand(value, info);
            }
            MirInstruction::BinaryOp { left, right, .. } => {
                self.replace_operand(left, info);
                self.replace_operand(right, info);
            }
            MirInstruction::UnaryOp { operand, .. } => {
                self.replace_operand(operand, info);
            }
            MirInstruction::Call { func, args, .. } => {
                self.replace_operand(func, info);
                for arg in args {
                    self.replace_operand(arg, info);
                }
            }
            MirInstruction::FieldAccess { object, .. } => {
                self.replace_operand(object, info);
            }
            MirInstruction::ArrayAccess { array, index, .. } => {
                self.replace_operand(array, info);
                self.replace_operand(index, info);
            }
            MirInstruction::Alloc { .. } => {}
            MirInstruction::Load { ptr, .. } => {
                self.replace_operand(ptr, info);
            }
            MirInstruction::Store { ptr, value, .. } => {
                self.replace_operand(ptr, info);
                self.replace_operand(value, info);
            }
            MirInstruction::Cast { value, .. } => {
                self.replace_operand(value, info);
            }
            MirInstruction::Dup { src, .. } => {
                self.replace_operand(src, info);
            }
            MirInstruction::Drop { value } => {
                self.replace_operand(value, info);
            }
            MirInstruction::Reuse { src, .. } => {
                self.replace_operand(src, info);
            }
            MirInstruction::WhenGuard {
                condition, body, ..
            } => {
                self.replace_operand(condition, info);
                self.replace_operand(body, info);
            }
        }
    }

    /// 替换终止指令中的常量
    fn replace_terminator(&self, terminator: &mut MirTerminator, info: &BlockConstInfo) {
        match terminator {
            MirTerminator::CondBranch { cond, .. } => {
                self.replace_operand(cond, info);
            }
            MirTerminator::Switch { value, .. } => {
                self.replace_operand(value, info);
            }
            MirTerminator::Return { value: Some(v) } => {
                self.replace_operand(v, info);
            }
            MirTerminator::Return { value: None } => {}
            _ => {}
        }
    }

    /// 如果可能，将操作数替换为常量
    fn replace_operand(&self, operand: &mut MirOperand, info: &BlockConstInfo) {
        let MirOperand::Local(local_id) = operand else {
            return;
        };

        let Some(ConstantValue::Constant(c)) = info.values.get(local_id) else {
            return;
        };

        *operand = MirOperand::Constant(c.clone());
    }

    /// 消除不可达基本块
    fn eliminate_unreachable_blocks(
        &self,
        func: &mut MirFunction,
        block_info: &HashMap<usize, BlockConstInfo>,
    ) {
        // 如果入口块未被处理，说明函数不可达（但这不可能发生）
        if func.blocks.is_empty() {
            return;
        }

        let entry_id = func.blocks[0].id;
        if !block_info
            .get(&entry_id)
            .map(|info| info.processed)
            .unwrap_or(false)
        {
            // 整个函数不可达，清空所有块
            func.blocks.clear();
            return;
        }

        // 收集所有可达块的ID
        let reachable: HashSet<usize> = block_info
            .iter()
            .filter(|(_, info)| info.processed)
            .map(|(id, _)| *id)
            .collect();

        // 处理条件分支，如果条件是常量，直接跳转到目标
        for block in &mut func.blocks {
            if !reachable.contains(&block.id) {
                continue;
            }

            if let MirTerminator::CondBranch {
                cond,
                then_block,
                else_block,
            } = &mut block.terminator
            {
                if let MirOperand::Constant(MirConstant::Bool(true)) = cond {
                    // 条件恒真，替换为无条件跳转到 then
                    block.terminator = MirTerminator::Branch {
                        target: *then_block,
                    };
                } else if let MirOperand::Constant(MirConstant::Bool(false)) = cond {
                    // 条件恒假，替换为无条件跳转到 else
                    block.terminator = MirTerminator::Branch {
                        target: *else_block,
                    };
                }
            }
        }

        // 从后向前遍历，找到所有实际上不可达的块
        // 即使被标记为 processed，如果没有前驱能到达它，它还是不可达
        // 这里简化处理：只移除从未被处理的块
        func.blocks.retain(|block| reachable.contains(&block.id));
    }
}

impl Default for ConstantPropagation {
    fn default() -> Self {
        Self::new()
    }
}

/// 便捷函数：对模块执行常量传播
pub fn constant_propagation(module: &mut MirModule) {
    let mut cp = ConstantPropagation::new();
    cp.run_on_module(module);
}
