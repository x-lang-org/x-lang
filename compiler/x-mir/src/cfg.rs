//! 控制流图（CFG）构建与分析
//!
//! 该模块提供从 MIR 构建控制流图的完整基础设施，包括：
//! - `BasicBlock`：扩展的基本块结构（含前驱/后继信息）
//! - `Cfg`：完整的控制流图表示
//! - `CfgBuilder`：从 HIR/MIR 构建 CFG 的构建器
//! - CFG 验证和可视化

use crate::mir::*;
use std::collections::{HashMap, HashSet};
use x_hir::{
    HirBinaryOp, HirBlock, HirExpression, HirFunctionDecl, HirLiteral, HirStatement, HirType,
};

/// 扩展的基本块（含控制流信息）
#[derive(Debug, Clone)]
pub struct BasicBlock {
    /// 块 ID
    pub id: MirBlockId,
    /// 指令列表
    pub instructions: Vec<MirInstruction>,
    /// 终止指令
    pub terminator: MirTerminator,
    /// 前驱块 ID 列表
    pub predecessors: HashSet<MirBlockId>,
    /// 后继块 ID 列表
    pub successors: HashSet<MirBlockId>,
}

impl Default for BasicBlock {
    fn default() -> Self {
        Self {
            id: 0,
            instructions: Vec::new(),
            terminator: MirTerminator::Unreachable,
            predecessors: HashSet::new(),
            successors: HashSet::new(),
        }
    }
}

impl BasicBlock {
    pub fn new(id: MirBlockId) -> Self {
        Self {
            id,
            ..Default::default()
        }
    }

    pub fn is_entry(&self) -> bool {
        self.predecessors.is_empty()
    }

    pub fn is_exit(&self) -> bool {
        matches!(
            self.terminator,
            MirTerminator::Return { .. } | MirTerminator::Unreachable
        )
    }

    pub fn add_instruction(&mut self, instr: MirInstruction) {
        self.instructions.push(instr);
    }

    pub fn set_terminator(&mut self, term: MirTerminator) {
        self.terminator = term;
    }
}

/// 控制流图
#[derive(Debug, Clone)]
pub struct Cfg {
    /// 基本块集合（ID -> BasicBlock）
    pub blocks: HashMap<MirBlockId, BasicBlock>,
    /// 入口块 ID
    pub entry_block: MirBlockId,
    /// 出口块 ID 列表
    pub exit_blocks: Vec<MirBlockId>,
    /// 块 ID 计数器
    next_block_id: MirBlockId,
}

impl Default for Cfg {
    fn default() -> Self {
        Self::new()
    }
}

impl Cfg {
    pub fn new() -> Self {
        let mut cfg = Self {
            blocks: HashMap::new(),
            entry_block: 0,
            exit_blocks: Vec::new(),
            next_block_id: 0,
        };
        let entry = cfg.create_block();
        cfg.entry_block = entry;
        cfg
    }

    pub fn create_block(&mut self) -> MirBlockId {
        let id = self.next_block_id;
        self.next_block_id += 1;
        self.blocks.insert(id, BasicBlock::new(id));
        id
    }

    pub fn get_block(&self, id: MirBlockId) -> Option<&BasicBlock> {
        self.blocks.get(&id)
    }

    pub fn get_block_mut(&mut self, id: MirBlockId) -> Option<&mut BasicBlock> {
        self.blocks.get_mut(&id)
    }

    pub fn set_entry(&mut self, id: MirBlockId) {
        self.entry_block = id;
    }

    pub fn add_exit(&mut self, id: MirBlockId) {
        if !self.exit_blocks.contains(&id) {
            self.exit_blocks.push(id);
        }
    }

    pub fn add_edge(&mut self, from: MirBlockId, to: MirBlockId) {
        if let Some(from_block) = self.blocks.get_mut(&from) {
            from_block.successors.insert(to);
        }
        if let Some(to_block) = self.blocks.get_mut(&to) {
            to_block.predecessors.insert(from);
        }
    }

    pub fn remove_block(&mut self, id: MirBlockId) {
        if let Some(block) = self.blocks.remove(&id) {
            for pred in &block.predecessors {
                if let Some(pred_block) = self.blocks.get_mut(pred) {
                    pred_block.successors.remove(&id);
                }
            }
            for succ in &block.successors {
                if let Some(succ_block) = self.blocks.get_mut(succ) {
                    succ_block.predecessors.remove(&id);
                }
            }
        }
        self.exit_blocks.retain(|&exit| exit != id);
    }

    pub fn block_count(&self) -> usize {
        self.blocks.len()
    }

    pub fn edge_count(&self) -> usize {
        self.blocks.values().map(|b| b.successors.len()).sum()
    }

    pub fn predecessors(&self, id: MirBlockId) -> Vec<MirBlockId> {
        self.blocks
            .get(&id)
            .map(|b| b.predecessors.iter().copied().collect())
            .unwrap_or_default()
    }

    pub fn successors(&self, id: MirBlockId) -> Vec<MirBlockId> {
        self.blocks
            .get(&id)
            .map(|b| b.successors.iter().copied().collect())
            .unwrap_or_default()
    }

    pub fn validate(&self) -> Result<(), CfgError> {
        if !self.blocks.contains_key(&self.entry_block) {
            return Err(CfgError::InvalidEntry(self.entry_block));
        }

        for exit in &self.exit_blocks {
            if !self.blocks.contains_key(exit) {
                return Err(CfgError::InvalidExit(*exit));
            }
        }

        for (id, block) in &self.blocks {
            for succ in &block.successors {
                if !self.blocks.contains_key(succ) {
                    return Err(CfgError::MissingSuccessor(*id, *succ));
                }
            }
            for pred in &block.predecessors {
                if !self.blocks.contains_key(pred) {
                    return Err(CfgError::MissingPredecessor(*id, *pred));
                }
            }
        }

        Ok(())
    }

    pub fn to_dot(&self) -> String {
        let mut output = String::from("digraph CFG {\n");
        output.push_str("  node [shape=box];\n");

        for (id, block) in &self.blocks {
            let label = format!(
                "Block {}\\n{} instructions\\n{:?}",
                id,
                block.instructions.len(),
                block.terminator
            );
            let shape = if self.exit_blocks.contains(id) {
                "doubleoctagon"
            } else if *id == self.entry_block {
                "ellipse"
            } else {
                "box"
            };
            output.push_str(&format!(
                "  {} [label=\"{}\", shape={}];\n",
                id, label, shape
            ));
        }

        for (id, block) in &self.blocks {
            for succ in &block.successors {
                output.push_str(&format!("  {} -> {};\n", id, succ));
            }
        }

        output.push_str("}\n");
        output
    }

    pub fn depth_first_order(&self) -> Vec<MirBlockId> {
        let mut visited = HashSet::new();
        let mut order = Vec::new();
        self.dfs(self.entry_block, &mut visited, &mut order);
        order
    }

    fn dfs(
        &self,
        block_id: MirBlockId,
        visited: &mut HashSet<MirBlockId>,
        order: &mut Vec<MirBlockId>,
    ) {
        if visited.contains(&block_id) {
            return;
        }
        visited.insert(block_id);
        if let Some(block) = self.get_block(block_id) {
            for succ in &block.successors {
                self.dfs(*succ, visited, order);
            }
        }
        order.push(block_id);
    }

    pub fn reverse_postorder(&self) -> Vec<MirBlockId> {
        let mut order = self.depth_first_order();
        order.reverse();
        order
    }

    pub fn compute_dominators(&self) -> HashMap<MirBlockId, HashSet<MirBlockId>> {
        let mut dominators: HashMap<MirBlockId, HashSet<MirBlockId>> = HashMap::new();

        dominators.insert(self.entry_block, HashSet::from([self.entry_block]));

        let all_blocks: HashSet<MirBlockId> = self.blocks.keys().copied().collect();

        for &block_id in self.blocks.keys() {
            if block_id != self.entry_block {
                dominators.insert(block_id, all_blocks.clone());
            }
        }

        let mut changed = true;
        while changed {
            changed = false;
            for &block_id in self.blocks.keys() {
                if block_id == self.entry_block {
                    continue;
                }

                let preds: Vec<MirBlockId> = self.predecessors(block_id);
                if preds.is_empty() {
                    continue;
                }

                let mut new_dom: HashSet<MirBlockId> = all_blocks.clone();
                for pred in preds {
                    if let Some(pred_dom) = dominators.get(&pred) {
                        new_dom = new_dom.intersection(pred_dom).copied().collect();
                    }
                }
                new_dom.insert(block_id);

                if dominators.get(&block_id) != Some(&new_dom) {
                    dominators.insert(block_id, new_dom);
                    changed = true;
                }
            }
        }

        dominators
    }

    pub fn compute_immediate_dominators(&self) -> HashMap<MirBlockId, MirBlockId> {
        let dominators = self.compute_dominators();
        let mut idom: HashMap<MirBlockId, MirBlockId> = HashMap::new();

        for (&block_id, dom_set) in &dominators {
            if block_id == self.entry_block {
                continue;
            }

            for &dom in dom_set {
                if dom == block_id {
                    continue;
                }
                let mut is_idom = true;
                for &other in dom_set {
                    if other != block_id && other != dom {
                        if let Some(other_dom) = dominators.get(&other) {
                            if other_dom.contains(&dom) {
                                is_idom = false;
                                break;
                            }
                        }
                    }
                }
                if is_idom {
                    idom.insert(block_id, dom);
                    break;
                }
            }
        }

        idom
    }

    pub fn compute_dominance_frontier(&self) -> HashMap<MirBlockId, HashSet<MirBlockId>> {
        let idom = self.compute_immediate_dominators();
        let mut df: HashMap<MirBlockId, HashSet<MirBlockId>> = HashMap::new();

        for block_id in self.blocks.keys() {
            df.insert(*block_id, HashSet::new());
        }

        for &block_id in self.blocks.keys() {
            let preds = self.predecessors(block_id);
            if preds.len() < 2 {
                continue;
            }

            for pred in preds {
                let mut runner = pred;
                loop {
                    if runner != idom.get(&block_id).copied().unwrap_or(runner) {
                        if let Some(df_set) = df.get_mut(&runner) {
                            df_set.insert(block_id);
                        }
                    }
                    if let Some(&idom_runner) = idom.get(&runner) {
                        runner = idom_runner;
                    } else {
                        break;
                    }
                    if runner == idom.get(&block_id).copied().unwrap_or(runner) {
                        break;
                    }
                }
            }
        }

        df
    }
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum CfgError {
    #[error("Invalid entry block: {0}")]
    InvalidEntry(MirBlockId),

    #[error("Invalid exit block: {0}")]
    InvalidExit(MirBlockId),

    #[error("Block {0} has missing successor {1}")]
    MissingSuccessor(MirBlockId, MirBlockId),

    #[error("Block {0} has missing predecessor {1}")]
    MissingPredecessor(MirBlockId, MirBlockId),

    #[error("Undefined variable: {0}")]
    UndefinedVariable(String),

    #[error("Unsupported feature: {0}")]
    UnsupportedFeature(String),
}

pub type CfgResult<T> = Result<T, CfgError>;

pub struct CfgBuilder {
    cfg: Cfg,
    current_block: Option<MirBlockId>,
    local_counter: MirLocalId,
    locals: HashMap<MirLocalId, MirType>,
    name_to_local: HashMap<String, MirLocalId>,
    scopes: Vec<HashMap<String, MirLocalId>>,
    loop_stack: Vec<(MirBlockId, MirBlockId)>,
    break_targets: Vec<MirBlockId>,
    continue_targets: Vec<MirBlockId>,
}

impl CfgBuilder {
    pub fn new() -> Self {
        let mut cfg = Cfg::new();
        let entry = *cfg.blocks.keys().next().unwrap();
        cfg.set_entry(entry);

        Self {
            cfg,
            current_block: Some(entry),
            local_counter: 0,
            locals: HashMap::new(),
            name_to_local: HashMap::new(),
            scopes: vec![HashMap::new()],
            loop_stack: Vec::new(),
            break_targets: Vec::new(),
            continue_targets: Vec::new(),
        }
    }

    pub fn build(mut self) -> Cfg {
        self.update_exit_blocks();
        self.cfg
    }

    fn update_exit_blocks(&mut self) {
        self.cfg.exit_blocks.clear();
        let exit_ids: Vec<MirBlockId> = self
            .cfg
            .blocks
            .iter()
            .filter(|(_, block)| block.is_exit())
            .map(|(&id, _)| id)
            .collect();
        for id in exit_ids {
            self.cfg.add_exit(id);
        }
    }

    pub fn create_block(&mut self) -> MirBlockId {
        self.cfg.create_block()
    }

    pub fn switch_to_block(&mut self, block_id: MirBlockId) {
        self.current_block = Some(block_id);
    }

    pub fn current_block(&self) -> Option<MirBlockId> {
        self.current_block
    }

    pub fn new_local(&mut self, ty: MirType) -> MirLocalId {
        let id = self.local_counter;
        self.local_counter += 1;
        self.locals.insert(id, ty);
        id
    }

    pub fn bind_local(&mut self, name: String, local: MirLocalId) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.clone(), local);
        }
        self.name_to_local.insert(name, local);
    }

    pub fn lookup_local(&self, name: &str) -> Option<MirLocalId> {
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get(name).copied())
    }

    pub fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    pub fn push_instruction(&mut self, instr: MirInstruction) {
        if let Some(block_id) = self.current_block {
            if let Some(block) = self.cfg.get_block_mut(block_id) {
                block.add_instruction(instr);
            }
        }
    }

    pub fn set_terminator(&mut self, term: MirTerminator) {
        if let Some(block_id) = self.current_block {
            if let Some(block) = self.cfg.get_block_mut(block_id) {
                block.set_terminator(term.clone());
            }
            let successors: Vec<MirBlockId> = self
                .cfg
                .get_block(block_id)
                .map(|b| b.successors.iter().copied().collect())
                .unwrap_or_default();
            for succ in successors {
                if let Some(succ_block) = self.cfg.get_block_mut(succ) {
                    succ_block.predecessors.insert(block_id);
                }
            }
        }
    }

    pub fn branch(&mut self, target: MirBlockId) {
        if let Some(block_id) = self.current_block {
            self.cfg.add_edge(block_id, target);
            if let Some(block) = self.cfg.get_block_mut(block_id) {
                block.set_terminator(MirTerminator::Branch { target });
            }
        }
    }

    pub fn cond_branch(
        &mut self,
        cond: MirOperand,
        then_block: MirBlockId,
        else_block: MirBlockId,
    ) {
        if let Some(block_id) = self.current_block {
            self.cfg.add_edge(block_id, then_block);
            self.cfg.add_edge(block_id, else_block);
            if let Some(block) = self.cfg.get_block_mut(block_id) {
                block.set_terminator(MirTerminator::CondBranch {
                    cond,
                    then_block,
                    else_block,
                });
            }
        }
    }

    pub fn return_value(&mut self, value: Option<MirOperand>) {
        if let Some(block_id) = self.current_block {
            if let Some(block) = self.cfg.get_block_mut(block_id) {
                block.set_terminator(MirTerminator::Return { value });
            }
        }
    }

    pub fn build_if(
        &mut self,
        cond: MirOperand,
        then_block: MirBlockId,
        else_block: MirBlockId,
        merge_block: MirBlockId,
    ) {
        self.cond_branch(cond, then_block, else_block);

        self.switch_to_block(then_block);
        self.branch(merge_block);

        self.switch_to_block(else_block);
        self.branch(merge_block);

        self.switch_to_block(merge_block);
    }

    pub fn build_while(
        &mut self,
        header_block: MirBlockId,
        body_block: MirBlockId,
        exit_block: MirBlockId,
        cond: MirOperand,
    ) {
        self.cfg.add_edge(header_block, body_block);
        self.cfg.add_edge(header_block, exit_block);
        if let Some(block) = self.cfg.get_block_mut(header_block) {
            block.set_terminator(MirTerminator::CondBranch {
                cond,
                then_block: body_block,
                else_block: exit_block,
            });
        }

        self.cfg.add_edge(body_block, header_block);
        if let Some(block) = self.cfg.get_block_mut(body_block) {
            block.set_terminator(MirTerminator::Branch {
                target: header_block,
            });
        }

        self.loop_stack.push((header_block, exit_block));
    }

    pub fn build_for(
        &mut self,
        init_block: MirBlockId,
        cond_block: MirBlockId,
        body_block: MirBlockId,
        update_block: MirBlockId,
        exit_block: MirBlockId,
        cond: MirOperand,
    ) {
        self.cfg.add_edge(init_block, cond_block);
        if let Some(block) = self.cfg.get_block_mut(init_block) {
            block.set_terminator(MirTerminator::Branch { target: cond_block });
        }

        self.cfg.add_edge(cond_block, body_block);
        self.cfg.add_edge(cond_block, exit_block);
        if let Some(block) = self.cfg.get_block_mut(cond_block) {
            block.set_terminator(MirTerminator::CondBranch {
                cond,
                then_block: body_block,
                else_block: exit_block,
            });
        }

        self.cfg.add_edge(body_block, update_block);
        if let Some(block) = self.cfg.get_block_mut(body_block) {
            block.set_terminator(MirTerminator::Branch {
                target: update_block,
            });
        }

        self.cfg.add_edge(update_block, cond_block);
        if let Some(block) = self.cfg.get_block_mut(update_block) {
            block.set_terminator(MirTerminator::Branch { target: cond_block });
        }

        self.loop_stack.push((cond_block, exit_block));
    }

    pub fn build_match(
        &mut self,
        discriminant_block: MirBlockId,
        case_blocks: Vec<MirBlockId>,
        default_block: MirBlockId,
        merge_block: MirBlockId,
        discriminant: MirOperand,
        cases: Vec<(MirConstant, MirBlockId)>,
    ) {
        if let Some(block) = self.cfg.get_block_mut(discriminant_block) {
            block.set_terminator(MirTerminator::Switch {
                value: discriminant,
                cases,
                default: default_block,
            });
        }

        for case_block in &case_blocks {
            self.cfg.add_edge(discriminant_block, *case_block);
            self.cfg.add_edge(*case_block, merge_block);
            if let Some(block) = self.cfg.get_block_mut(*case_block) {
                block.set_terminator(MirTerminator::Branch {
                    target: merge_block,
                });
            }
        }

        self.cfg.add_edge(discriminant_block, default_block);
        self.cfg.add_edge(default_block, merge_block);
        if let Some(block) = self.cfg.get_block_mut(default_block) {
            block.set_terminator(MirTerminator::Branch {
                target: merge_block,
            });
        }
    }

    pub fn handle_break(&mut self) {
        if let Some(&exit_block) = self.loop_stack.last().map(|(_, exit)| exit) {
            self.branch(exit_block);
        }
    }

    pub fn handle_continue(&mut self) {
        if let Some(&(header_block, _)) = self.loop_stack.last() {
            self.branch(header_block);
        }
    }
}

impl Default for CfgBuilder {
    fn default() -> Self {
        Self::new()
    }
}

pub fn build_cfg_from_hir_function(func: &HirFunctionDecl) -> CfgResult<Cfg> {
    let mut builder = CfgBuilder::new();

    for param in &func.parameters {
        let local = builder.new_local(lower_hir_type_to_mir(&param.ty));
        builder.bind_local(param.name.clone(), local);
    }

    builder.lower_hir_block(&func.body)?;

    if let Some(current) = builder.current_block() {
        if let Some(block) = builder.cfg.get_block(current) {
            if matches!(block.terminator, MirTerminator::Unreachable) {
                builder.return_value(None);
            }
        }
    }

    Ok(builder.build())
}

impl CfgBuilder {
    fn lower_hir_block(&mut self, block: &HirBlock) -> CfgResult<()> {
        self.push_scope();

        for stmt in &block.statements {
            self.lower_hir_statement(stmt)?;
        }

        self.pop_scope();
        Ok(())
    }

    fn lower_hir_statement(&mut self, stmt: &HirStatement) -> CfgResult<()> {
        match stmt {
            HirStatement::Expression(expr) => {
                self.lower_hir_expression(expr)?;
            }
            HirStatement::Variable(var) => {
                let local = self.new_local(lower_hir_type_to_mir(&var.ty));
                self.bind_local(var.name.clone(), local);

                if let Some(init) = &var.initializer {
                    let value = self.lower_hir_expression(init)?;
                    self.push_instruction(MirInstruction::Assign { dest: local, value });
                }
            }
            HirStatement::Return(expr) => {
                let value = expr
                    .as_ref()
                    .map(|e| self.lower_hir_expression(e))
                    .transpose()?;
                self.return_value(value);
            }
            HirStatement::If(if_stmt) => {
                let cond = self.lower_hir_expression(&if_stmt.condition)?;

                let then_block = self.create_block();
                let else_block = self.create_block();
                let merge_block = self.create_block();

                let current = self
                    .current_block()
                    .ok_or_else(|| CfgError::UnsupportedFeature("No current block".to_string()))?;

                self.cfg.add_edge(current, then_block);
                self.cfg.add_edge(current, else_block);
                if let Some(block) = self.cfg.get_block_mut(current) {
                    block.set_terminator(MirTerminator::CondBranch {
                        cond,
                        then_block,
                        else_block,
                    });
                }

                self.switch_to_block(then_block);
                self.lower_hir_block(&if_stmt.then_block)?;
                if let Some(current) = self.current_block() {
                    if !matches!(
                        self.cfg.get_block(current).map(|b| &b.terminator),
                        Some(MirTerminator::Return { .. })
                    ) {
                        self.branch(merge_block);
                    }
                }

                self.switch_to_block(else_block);
                if let Some(else_body) = &if_stmt.else_block {
                    self.lower_hir_block(else_body)?;
                }
                if let Some(current) = self.current_block() {
                    if !matches!(
                        self.cfg.get_block(current).map(|b| &b.terminator),
                        Some(MirTerminator::Return { .. })
                    ) {
                        self.branch(merge_block);
                    }
                }

                self.switch_to_block(merge_block);
            }
            HirStatement::While(while_stmt) => {
                let header_block = self.create_block();
                let body_block = self.create_block();
                let exit_block = self.create_block();

                if let Some(current) = self.current_block() {
                    self.cfg.add_edge(current, header_block);
                    if let Some(block) = self.cfg.get_block_mut(current) {
                        block.set_terminator(MirTerminator::Branch {
                            target: header_block,
                        });
                    }
                }

                self.switch_to_block(header_block);
                let cond = self.lower_hir_expression(&while_stmt.condition)?;
                self.cfg.add_edge(header_block, body_block);
                self.cfg.add_edge(header_block, exit_block);
                if let Some(block) = self.cfg.get_block_mut(header_block) {
                    block.set_terminator(MirTerminator::CondBranch {
                        cond,
                        then_block: body_block,
                        else_block: exit_block,
                    });
                }

                self.loop_stack.push((header_block, exit_block));

                self.switch_to_block(body_block);
                self.lower_hir_block(&while_stmt.body)?;
                if let Some(current) = self.current_block() {
                    if !matches!(
                        self.cfg.get_block(current).map(|b| &b.terminator),
                        Some(MirTerminator::Return { .. })
                    ) {
                        self.branch(header_block);
                    }
                }

                self.loop_stack.pop();

                self.switch_to_block(exit_block);
            }
            HirStatement::For(for_stmt) => {
                let init_block = self.create_block();
                let cond_block = self.create_block();
                let body_block = self.create_block();
                let update_block = self.create_block();
                let exit_block = self.create_block();

                if let Some(current) = self.current_block() {
                    self.cfg.add_edge(current, init_block);
                    if let Some(block) = self.cfg.get_block_mut(current) {
                        block.set_terminator(MirTerminator::Branch { target: init_block });
                    }
                }

                self.switch_to_block(init_block);
                if let x_hir::HirPattern::Variable(name) = &for_stmt.pattern {
                    let local = self.new_local(MirType::Unknown);
                    self.bind_local(name.clone(), local);
                }
                let _ = self.lower_hir_expression(&for_stmt.iterator)?;
                self.branch(cond_block);

                self.switch_to_block(cond_block);
                let cond = self.lower_hir_expression(&for_stmt.iterator)?;
                self.cond_branch(cond, body_block, exit_block);

                self.loop_stack.push((cond_block, exit_block));

                self.switch_to_block(body_block);
                self.lower_hir_block(&for_stmt.body)?;
                if let Some(current) = self.current_block() {
                    if !matches!(
                        self.cfg.get_block(current).map(|b| &b.terminator),
                        Some(MirTerminator::Return { .. })
                    ) {
                        self.branch(update_block);
                    }
                }

                self.switch_to_block(update_block);
                self.branch(cond_block);

                self.loop_stack.pop();

                self.switch_to_block(exit_block);
            }
            HirStatement::Match(match_stmt) => {
                let discr = self.lower_hir_expression(&match_stmt.expression)?;

                let discr_block = self
                    .current_block()
                    .ok_or_else(|| CfgError::UnsupportedFeature("No current block".to_string()))?;

                let mut case_blocks = Vec::new();
                let merge_block = self.create_block();

                let mut cases = Vec::new();
                for case in &match_stmt.cases {
                    let case_block = self.create_block();
                    case_blocks.push(case_block);
                    self.cfg.add_edge(discr_block, case_block);

                    if let x_hir::HirPattern::Literal(lit) = &case.pattern {
                        if let Some(constant) = hir_literal_to_constant(lit) {
                            cases.push((constant, case_block));
                        }
                    }
                }

                let default_block = if cases.len() < match_stmt.cases.len() {
                    let default = self.create_block();
                    self.cfg.add_edge(discr_block, default);
                    Some(default)
                } else {
                    None
                };

                if let Some(block) = self.cfg.get_block_mut(discr_block) {
                    block.set_terminator(MirTerminator::Switch {
                        value: discr,
                        cases: cases.clone(),
                        default: default_block.unwrap_or(merge_block),
                    });
                }

                for (i, case) in match_stmt.cases.iter().enumerate() {
                    if i < case_blocks.len() {
                        self.switch_to_block(case_blocks[i]);
                        self.lower_hir_block(&case.body)?;
                        if let Some(current) = self.current_block() {
                            if !matches!(
                                self.cfg.get_block(current).map(|b| &b.terminator),
                                Some(MirTerminator::Return { .. })
                            ) {
                                self.branch(merge_block);
                            }
                        }
                    }
                }

                if let Some(default) = default_block {
                    self.switch_to_block(default);
                    self.branch(merge_block);
                }

                self.switch_to_block(merge_block);
            }
            HirStatement::Loop(body) => {
                let header_block = self.create_block();
                let exit_block = self.create_block();

                if let Some(current) = self.current_block() {
                    self.cfg.add_edge(current, header_block);
                    if let Some(block) = self.cfg.get_block_mut(current) {
                        block.set_terminator(MirTerminator::Branch {
                            target: header_block,
                        });
                    }
                }

                self.loop_stack.push((header_block, exit_block));

                self.switch_to_block(header_block);
                self.lower_hir_block(body)?;
                if let Some(current) = self.current_block() {
                    if !matches!(
                        self.cfg.get_block(current).map(|b| &b.terminator),
                        Some(MirTerminator::Return { .. })
                    ) {
                        self.branch(header_block);
                    }
                }

                self.loop_stack.pop();

                self.switch_to_block(exit_block);
            }
            HirStatement::Break => {
                self.handle_break();
            }
            HirStatement::Continue => {
                self.handle_continue();
            }
            HirStatement::Try(_)
            | HirStatement::Unsafe(_)
            | HirStatement::Defer(_)
            | HirStatement::Yield(_) => {
                return Err(CfgError::UnsupportedFeature(format!("{:?}", stmt)));
            }
        }
        Ok(())
    }

    fn lower_hir_expression(&mut self, expr: &HirExpression) -> CfgResult<MirOperand> {
        match expr {
            HirExpression::Literal(lit) => Ok(MirOperand::Constant(
                hir_literal_to_constant(lit).unwrap_or(MirConstant::Unit),
            )),
            HirExpression::Variable(name) => {
                if let Some(local) = self.lookup_local(name) {
                    Ok(MirOperand::Local(local))
                } else {
                    Ok(MirOperand::Global(name.clone()))
                }
            }
            HirExpression::Binary(op, lhs, rhs) => {
                let left = self.lower_hir_expression(lhs)?;
                let right = self.lower_hir_expression(rhs)?;
                let dest = self.new_local(MirType::Unknown);

                let mir_op = match op {
                    HirBinaryOp::Add => MirBinOp::Add,
                    HirBinaryOp::Sub => MirBinOp::Sub,
                    HirBinaryOp::Mul => MirBinOp::Mul,
                    HirBinaryOp::Div => MirBinOp::Div,
                    HirBinaryOp::Mod => MirBinOp::Mod,
                    HirBinaryOp::Equal => MirBinOp::Eq,
                    HirBinaryOp::NotEqual => MirBinOp::Ne,
                    HirBinaryOp::Less => MirBinOp::Lt,
                    HirBinaryOp::LessEqual => MirBinOp::Le,
                    HirBinaryOp::Greater => MirBinOp::Gt,
                    HirBinaryOp::GreaterEqual => MirBinOp::Ge,
                    HirBinaryOp::And => MirBinOp::And,
                    HirBinaryOp::Or => MirBinOp::Or,
                    HirBinaryOp::BitAnd => MirBinOp::BitAnd,
                    HirBinaryOp::BitOr => MirBinOp::BitOr,
                    HirBinaryOp::BitXor => MirBinOp::BitXor,
                    HirBinaryOp::LeftShift => MirBinOp::Shl,
                    HirBinaryOp::RightShift => MirBinOp::Shr,
                    _ => return Err(CfgError::UnsupportedFeature(format!("Binary op {:?}", op))),
                };

                self.push_instruction(MirInstruction::BinaryOp {
                    dest,
                    op: mir_op,
                    left,
                    right,
                });
                Ok(MirOperand::Local(dest))
            }
            HirExpression::Unary(op, operand) => {
                let operand_val = self.lower_hir_expression(operand)?;
                let dest = self.new_local(MirType::Unknown);

                let mir_op = match op {
                    x_hir::HirUnaryOp::Negate => MirUnOp::Neg,
                    x_hir::HirUnaryOp::Not => MirUnOp::Not,
                    x_hir::HirUnaryOp::BitNot => MirUnOp::BitNot,
                    _ => return Err(CfgError::UnsupportedFeature(format!("Unary op {:?}", op))),
                };

                self.push_instruction(MirInstruction::UnaryOp {
                    dest,
                    op: mir_op,
                    operand: operand_val,
                });
                Ok(MirOperand::Local(dest))
            }
            HirExpression::Call(callee, args) => {
                let func = self.lower_hir_expression(callee)?;
                let lowered_args = args
                    .iter()
                    .map(|arg| self.lower_hir_expression(arg))
                    .collect::<CfgResult<Vec<_>>>()?;

                let dest = self.new_local(MirType::Unknown);
                self.push_instruction(MirInstruction::Call {
                    dest: Some(dest),
                    func,
                    args: lowered_args,
                });
                Ok(MirOperand::Local(dest))
            }
            HirExpression::Assign(target, value) => {
                let value_op = self.lower_hir_expression(value)?;

                match target.as_ref() {
                    HirExpression::Variable(name) => {
                        if let Some(local) = self.lookup_local(name) {
                            self.push_instruction(MirInstruction::Assign {
                                dest: local,
                                value: value_op.clone(),
                            });
                            Ok(MirOperand::Local(local))
                        } else {
                            self.push_instruction(MirInstruction::Store {
                                ptr: MirOperand::Global(name.clone()),
                                value: value_op.clone(),
                            });
                            Ok(MirOperand::Global(name.clone()))
                        }
                    }
                    _ => Ok(value_op),
                }
            }
            HirExpression::Member(obj, field) => {
                let object = self.lower_hir_expression(obj)?;
                let dest = self.new_local(MirType::Unknown);
                self.push_instruction(MirInstruction::FieldAccess {
                    dest,
                    object,
                    field: field.clone(),
                });
                Ok(MirOperand::Local(dest))
            }
            HirExpression::If(cond, then_expr, else_expr) => {
                let cond_val = self.lower_hir_expression(cond)?;

                let then_block = self.create_block();
                let else_block = self.create_block();
                let merge_block = self.create_block();

                let current = self
                    .current_block()
                    .ok_or_else(|| CfgError::UnsupportedFeature("No current block".to_string()))?;

                self.cfg.add_edge(current, then_block);
                self.cfg.add_edge(current, else_block);
                if let Some(block) = self.cfg.get_block_mut(current) {
                    block.set_terminator(MirTerminator::CondBranch {
                        cond: cond_val,
                        then_block,
                        else_block,
                    });
                }

                self.switch_to_block(then_block);
                let then_val = self.lower_hir_expression(then_expr)?;
                let then_result_local = self.new_local(MirType::Unknown);
                self.push_instruction(MirInstruction::Assign {
                    dest: then_result_local,
                    value: then_val,
                });
                self.branch(merge_block);

                self.switch_to_block(else_block);
                let else_val = self.lower_hir_expression(else_expr)?;
                let else_result_local = self.new_local(MirType::Unknown);
                self.push_instruction(MirInstruction::Assign {
                    dest: else_result_local,
                    value: else_val,
                });
                self.branch(merge_block);

                self.switch_to_block(merge_block);
                Ok(MirOperand::Local(then_result_local))
            }
            HirExpression::Array(items) => {
                let dest = self.new_local(MirType::Array(Box::new(MirType::Unknown), items.len()));
                self.push_instruction(MirInstruction::Alloc {
                    dest,
                    ty: MirType::Array(Box::new(MirType::Unknown), items.len()),
                    size: items.len(),
                });
                for item in items {
                    let _ = self.lower_hir_expression(item)?;
                }
                Ok(MirOperand::Local(dest))
            }
            HirExpression::Tuple(items) => {
                let dest = self.new_local(MirType::Array(Box::new(MirType::Unknown), items.len()));
                self.push_instruction(MirInstruction::Alloc {
                    dest,
                    ty: MirType::Array(Box::new(MirType::Unknown), items.len()),
                    size: items.len(),
                });
                for item in items {
                    let _ = self.lower_hir_expression(item)?;
                }
                Ok(MirOperand::Local(dest))
            }
            HirExpression::Cast(expr, _ty) => self.lower_hir_expression(expr),
            HirExpression::Typed(expr, _) => self.lower_hir_expression(expr),
            HirExpression::Await(expr)
            | HirExpression::TryPropagate(expr)
            | HirExpression::Given(_, expr) => self.lower_hir_expression(expr),
            HirExpression::NullCoalescing(left, right) => {
                let _ = self.lower_hir_expression(left)?;
                let _ = self.lower_hir_expression(right)?;
                let dest = self.new_local(MirType::Unknown);
                Ok(MirOperand::Local(dest))
            }
            HirExpression::OptionalChain(base, _member) => {
                let _ = self.lower_hir_expression(base)?;
                let dest = self.new_local(MirType::Unknown);
                Ok(MirOperand::Local(dest))
            }
            _ => Err(CfgError::UnsupportedFeature(format!(
                "Expression type: {:?}",
                expr
            ))),
        }
    }
}

fn lower_hir_type_to_mir(ty: &HirType) -> MirType {
    match ty {
        HirType::Int => MirType::Int(32),
        HirType::UnsignedInt(_) => MirType::Int(32),
        HirType::Float => MirType::Float(64),
        HirType::Bool => MirType::Bool,
        HirType::String | HirType::CString => MirType::String,
        HirType::Char | HirType::CChar => MirType::Char,
        HirType::Unit | HirType::Void | HirType::Never => MirType::Unit,
        HirType::Array(inner) => MirType::Array(Box::new(lower_hir_type_to_mir(inner)), 0),
        HirType::Reference(inner)
        | HirType::MutableReference(inner)
        | HirType::Pointer(inner)
        | HirType::ConstPointer(inner) => MirType::Pointer(Box::new(lower_hir_type_to_mir(inner))),
        HirType::Function(params, ret) => MirType::Function(
            params.iter().map(lower_hir_type_to_mir).collect(),
            Box::new(lower_hir_type_to_mir(ret)),
        ),
        _ => MirType::Unknown,
    }
}

fn hir_literal_to_constant(lit: &HirLiteral) -> Option<MirConstant> {
    match lit {
        HirLiteral::Integer(v) => Some(MirConstant::Int(*v)),
        HirLiteral::Float(v) => Some(MirConstant::Float(*v)),
        HirLiteral::Boolean(v) => Some(MirConstant::Bool(*v)),
        HirLiteral::String(v) => Some(MirConstant::String(v.clone())),
        HirLiteral::Char(v) => Some(MirConstant::Char(*v)),
        HirLiteral::Unit => Some(MirConstant::Unit),
        HirLiteral::None => Some(MirConstant::Null),
    }
}

pub fn cfg_to_mir_function(
    cfg: &Cfg,
    name: &str,
    params: Vec<MirParameter>,
    return_type: MirType,
) -> MirFunction {
    let blocks: Vec<MirBasicBlock> = cfg
        .blocks
        .iter()
        .map(|(id, block)| MirBasicBlock {
            id: *id,
            instructions: block.instructions.clone(),
            terminator: block.terminator.clone(),
        })
        .collect();

    MirFunction {
        name: name.to_string(),
        type_params: Vec::new(),
        parameters: params,
        return_type,
        blocks,
        locals: HashMap::new(),
        name_to_local: HashMap::new(),
        is_extern: false,
    }
}

pub fn mir_function_to_cfg(func: &MirFunction) -> Cfg {
    let mut cfg = Cfg::new();

    let mut block_mapping: HashMap<MirBlockId, MirBlockId> = HashMap::new();

    for block in &func.blocks {
        let new_id = if block.id == 0 {
            cfg.entry_block
        } else {
            cfg.create_block()
        };
        block_mapping.insert(block.id, new_id);
    }

    cfg.set_entry(block_mapping.get(&0).copied().unwrap_or(0));

    for block in &func.blocks {
        let new_id = block_mapping[&block.id];
        if let Some(new_block) = cfg.get_block_mut(new_id) {
            new_block.instructions = block.instructions.clone();
            new_block.terminator = block.terminator.clone();
        }
    }

    for block in &func.blocks {
        let new_id = block_mapping[&block.id];
        let successors: Vec<MirBlockId> = match &block.terminator {
            MirTerminator::Branch { target } => vec![*target],
            MirTerminator::CondBranch {
                then_block,
                else_block,
                ..
            } => vec![*then_block, *else_block],
            MirTerminator::Switch { cases, default, .. } => {
                let mut succs: Vec<MirBlockId> = cases.iter().map(|(_, b)| *b).collect();
                succs.push(*default);
                succs
            }
            _ => Vec::new(),
        };

        for succ in successors {
            if let Some(&new_succ) = block_mapping.get(&succ) {
                cfg.add_edge(new_id, new_succ);
            }
        }
    }

    for block in &func.blocks {
        let is_exit = matches!(
            block.terminator,
            MirTerminator::Return { .. } | MirTerminator::Unreachable
        );
        if is_exit {
            if let Some(&new_id) = block_mapping.get(&block.id) {
                cfg.add_exit(new_id);
            }
        }
    }

    cfg
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cfg_builder_creates_entry_block() {
        let builder = CfgBuilder::new();
        let cfg = builder.build();

        assert!(cfg.blocks.contains_key(&0));
        assert_eq!(cfg.entry_block, 0);
    }

    #[test]
    fn cfg_can_create_multiple_blocks() {
        let mut builder = CfgBuilder::new();
        let block1 = builder.create_block();
        let block2 = builder.create_block();

        assert_ne!(block1, block2);

        let cfg = builder.build();
        assert_eq!(cfg.block_count(), 3);
    }

    #[test]
    fn cfg_can_add_edges() {
        let mut builder = CfgBuilder::new();
        let block1 = builder.create_block();
        let block2 = builder.create_block();

        builder.switch_to_block(0);
        builder.branch(block1);

        builder.switch_to_block(block1);
        builder.branch(block2);

        let cfg = builder.build();

        assert!(cfg.blocks[&0].successors.contains(&block1));
        assert!(cfg.blocks[&block1].predecessors.contains(&0));
    }

    #[test]
    fn cfg_conditional_branch() {
        let mut builder = CfgBuilder::new();
        let then_block = builder.create_block();
        let else_block = builder.create_block();

        builder.switch_to_block(0);
        builder.cond_branch(
            MirOperand::Constant(MirConstant::Bool(true)),
            then_block,
            else_block,
        );

        let cfg = builder.build();

        assert!(cfg.blocks[&0].successors.contains(&then_block));
        assert!(cfg.blocks[&0].successors.contains(&else_block));
    }

    #[test]
    fn cfg_validate_success() {
        let mut builder = CfgBuilder::new();
        let block1 = builder.create_block();

        builder.switch_to_block(0);
        builder.branch(block1);
        builder.switch_to_block(block1);
        builder.return_value(None);

        let cfg = builder.build();
        assert!(cfg.validate().is_ok());
    }

    #[test]
    fn cfg_depth_first_order() {
        let mut builder = CfgBuilder::new();
        let block1 = builder.create_block();
        let block2 = builder.create_block();

        builder.switch_to_block(0);
        builder.branch(block1);
        builder.switch_to_block(block1);
        builder.branch(block2);
        builder.switch_to_block(block2);
        builder.return_value(None);

        let cfg = builder.build();
        let order = cfg.depth_first_order();

        assert!(!order.is_empty());
        assert!(order.contains(&0));
        assert!(order.contains(&block1));
        assert!(order.contains(&block2));
    }

    #[test]
    fn cfg_dominators() {
        let mut builder = CfgBuilder::new();
        let block1 = builder.create_block();
        let block2 = builder.create_block();

        builder.switch_to_block(0);
        builder.branch(block1);
        builder.switch_to_block(block1);
        builder.branch(block2);
        builder.switch_to_block(block2);
        builder.return_value(None);

        let cfg = builder.build();
        let dom = cfg.compute_dominators();

        assert!(dom[&0].contains(&0));
        assert!(dom[&block1].contains(&0));
        assert!(dom[&block1].contains(&block1));
    }

    #[test]
    fn basic_block_is_entry_exit() {
        let mut block = BasicBlock::new(0);

        assert!(block.is_entry());
        assert!(block.is_exit());

        block.set_terminator(MirTerminator::Branch { target: 1 });
        assert!(!block.is_exit());

        block.set_terminator(MirTerminator::Return { value: None });
        assert!(block.is_exit());
    }

    #[test]
    fn cfg_to_dot() {
        let mut builder = CfgBuilder::new();
        let block1 = builder.create_block();

        builder.switch_to_block(0);
        builder.branch(block1);
        builder.switch_to_block(block1);
        builder.return_value(None);

        let cfg = builder.build();
        let dot = cfg.to_dot();

        assert!(dot.contains("digraph CFG"));
        assert!(dot.contains("->"));
    }

    #[test]
    fn mir_function_to_cfg_conversion() {
        let func = MirFunction {
            name: "test".to_string(),
            type_params: Vec::new(),
            parameters: Vec::new(),
            return_type: MirType::Unit,
            blocks: vec![
                MirBasicBlock {
                    id: 0,
                    instructions: vec![],
                    terminator: MirTerminator::Branch { target: 1 },
                },
                MirBasicBlock {
                    id: 1,
                    instructions: vec![],
                    terminator: MirTerminator::Return { value: None },
                },
            ],
            locals: HashMap::new(),
            name_to_local: HashMap::new(),
            is_extern: false,
        };

        let cfg = mir_function_to_cfg(&func);

        assert_eq!(cfg.block_count(), 2);
        assert!(cfg.validate().is_ok());
    }
}
