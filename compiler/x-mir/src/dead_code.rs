//! 死代码消除（Dead Code Elimination）
//!
//! 该模块实现死代码消除优化：
//! 1. 消除从未被使用的局部变量定义
//! 2. 消除不可达的基本块
//! 3. 消除没有副作用且结果未被使用的指令

use crate::{
    MirBasicBlock, MirFunction, MirInstruction, MirLocalId, MirModule, MirOperand, MirTerminator,
};
use std::collections::HashSet;

/// 死代码消除优化器
pub struct DeadCodeElimination;

impl DeadCodeElimination {
    pub fn new() -> Self {
        Self
    }

    /// 对整个模块执行死代码消除
    pub fn run_on_module(&mut self, module: &mut MirModule) {
        for func in &mut module.functions {
            if !func.is_extern {
                self.run_on_function(func);
            }
        }
    }

    /// 对单个函数执行死代码消除
    pub fn run_on_function(&self, func: &mut MirFunction) {
        // 第一步：找出所有被使用的局部变量
        let used_locals = self.find_used_locals(func);

        // 第二步：从入口开始找出所有可达的基本块
        let reachable_blocks = self.find_reachable_blocks(func);

        // 第三步：消除不可达基本块
        func.blocks
            .retain(|block| reachable_blocks.contains(&block.id));

        // 第四步：对每个可达块，消除结果未被使用且无副作用的指令
        for block in &mut func.blocks {
            self.eliminate_dead_instructions(block, &used_locals);
        }

        // 第五步：从 func.locals 中移除未使用的局部变量
        func.locals.retain(|id, _| used_locals.contains(id));
    }

    /// 找出所有被使用的局部变量
    fn find_used_locals(&self, func: &MirFunction) -> HashSet<MirLocalId> {
        let mut used = HashSet::new();

        // 参数总是被使用（它们是函数入口）
        // 不过参数在 MIR 中是通过 Param 操作数访问，不占用 locals
        // locals 只包含函数内分配的局部变量

        for block in &func.blocks {
            // 检查指令中对局部变量的使用
            for instr in &block.instructions {
                self.collect_used_locals_in_instr(instr, &mut used);
            }

            // 检查终止指令中对局部变量的使用
            self.collect_used_locals_in_terminator(&block.terminator, &mut used);
        }

        used
    }

    /// 从指令中收集被使用的局部变量
    fn collect_used_locals_in_instr(&self, instr: &MirInstruction, used: &mut HashSet<MirLocalId>) {
        match instr {
            MirInstruction::Assign { value, .. } => {
                self.collect_used_locals_in_operand(value, used);
            }
            MirInstruction::BinaryOp { left, right, .. } => {
                self.collect_used_locals_in_operand(left, used);
                self.collect_used_locals_in_operand(right, used);
            }
            MirInstruction::UnaryOp { operand, .. } => {
                self.collect_used_locals_in_operand(operand, used);
            }
            MirInstruction::Call { func, args, .. } => {
                self.collect_used_locals_in_operand(func, used);
                for arg in args {
                    self.collect_used_locals_in_operand(arg, used);
                }
            }
            MirInstruction::FieldAccess { object, .. } => {
                self.collect_used_locals_in_operand(object, used);
            }
            MirInstruction::ArrayAccess { array, index, .. } => {
                self.collect_used_locals_in_operand(array, used);
                self.collect_used_locals_in_operand(index, used);
            }
            MirInstruction::Alloc { .. } => {
                // Alloc 定义一个新局部变量，这里不收集它自己
            }
            MirInstruction::Load { ptr, .. } => {
                self.collect_used_locals_in_operand(ptr, used);
            }
            MirInstruction::Store { ptr, value, .. } => {
                self.collect_used_locals_in_operand(ptr, used);
                self.collect_used_locals_in_operand(value, used);
            }
            MirInstruction::SetField { object, value, .. } => {
                self.collect_used_locals_in_operand(object, used);
                self.collect_used_locals_in_operand(value, used);
            }
            MirInstruction::Cast { value, .. } => {
                self.collect_used_locals_in_operand(value, used);
            }
            MirInstruction::Dup { src, .. } => {
                self.collect_used_locals_in_operand(src, used);
            }
            MirInstruction::Drop { value } => {
                self.collect_used_locals_in_operand(value, used);
            }
            MirInstruction::Reuse { src, .. } => {
                self.collect_used_locals_in_operand(src, used);
            }
            MirInstruction::WhenGuard {
                condition, body, ..
            } => {
                self.collect_used_locals_in_operand(condition, used);
                self.collect_used_locals_in_operand(body, used);
            }
        }
    }

    /// 从终止指令收集被使用的局部变量
    fn collect_used_locals_in_terminator(
        &self,
        term: &MirTerminator,
        used: &mut HashSet<MirLocalId>,
    ) {
        match term {
            MirTerminator::Branch { .. } => {}
            MirTerminator::CondBranch { cond, .. } => {
                self.collect_used_locals_in_operand(cond, used);
            }
            MirTerminator::Return { value } => {
                if let Some(value) = value {
                    self.collect_used_locals_in_operand(value, used);
                }
            }
            MirTerminator::Unreachable => {}
            MirTerminator::Switch { value, .. } => {
                self.collect_used_locals_in_operand(value, used);
            }
        }
    }

    /// 从操作数收集被使用的局部变量
    fn collect_used_locals_in_operand(&self, operand: &MirOperand, used: &mut HashSet<MirLocalId>) {
        match operand {
            MirOperand::Local(id) => {
                used.insert(*id);
            }
            MirOperand::Constant(_) => {}
            MirOperand::Param(_) => {}
            MirOperand::Global(_) => {}
        }
    }

    /// 找出从入口可达的所有基本块
    fn find_reachable_blocks(&self, func: &MirFunction) -> HashSet<usize> {
        let mut reachable = HashSet::new();
        let mut work_list = Vec::new();

        // 从入口块开始
        if let Some(entry) = func.blocks.first() {
            work_list.push(entry.id);
            reachable.insert(entry.id);
        }

        while let Some(block_id) = work_list.pop() {
            let Some(block) = func.blocks.iter().find(|b| b.id == block_id) else {
                continue;
            };

            let successors = self.get_successors(block);
            for succ_id in successors {
                if !reachable.contains(&succ_id) {
                    reachable.insert(succ_id);
                    work_list.push(succ_id);
                }
            }
        }

        reachable
    }

    /// 获取基本块的后继
    fn get_successors(&self, block: &MirBasicBlock) -> Vec<usize> {
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

    /// 消除块中结果未被使用且无副作用的指令
    fn eliminate_dead_instructions(
        &self,
        block: &mut MirBasicBlock,
        used_locals: &HashSet<MirLocalId>,
    ) {
        let mut new_instructions = Vec::new();

        for instr in block.instructions.drain(..) {
            if !self.is_dead_instruction(&instr, used_locals) {
                new_instructions.push(instr);
            }
        }

        block.instructions = new_instructions;
    }

    /// 判断一条指令是否是死代码（可以安全消除）
    ///
    /// 死代码的条件：
    /// 1. 指令定义了一个局部变量（有 dest）
    /// 2. 该局部变量从未被使用
    /// 3. 指令没有副作用（不会修改内存、不会调用函数等）
    fn is_dead_instruction(
        &self,
        instr: &MirInstruction,
        used_locals: &HashSet<MirLocalId>,
    ) -> bool {
        match instr {
            // 定义了一个局部变量，且该变量未被使用，且操作本身没有副作用
            MirInstruction::Assign { dest, .. }
            | MirInstruction::BinaryOp { dest, .. }
            | MirInstruction::UnaryOp { dest, .. }
            | MirInstruction::FieldAccess { dest, .. }
            | MirInstruction::ArrayAccess { dest, .. }
            | MirInstruction::Alloc { dest, .. }
            | MirInstruction::Load { dest, .. }
            | MirInstruction::Cast { dest, .. }
            | MirInstruction::Dup { dest, .. }
            | MirInstruction::Reuse { dest, .. } => !used_locals.contains(dest),

            // Call 总是可能有副作用，不消除
            MirInstruction::Call { .. } => false,

            // Store 有副作用（修改内存），不消除
            MirInstruction::Store { .. } => false,

            // SetField 修改对象字段，有副作用，不消除
            MirInstruction::SetField { .. } => false,

            // Drop 释放引用，有副作用，不消除
            MirInstruction::Drop { .. } => false,

            // WhenGuard 有控制流副作用，不消除
            MirInstruction::WhenGuard { .. } => false,
        }
    }
}

impl Default for DeadCodeElimination {
    fn default() -> Self {
        Self::new()
    }
}

/// 便捷函数：对模块执行死代码消除
pub fn dead_code_elimination(module: &mut MirModule) {
    let mut dce = DeadCodeElimination::new();
    dce.run_on_module(module);
}
