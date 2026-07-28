//! MIR → LIR lowering
//!
//! 该模块提供从 `x_mir::MirModule` 到 `x_lir::Program` 的最小可用 lowering，
//! 用于把新的编译流水线真正接起来：
//!
//! AST -> HIR -> MIR -> LIR -> Backend
//!
//! 当前实现目标：
//! - 为 CLI 的 `--emit lir` 提供结构化输出
//! - 为后端统一输入提供稳定边界
//! - 保持实现简单、保守、可扩展
//!
//! 这不是最终优化版 lowering：
//! - 暂未保留完整 CFG 语义
//! - 暂未处理 Phi/SSA 消解
//! - 暂未做寄存器分配、栈帧布局、调用约定细化
//!
//! 但它已经足以作为新的架构层次中的 LIR/XIR 生成入口。

use crate::{
    BinaryOp, Block, Declaration, Expression, ExternFunction, Field, Function, GlobalVar, Literal,
    Program, Qualifiers, Statement, Struct, Type, UnaryOp, Variable,
};
use x_mir::{
    MirBasicBlock, MirBinOp, MirConstant, MirFunction, MirInstruction, MirModule, MirOperand,
    MirTerminator, MirType, MirUnOp,
};

/// MIR 到 LIR 的 lowering 错误
#[derive(Debug, thiserror::Error)]
pub enum LirLowerError {
    #[error("不支持的 MIR 特性: {0}")]
    UnsupportedFeature(String),

    #[error("内部 lowering 错误: {0}")]
    Internal(String),
}

pub type LirLowerResult<T> = Result<T, LirLowerError>;

/// 将整个 MIR 模块 lowering 为 LIR 程序
pub fn lower_mir_to_lir(module: &MirModule) -> LirLowerResult<Program> {
    let mut program = Program::new();

    add_runtime_declarations(&mut program);

    // Lower import declarations
    for import in &module.imports {
        program.add(Declaration::Import(crate::Import {
            module_path: import.module_path.clone(),
            symbols: import.symbols.clone(),
            import_all: import.import_all,
        }));
    }

    // 结构体定义（class/record/enum 展开）
    for strct in &module.structs {
        program.add(Declaration::Struct(Struct {
            name: strct.name.clone(),
            fields: strct
                .fields
                .iter()
                .map(|(name, ty)| Field {
                    name: name.clone(),
                    type_: lower_type(ty),
                })
                .collect(),
        }));
    }

    for global in &module.globals {
        program.add(Declaration::Global(lower_global(global)?));
    }

    for func in &module.functions {
        if func.is_extern {
            program.add(Declaration::ExternFunction(lower_extern_function(func)?));
        } else {
            program.add(Declaration::Function(lower_function(func)?));
        }
    }

    Ok(program)
}

/// 添加运行时外部声明
fn add_runtime_declarations(program: &mut Program) {
    // Collect existing extern function names to avoid duplicates.
    let existing: std::collections::HashSet<String> = program
        .declarations
        .iter()
        .filter_map(|d| match d {
            Declaration::ExternFunction(ef) => Some(ef.name.clone()),
            _ => None,
        })
        .collect();

    let runtime = [
        ExternFunction {
            name: "printf".to_string(),
            type_params: Vec::new(),
            return_type: Type::Int,
            parameters: vec![Type::Pointer(Box::new(Type::Char))],
            abi: Some("c".to_string()),
        },
        ExternFunction {
            name: "malloc".to_string(),
            type_params: Vec::new(),
            return_type: Type::Pointer(Box::new(Type::Void)),
            parameters: vec![Type::Size],
            abi: Some("c".to_string()),
        },
        ExternFunction {
            name: "free".to_string(),
            type_params: Vec::new(),
            return_type: Type::Void,
            parameters: vec![Type::Pointer(Box::new(Type::Void))],
            abi: Some("c".to_string()),
        },
        ExternFunction {
            name: "x_perceus_retain".to_string(),
            type_params: Vec::new(),
            return_type: Type::Void,
            parameters: vec![Type::Pointer(Box::new(Type::Void))],
            abi: None,
        },
        ExternFunction {
            name: "x_perceus_release".to_string(),
            type_params: Vec::new(),
            return_type: Type::Void,
            parameters: vec![Type::Pointer(Box::new(Type::Void))],
            abi: None,
        },
    ];

    for decl in runtime {
        if !existing.contains(&decl.name) {
            program.add(Declaration::ExternFunction(decl));
        }
    }

    // ── X 动态值运行时（library/runtime/xrt.c）────────────────────────────
    let xptr = || Type::Pointer(Box::new(Type::Named("XValue".to_string())));
    // X strings are const C strings; keep distinct from mutable `*character`.
    let cstr = || {
        Type::Qualified(
            Qualifiers::const_(),
            Box::new(Type::Pointer(Box::new(Type::Char))),
        )
    };
    let xrt: Vec<ExternFunction> = vec![
        ext("x_from_int", xptr(), vec![Type::LongLong]),
        ext("x_from_double", xptr(), vec![Type::Double]),
        ext("x_from_bool", xptr(), vec![Type::LongLong]),
        ext("x_from_char", xptr(), vec![Type::LongLong]),
        ext("x_from_str", xptr(), vec![cstr()]),
        ext(
            "x_from_ptr",
            xptr(),
            vec![Type::Pointer(Box::new(Type::Void))],
        ),
        ext("x_list_new", xptr(), vec![]),
        ext("x_list_push", Type::Void, vec![xptr(), xptr()]),
        ext("x_list_get", xptr(), vec![xptr(), Type::LongLong]),
        ext(
            "x_list_set",
            Type::Void,
            vec![xptr(), Type::LongLong, xptr()],
        ),
        ext("x_list_len", Type::LongLong, vec![xptr()]),
        ext("x_map_new", xptr(), vec![]),
        ext("x_map_put", Type::Void, vec![xptr(), xptr(), xptr()]),
        ext("x_map_get", xptr(), vec![xptr(), xptr()]),
        ext("__index__", xptr(), vec![xptr(), Type::LongLong]),
        ext("x_as_int", Type::LongLong, vec![xptr()]),
        ext("x_as_double", Type::Double, vec![xptr()]),
        ext("x_as_bool", Type::LongLong, vec![xptr()]),
        ext("x_as_str", cstr(), vec![xptr()]),
        ext(
            "x_as_ptr",
            Type::Pointer(Box::new(Type::Void)),
            vec![xptr()],
        ),
        ext("x_fmt_value", cstr(), vec![xptr()]),
        ext("x_to_str", cstr(), vec![xptr()]),
        ext("x_str_concat", cstr(), vec![cstr(), cstr()]),
        ext("x_print", Type::Void, vec![xptr()]),
        ext("x_print_inline", Type::Void, vec![xptr()]),
        ext("x_print_newline", Type::Void, vec![]),
    ];
    for decl in xrt {
        if !existing.contains(&decl.name) {
            program.add(Declaration::ExternFunction(decl));
        }
    }
}

fn ext(name: &str, return_type: Type, parameters: Vec<Type>) -> ExternFunction {
    ExternFunction {
        name: name.to_string(),
        type_params: Vec::new(),
        return_type,
        parameters,
        abi: Some("c".to_string()),
    }
}

fn lower_global(global: &x_mir::MirGlobal) -> LirLowerResult<GlobalVar> {
    Ok(GlobalVar {
        name: global.name.clone(),
        type_: lower_type(&global.ty),
        initializer: global
            .initializer
            .as_ref()
            .map(lower_constant_to_expression),
        is_static: !global.mutable,
        extern_abi: global.extern_abi.clone(),
    })
}

fn lower_extern_function(func: &MirFunction) -> LirLowerResult<ExternFunction> {
    let type_params = func.type_params.iter().map(|tp| tp.name.clone()).collect();

    Ok(ExternFunction {
        name: func.name.clone(),
        type_params,
        return_type: lower_type(&func.return_type),
        parameters: func.parameters.iter().map(|p| lower_type(&p.ty)).collect(),
        abi: None,
    })
}

fn lower_function(func: &MirFunction) -> LirLowerResult<Function> {
    let mut lir_func = Function::new(&func.name, lower_type(&func.return_type));
    lir_func.type_params = func.type_params.iter().map(|tp| tp.name.clone()).collect();

    for (index, param) in func.parameters.iter().enumerate() {
        // Use arg{index} format to match param_name() in lower_operand
        lir_func = lir_func.param(&param_name(index), lower_type(&param.ty));
    }

    let mut body = Block::new();

    if !func.locals.is_empty() {
        let mut locals: Vec<_> = func.locals.iter().collect();
        locals.sort_by_key(|(id, _)| **id);

        for (id, ty) in locals {
            body.add(Statement::Variable(Variable {
                name: local_name(*id),
                type_: lower_type(ty),
                initializer: None,
                is_static: false,
                is_extern: false,
            }));
        }
    }

    if func.blocks.is_empty() {
        if let Some(default_return) = default_return_expr(&func.return_type) {
            body.add(Statement::Return(Some(default_return)));
        } else {
            body.add(Statement::Return(None));
        }
    } else {
        // Prefer structured control flow (While/If) so backends without goto work.
        let structured = structure_mir_blocks(&func.blocks)?;
        for stmt in structured {
            body.add(stmt);
        }
    }

    lir_func.body = body;
    Ok(lir_func)
}

/// Convert MIR basic blocks into structured LIR statements (While/If/Return),
/// falling back to Label/Goto only for irreducible regions.
fn structure_mir_blocks(blocks: &[MirBasicBlock]) -> LirLowerResult<Vec<Statement>> {
    if blocks.is_empty() {
        return Ok(Vec::new());
    }
    let by_id: std::collections::HashMap<usize, &MirBasicBlock> =
        blocks.iter().map(|b| (b.id, b)).collect();
    let entry = blocks[0].id;
    let mut out = Vec::new();
    emit_region(entry, &by_id, &std::collections::HashSet::new(), &mut out)?;
    Ok(out)
}

fn emit_region(
    start: usize,
    by_id: &std::collections::HashMap<usize, &MirBasicBlock>,
    stop: &std::collections::HashSet<usize>,
    out: &mut Vec<Statement>,
) -> LirLowerResult<()> {
    let mut current = Some(start);
    let mut guard = 0usize;
    while let Some(bid) = current {
        guard += 1;
        if guard > by_id.len() * 8 {
            return Err(LirLowerError::Internal(format!(
                "结构化控制流恢复陷入循环 (block {})",
                bid
            )));
        }
        if stop.contains(&bid) {
            break;
        }
        let block = match by_id.get(&bid) {
            Some(b) => *b,
            None => break,
        };

        // Peek: while-header must keep header instructions inside the loop body
        // so the condition is re-evaluated each iteration.
        if let MirTerminator::CondBranch {
            cond,
            then_block,
            else_block,
        } = &block.terminator
        {
            if is_while_header(bid, *then_block, *else_block, by_id, stop) {
                let mut loop_body = Vec::new();
                for instr in &block.instructions {
                    lower_instruction(instr, &mut loop_body)?;
                }
                // if (!cond) break;
                loop_body.push(Statement::If(crate::IfStatement {
                    condition: Expression::Unary(
                        UnaryOp::Not,
                        Box::new(lower_operand(cond)),
                    ),
                    then_branch: Box::new(Statement::Break),
                    else_branch: None,
                }));
                let mut body_stop = stop.clone();
                body_stop.insert(bid);
                body_stop.insert(*else_block);
                emit_region(*then_block, by_id, &body_stop, &mut loop_body)?;
                out.push(Statement::While(crate::WhileStatement {
                    condition: Expression::Literal(Literal::Bool(true)),
                    body: Box::new(Statement::Compound(Block {
                        statements: loop_body,
                    })),
                }));
                current = Some(*else_block);
                continue;
            }
        }

        // Emit instructions for this block
        for instr in &block.instructions {
            lower_instruction(instr, out)?;
        }

        match &block.terminator {
            MirTerminator::Return { value } => {
                let ret = match value {
                    Some(MirOperand::Constant(MirConstant::Unit)) | None => None,
                    Some(v) => Some(lower_operand(v)),
                };
                out.push(Statement::Return(ret));
                current = None;
            }
            MirTerminator::Unreachable => {
                out.push(Statement::Expression(Expression::Call(
                    Box::new(Expression::Variable("abort".to_string())),
                    vec![],
                )));
                current = None;
            }
            MirTerminator::Branch { target } => {
                if stop.contains(target) {
                    current = None;
                } else {
                    current = Some(*target);
                }
            }
            MirTerminator::CondBranch {
                cond,
                then_block,
                else_block,
            } => {
                let join = find_join_point(*then_block, *else_block, by_id, stop);
                let mut then_stop = stop.clone();
                let mut else_stop = stop.clone();
                if let Some(j) = join {
                    then_stop.insert(j);
                    else_stop.insert(j);
                }
                let mut then_stmts = Vec::new();
                let mut else_stmts = Vec::new();
                emit_region(*then_block, by_id, &then_stop, &mut then_stmts)?;
                emit_region(*else_block, by_id, &else_stop, &mut else_stmts)?;

                let then_branch = Box::new(Statement::Compound(Block {
                    statements: then_stmts,
                }));
                let else_branch = if else_stmts.is_empty() {
                    None
                } else {
                    Some(Box::new(Statement::Compound(Block {
                        statements: else_stmts,
                    })))
                };
                out.push(Statement::If(crate::IfStatement {
                    condition: lower_operand(cond),
                    then_branch,
                    else_branch,
                }));
                current = join;
            }
            MirTerminator::Switch {
                value,
                cases,
                default,
            } => {
                let join = {
                    let mut targets: Vec<usize> =
                        cases.iter().map(|(_, b)| *b).collect();
                    targets.push(*default);
                    let mut j = None;
                    for t in targets {
                        j = match j {
                            None => Some(t),
                            Some(a) => find_join_point(a, t, by_id, stop),
                        };
                    }
                    // Recompute join as common exit of all arms
                    let mut exits: Option<std::collections::HashSet<usize>> = None;
                    for (_, t) in cases.iter() {
                        let e = region_exit_targets(*t, by_id, stop);
                        exits = Some(match exits {
                            None => e,
                            Some(prev) => prev.intersection(&e).copied().collect(),
                        });
                    }
                    let e_def = region_exit_targets(*default, by_id, stop);
                    let common = match exits {
                        Some(e) => e.intersection(&e_def).copied().collect::<Vec<_>>(),
                        None => e_def.into_iter().collect(),
                    };
                    common.into_iter().min().or(j)
                };
                let mut case_list = Vec::new();
                for (v, target) in cases {
                    let mut arm_stop = stop.clone();
                    if let Some(j) = join {
                        arm_stop.insert(j);
                    }
                    let mut arm = Vec::new();
                    emit_region(*target, by_id, &arm_stop, &mut arm)?;
                    case_list.push(crate::SwitchCase {
                        value: lower_constant_to_expression(v),
                        body: Box::new(Statement::Compound(Block { statements: arm })),
                    });
                }
                let mut def_stop = stop.clone();
                if let Some(j) = join {
                    def_stop.insert(j);
                }
                let mut def_body = Vec::new();
                emit_region(*default, by_id, &def_stop, &mut def_body)?;
                out.push(Statement::Switch(crate::SwitchStatement {
                    expression: lower_operand(value),
                    cases: case_list,
                    default: Some(Box::new(Statement::Compound(Block {
                        statements: def_body,
                    }))),
                }));
                current = join;
            }
        }
    }
    Ok(())
}

fn lower_instruction(instr: &MirInstruction, out: &mut Vec<Statement>) -> LirLowerResult<()> {
    let mut tmp = Block::new();
    lower_instruction_into(instr, &mut tmp)?;
    out.extend(tmp.statements);
    Ok(())
}

/// While header detection: CondBranch(body, exit) where body can reach header again
/// *without leaving the enclosing region*.
///
/// `stop` must be treated as blocked: otherwise an `if` nested inside a `while`
/// is mis-detected as a loop header, because its then-arm can reach the if-header
/// again via the outer loop's back-edge. That mis-lowering turns:
///   `if (c) { A } else { B }`
/// into:
///   `while (true) { if (!c) break; A }` followed by `B`,
/// which infinite-loops (and can OOM) whenever `c` is true.
fn is_while_header(
    header: usize,
    body: usize,
    exit: usize,
    by_id: &std::collections::HashMap<usize, &MirBasicBlock>,
    stop: &std::collections::HashSet<usize>,
) -> bool {
    if body == exit || body == header {
        return false;
    }
    let mut blocked: Vec<usize> = stop.iter().copied().collect();
    if !blocked.contains(&exit) {
        blocked.push(exit);
    }
    can_reach(body, header, by_id, &blocked)
}

fn can_reach(
    from: usize,
    to: usize,
    by_id: &std::collections::HashMap<usize, &MirBasicBlock>,
    blocked: &[usize],
) -> bool {
    let mut stack = vec![from];
    let mut seen = std::collections::HashSet::new();
    while let Some(b) = stack.pop() {
        if blocked.contains(&b) {
            continue;
        }
        if b == to {
            return true;
        }
        if !seen.insert(b) {
            continue;
        }
        if let Some(block) = by_id.get(&b) {
            for s in terminator_successors(&block.terminator) {
                stack.push(s);
            }
        }
    }
    false
}

fn terminator_successors(term: &MirTerminator) -> Vec<usize> {
    match term {
        MirTerminator::Branch { target } => vec![*target],
        MirTerminator::CondBranch {
            then_block,
            else_block,
            ..
        } => vec![*then_block, *else_block],
        MirTerminator::Switch { cases, default, .. } => {
            let mut v: Vec<_> = cases.iter().map(|(_, b)| *b).collect();
            v.push(*default);
            v
        }
        MirTerminator::Return { .. } | MirTerminator::Unreachable => vec![],
    }
}

/// Exit targets of a region: successors that leave the interior reachable set.
fn region_exit_targets(
    start: usize,
    by_id: &std::collections::HashMap<usize, &MirBasicBlock>,
    stop: &std::collections::HashSet<usize>,
) -> std::collections::HashSet<usize> {
    let mut interior = std::collections::HashSet::new();
    let mut stack = vec![start];
    while let Some(b) = stack.pop() {
        if stop.contains(&b) {
            continue;
        }
        if !interior.insert(b) {
            continue;
        }
        if let Some(block) = by_id.get(&b) {
            for s in terminator_successors(&block.terminator) {
                if !stop.contains(&s) {
                    stack.push(s);
                }
            }
        }
    }
    let mut exits = std::collections::HashSet::new();
    for &b in &interior {
        if let Some(block) = by_id.get(&b) {
            for s in terminator_successors(&block.terminator) {
                if !interior.contains(&s) {
                    exits.insert(s);
                }
            }
        }
    }
    exits
}

/// Find join point of two branches via common exit targets.
fn find_join_point(
    a: usize,
    b: usize,
    by_id: &std::collections::HashMap<usize, &MirBasicBlock>,
    stop: &std::collections::HashSet<usize>,
) -> Option<usize> {
    if a == b {
        return Some(a);
    }
    let exits_a = region_exit_targets(a, by_id, stop);
    let exits_b = region_exit_targets(b, by_id, stop);
    let mut common: Vec<_> = exits_a.intersection(&exits_b).copied().collect();
    common.retain(|x| !stop.contains(x));
    if !common.is_empty() {
        common.sort_unstable();
        return Some(common[0]);
    }
    // Fallback: nearest common reachable block (excluding the two entries)
    let reach_a = reachable_set(a, by_id, stop);
    let reach_b = reachable_set(b, by_id, stop);
    let mut common: Vec<_> = reach_a
        .intersection(&reach_b)
        .copied()
        .filter(|x| *x != a && *x != b)
        .collect();
    common.sort_unstable();
    common.first().copied()
}

fn reachable_set(
    from: usize,
    by_id: &std::collections::HashMap<usize, &MirBasicBlock>,
    stop: &std::collections::HashSet<usize>,
) -> std::collections::HashSet<usize> {
    let mut seen = std::collections::HashSet::new();
    let mut stack = vec![from];
    while let Some(b) = stack.pop() {
        if stop.contains(&b) {
            seen.insert(b);
            continue;
        }
        if !seen.insert(b) {
            continue;
        }
        if let Some(block) = by_id.get(&b) {
            for s in terminator_successors(&block.terminator) {
                stack.push(s);
            }
        }
    }
    seen
}

fn lower_basic_block(block: &MirBasicBlock, body: &mut Block) -> LirLowerResult<()> {
    body.add(Statement::Label(block_label(block.id)));

    for instr in &block.instructions {
        lower_instruction_into(instr, body)?;
    }

    lower_terminator(&block.terminator, body)?;
    Ok(())
}

fn lower_instruction_into(instr: &MirInstruction, body: &mut Block) -> LirLowerResult<()> {
    match instr {
        MirInstruction::Assign { dest, value } => {
            body.add(assign_local_stmt(*dest, lower_operand(value)));
        }
        MirInstruction::BinaryOp {
            dest,
            op,
            left,
            right,
        } => {
            body.add(assign_local_stmt(
                *dest,
                Expression::Binary(
                    lower_binary_op(*op),
                    Box::new(lower_operand(left)),
                    Box::new(lower_operand(right)),
                ),
            ));
        }
        MirInstruction::UnaryOp { dest, op, operand } => {
            body.add(assign_local_stmt(
                *dest,
                Expression::Unary(lower_unary_op(*op), Box::new(lower_operand(operand))),
            ));
        }
        MirInstruction::Call { dest, func, args } => {
            let call = Expression::Call(
                Box::new(lower_operand(func)),
                args.iter().map(lower_operand).collect(),
            );

            if let Some(dest) = dest {
                body.add(assign_local_stmt(*dest, call));
            } else {
                body.add(Statement::Expression(call));
            }
        }
        MirInstruction::FieldAccess {
            dest,
            object,
            field,
        } => {
            // 类实例以指针表示，字段访问用 PointerMember（Native/Zig 均正确）。
            body.add(assign_local_stmt(
                *dest,
                Expression::PointerMember(Box::new(lower_operand(object)), field.clone()),
            ));
        }
        MirInstruction::SetField {
            object,
            field,
            value,
        } => {
            body.add(Statement::Expression(Expression::Assign(
                Box::new(Expression::PointerMember(
                    Box::new(lower_operand(object)),
                    field.clone(),
                )),
                Box::new(lower_operand(value)),
            )));
        }
        MirInstruction::ArrayAccess { dest, array, index } => {
            body.add(assign_local_stmt(
                *dest,
                Expression::Index(
                    Box::new(lower_operand(array)),
                    Box::new(lower_operand(index)),
                ),
            ));
        }
        MirInstruction::Alloc { dest, ty, size } => {
            let malloc_call = Expression::Call(
                Box::new(Expression::Variable("malloc".to_string())),
                vec![Expression::Literal(Literal::UnsignedLongLong(*size as u64))],
            );

            body.add(assign_local_stmt(
                *dest,
                Expression::Cast(
                    Type::Pointer(Box::new(lower_type(ty))),
                    Box::new(malloc_call),
                ),
            ));
        }
        MirInstruction::Load { dest, ptr } => {
            body.add(assign_local_stmt(
                *dest,
                Expression::Dereference(Box::new(lower_operand(ptr))),
            ));
        }
        MirInstruction::Store { ptr, value } => {
            // Handle global variable stores differently from dereference
            let ptr_expr = match ptr {
                MirOperand::Global(name) => Expression::Variable(name.clone()),
                _ => Expression::Dereference(Box::new(lower_operand(ptr))),
            };
            body.add(Statement::Expression(Expression::Assign(
                Box::new(ptr_expr),
                Box::new(lower_operand(value)),
            )));
        }
        MirInstruction::Cast { dest, value, ty } => {
            body.add(assign_local_stmt(
                *dest,
                Expression::Cast(lower_type(ty), Box::new(lower_operand(value))),
            ));
        }
        MirInstruction::Dup { dest, src } => {
            // Perceus: retain the reference before assignment
            let src_expr = lower_operand(src);
            body.add(Statement::Expression(Expression::Call(
                Box::new(Expression::Variable("x_perceus_retain".to_string())),
                vec![Expression::Cast(
                    Type::Pointer(Box::new(Type::Void)),
                    Box::new(src_expr.clone()),
                )],
            )));
            body.add(assign_local_stmt(*dest, src_expr));
        }
        MirInstruction::Drop { value } => {
            // Perceus: release the reference (deallocates when count reaches zero)
            let expr = lower_operand(value);
            body.add(Statement::Expression(Expression::Call(
                Box::new(Expression::Variable("x_perceus_release".to_string())),
                vec![Expression::Cast(
                    Type::Pointer(Box::new(Type::Void)),
                    Box::new(expr),
                )],
            )));
        }
        MirInstruction::Reuse { dest, src } => {
            // Reuse just moves the reference, no retain/release needed
            body.add(assign_local_stmt(*dest, lower_operand(src)));
        }
        MirInstruction::WhenGuard {
            dest,
            condition,
            body: guard_body,
        } => {
            // WhenGuard: if condition is true, return the body value
            // For now, emit as a simple if-return pattern
            body.add(Statement::If(crate::IfStatement {
                condition: lower_operand(condition),
                then_branch: Box::new(Statement::Return(Some(lower_operand(guard_body)))),
                else_branch: None,
            }));
            body.add(assign_local_stmt(
                *dest,
                Expression::Literal(Literal::Integer(0)),
            ));
        }
    }

    Ok(())
}

fn lower_terminator(term: &MirTerminator, body: &mut Block) -> LirLowerResult<()> {
    match term {
        MirTerminator::Branch { target } => {
            body.add(Statement::Goto(block_label(*target)));
        }
        MirTerminator::CondBranch {
            cond,
            then_block,
            else_block,
        } => {
            body.add(Statement::If(crate::IfStatement {
                condition: lower_operand(cond),
                then_branch: Box::new(Statement::Goto(block_label(*then_block))),
                else_branch: Some(Box::new(Statement::Goto(block_label(*else_block)))),
            }));
        }
        MirTerminator::Return { value } => {
            let ret = match value {
                Some(MirOperand::Constant(MirConstant::Unit)) | None => None,
                Some(v) => Some(lower_operand(v)),
            };
            body.add(Statement::Return(ret));
        }
        MirTerminator::Unreachable => {
            body.add(Statement::Expression(Expression::Call(
                Box::new(Expression::Variable("abort".to_string())),
                vec![],
            )));
        }
        MirTerminator::Switch {
            value,
            cases,
            default,
        } => {
            body.add(Statement::Switch(crate::SwitchStatement {
                expression: lower_operand(value),
                cases: cases
                    .iter()
                    .map(|(constant, block)| crate::SwitchCase {
                        value: lower_constant_to_expression(constant),
                        body: Box::new(Statement::Goto(block_label(*block))),
                    })
                    .collect(),
                default: Some(Box::new(Statement::Goto(block_label(*default)))),
            }));
        }
    }

    Ok(())
}

fn lower_operand(operand: &MirOperand) -> Expression {
    match operand {
        MirOperand::Local(id) => Expression::Variable(local_name(*id)),
        MirOperand::Constant(c) => lower_constant_to_expression(c),
        MirOperand::Param(index) => Expression::Variable(param_name(*index)),
        MirOperand::Global(name) => Expression::Variable(name.clone()),
    }
}

fn lower_constant_to_expression(constant: &MirConstant) -> Expression {
    Expression::Literal(match constant {
        MirConstant::Int(v) => Literal::Integer(*v),
        MirConstant::Float(v) => Literal::Double(*v),
        MirConstant::Bool(v) => Literal::Bool(*v),
        MirConstant::String(v) => Literal::String(v.clone()),
        MirConstant::Char(v) => Literal::Char(*v),
        MirConstant::Null => Literal::NullPointer,
        MirConstant::Unit => Literal::Integer(0),
    })
}

fn lower_type(ty: &MirType) -> Type {
    match ty {
        MirType::Int(bits) => match bits {
            0..=32 => Type::Int,
            _ => Type::Long,
        },
        MirType::Float(bits) => match bits {
            0..=32 => Type::Float,
            _ => Type::Double,
        },
        MirType::Bool => Type::Bool,
        MirType::String => Type::Qualified(
            Qualifiers::const_(),
            Box::new(Type::Pointer(Box::new(Type::Char))),
        ),
        MirType::Char => Type::Char,
        MirType::Unit => Type::Void,
        MirType::Pointer(inner) => match inner.as_ref() {
            // XValue is a heap-allocated boxed value, so Pointer(XValue) is just Pointer(Named("XValue")).
            MirType::Struct(name, _) if name == "XValue" => {
                Type::Pointer(Box::new(Type::Named(name.clone())))
            }
            _ => Type::Pointer(Box::new(lower_type(inner))),
        },
        MirType::Array(inner, len) => Type::Array(Box::new(lower_type(inner)), Some(*len as u64)),
        MirType::Struct(name, fields) if name == "tuple" => {
            Type::Tuple(fields.iter().map(lower_type).collect())
        }
        // XValue is a heap-allocated boxed value, so it's represented as a pointer in LIR.
        MirType::Struct(name, _) if name == "XValue" => {
            Type::Pointer(Box::new(Type::Named(name.clone())))
        }
        MirType::Struct(name, _) => Type::Named(name.clone()),
        MirType::Function(params, ret) => Type::FunctionPointer(
            Box::new(lower_type(ret)),
            params.iter().map(lower_type).collect(),
        ),
        MirType::Unknown => Type::Long,
    }
}

fn lower_binary_op(op: MirBinOp) -> BinaryOp {
    match op {
        MirBinOp::Add => BinaryOp::Add,
        MirBinOp::Sub => BinaryOp::Subtract,
        MirBinOp::Mul => BinaryOp::Multiply,
        MirBinOp::Div => BinaryOp::Divide,
        MirBinOp::Mod => BinaryOp::Modulo,
        MirBinOp::Eq => BinaryOp::Equal,
        MirBinOp::Ne => BinaryOp::NotEqual,
        MirBinOp::Lt => BinaryOp::LessThan,
        MirBinOp::Le => BinaryOp::LessThanEqual,
        MirBinOp::Gt => BinaryOp::GreaterThan,
        MirBinOp::Ge => BinaryOp::GreaterThanEqual,
        MirBinOp::And => BinaryOp::LogicalAnd,
        MirBinOp::Or => BinaryOp::LogicalOr,
        MirBinOp::BitAnd => BinaryOp::BitAnd,
        MirBinOp::BitOr => BinaryOp::BitOr,
        MirBinOp::BitXor => BinaryOp::BitXor,
        MirBinOp::Shl => BinaryOp::LeftShift,
        MirBinOp::Shr => BinaryOp::RightShift,
    }
}

fn lower_unary_op(op: MirUnOp) -> UnaryOp {
    match op {
        MirUnOp::Neg => UnaryOp::Minus,
        MirUnOp::Not => UnaryOp::Not,
        MirUnOp::BitNot => UnaryOp::BitNot,
        MirUnOp::Reference => UnaryOp::Reference,
        MirUnOp::MutableReference => UnaryOp::MutableReference,
    }
}

fn assign_local_stmt(local: usize, expr: Expression) -> Statement {
    Statement::Expression(Expression::Assign(
        Box::new(Expression::Variable(local_name(local))),
        Box::new(expr),
    ))
}

fn local_name(id: usize) -> String {
    format!("t{id}")
}

fn param_name(index: usize) -> String {
    format!("arg{index}")
}

fn block_label(id: usize) -> String {
    format!("bb{id}")
}

fn default_return_expr(ty: &MirType) -> Option<Expression> {
    match ty {
        MirType::Unit => None,
        MirType::Bool => Some(Expression::Literal(Literal::Bool(false))),
        MirType::Int(_) => Some(Expression::Literal(Literal::Integer(0))),
        MirType::Float(_) => Some(Expression::Literal(Literal::Double(0.0))),
        MirType::String => Some(Expression::Literal(Literal::String(String::new()))),
        MirType::Char => Some(Expression::Literal(Literal::Char('\0'))),
        MirType::Pointer(_) => Some(Expression::Literal(Literal::NullPointer)),
        MirType::Array(_, _)
        | MirType::Struct(_, _)
        | MirType::Function(_, _)
        | MirType::Unknown => Some(Expression::Literal(Literal::Integer(0))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use x_mir::{
        MirBasicBlock, MirConstant, MirFunction, MirGlobal, MirInstruction, MirModule, MirOperand,
        MirParameter, MirTerminator, MirType,
    };

    #[test]
    fn lower_empty_module() {
        let mir = MirModule {
            name: "main".to_string(),
            imports: Vec::new(),
            structs: Vec::new(),
            functions: vec![],
            globals: vec![],
        };

        let lir = lower_mir_to_lir(&mir).expect("lowering should succeed");
        assert!(!lir.declarations.is_empty()); // runtime decls
    }

    #[test]
    fn lower_simple_function() {
        let mir = MirModule {
            name: "main".to_string(),
            imports: Vec::new(),
            structs: Vec::new(),
            globals: vec![],
            functions: vec![MirFunction {
                name: "main".to_string(),
                type_params: Vec::new(),
                parameters: vec![MirParameter {
                    name: "x".to_string(),
                    ty: MirType::Int(32),
                    index: 0,
                }],
                return_type: MirType::Int(32),
                blocks: vec![MirBasicBlock {
                    id: 0,
                    instructions: vec![MirInstruction::Assign {
                        dest: 0,
                        value: MirOperand::Constant(MirConstant::Int(42)),
                    }],
                    terminator: MirTerminator::Return {
                        value: Some(MirOperand::Local(0)),
                    },
                }],
                locals: [(0usize, MirType::Int(32))].into_iter().collect(),
                name_to_local: [("x".to_string(), 0)].into_iter().collect(),
                is_extern: false,
            }],
        };

        let lir = lower_mir_to_lir(&mir).expect("lowering should succeed");
        let text = lir.to_string();
        assert!(text.contains("main"));
        assert!(text.contains("t0"));
        assert!(text.contains("return t0;"));
    }

    #[test]
    fn lower_global_variable() {
        let mir = MirModule {
            name: "main".to_string(),
            imports: Vec::new(),
            structs: Vec::new(),
            functions: vec![],
            globals: vec![MirGlobal {
                name: "answer".to_string(),
                ty: MirType::Int(32),
                initializer: Some(MirConstant::Int(42)),
                mutable: false,
                extern_abi: None,
            }],
        };

        let lir = lower_mir_to_lir(&mir).expect("lowering should succeed");
        let text = lir.to_string();
        assert!(text.contains("answer"));
        assert!(text.contains("42"));
    }

    // ==================== Binary Operations ====================

    #[test]
    fn lower_binary_add() {
        let mir = create_binary_op_module(x_mir::MirBinOp::Add, 10, 20);
        let lir = lower_mir_to_lir(&mir).expect("lowering should succeed");
        let text = lir.to_string();
        assert!(text.contains("t0 + t1") || text.contains("+"));
    }

    #[test]
    fn lower_binary_sub() {
        let mir = create_binary_op_module(x_mir::MirBinOp::Sub, 30, 10);
        let lir = lower_mir_to_lir(&mir).expect("lowering should succeed");
        let text = lir.to_string();
        assert!(text.contains("t0 - t1") || text.contains("-"));
    }

    #[test]
    fn lower_binary_mul() {
        let mir = create_binary_op_module(x_mir::MirBinOp::Mul, 5, 6);
        let lir = lower_mir_to_lir(&mir).expect("lowering should succeed");
        let text = lir.to_string();
        assert!(text.contains("t0 * t1") || text.contains("*"));
    }

    #[test]
    fn lower_binary_div() {
        let mir = create_binary_op_module(x_mir::MirBinOp::Div, 100, 10);
        let lir = lower_mir_to_lir(&mir).expect("lowering should succeed");
        let text = lir.to_string();
        assert!(text.contains("t0 / t1") || text.contains("/"));
    }

    #[test]
    fn lower_binary_mod() {
        let mir = create_binary_op_module(x_mir::MirBinOp::Mod, 17, 5);
        let lir = lower_mir_to_lir(&mir).expect("lowering should succeed");
        let text = lir.to_string();
        assert!(text.contains("t0 % t1") || text.contains("%"));
    }

    #[test]
    fn lower_binary_eq() {
        let mir = create_binary_op_module(x_mir::MirBinOp::Eq, 5, 5);
        let lir = lower_mir_to_lir(&mir).expect("lowering should succeed");
        let text = lir.to_string();
        assert!(text.contains("t0 == t1") || text.contains("=="));
    }

    #[test]
    fn lower_binary_lt() {
        let mir = create_binary_op_module(x_mir::MirBinOp::Lt, 3, 5);
        let lir = lower_mir_to_lir(&mir).expect("lowering should succeed");
        let text = lir.to_string();
        assert!(text.contains("t0 < t1") || text.contains("<"));
    }

    #[test]
    fn lower_binary_and() {
        let mir = create_binary_op_module(x_mir::MirBinOp::And, 1, 1);
        let lir = lower_mir_to_lir(&mir).expect("lowering should succeed");
        let text = lir.to_string();
        assert!(text.contains("t0 && t1") || text.contains("&&"));
    }

    #[test]
    fn lower_binary_or() {
        let mir = create_binary_op_module(x_mir::MirBinOp::Or, 0, 1);
        let lir = lower_mir_to_lir(&mir).expect("lowering should succeed");
        let text = lir.to_string();
        assert!(text.contains("t0 || t1") || text.contains("||"));
    }

    // ==================== Unary Operations ====================

    #[test]
    fn lower_unary_not() {
        let mir = MirModule {
            name: "main".to_string(),
            imports: Vec::new(),
            structs: Vec::new(),
            globals: vec![],
            functions: vec![MirFunction {
                name: "test_not".to_string(),
                type_params: Vec::new(),
                parameters: vec![],
                return_type: MirType::Bool,
                blocks: vec![MirBasicBlock {
                    id: 0,
                    instructions: vec![
                        MirInstruction::Assign {
                            dest: 0,
                            value: MirOperand::Constant(MirConstant::Bool(true)),
                        },
                        MirInstruction::UnaryOp {
                            dest: 1,
                            op: MirUnOp::Not,
                            operand: MirOperand::Local(0),
                        },
                    ],
                    terminator: MirTerminator::Return {
                        value: Some(MirOperand::Local(1)),
                    },
                }],
                locals: [(0, MirType::Bool), (1, MirType::Bool)]
                    .into_iter()
                    .collect(),
                name_to_local: vec![].into_iter().collect(),
                is_extern: false,
            }],
        };
        let lir = lower_mir_to_lir(&mir).expect("lowering should succeed");
        let text = lir.to_string();
        assert!(text.contains("!"));
    }

    #[test]
    fn lower_unary_neg() {
        let mir = MirModule {
            name: "main".to_string(),
            imports: Vec::new(),
            structs: Vec::new(),
            globals: vec![],
            functions: vec![MirFunction {
                name: "test_neg".to_string(),
                type_params: Vec::new(),
                parameters: vec![],
                return_type: MirType::Int(32),
                blocks: vec![MirBasicBlock {
                    id: 0,
                    instructions: vec![
                        MirInstruction::Assign {
                            dest: 0,
                            value: MirOperand::Constant(MirConstant::Int(42)),
                        },
                        MirInstruction::UnaryOp {
                            dest: 1,
                            op: MirUnOp::Neg,
                            operand: MirOperand::Local(0),
                        },
                    ],
                    terminator: MirTerminator::Return {
                        value: Some(MirOperand::Local(1)),
                    },
                }],
                locals: [(0, MirType::Int(32)), (1, MirType::Int(32))]
                    .into_iter()
                    .collect(),
                name_to_local: vec![].into_iter().collect(),
                is_extern: false,
            }],
        };
        let lir = lower_mir_to_lir(&mir).expect("lowering should succeed");
        let text = lir.to_string();
        assert!(text.contains("-"));
    }

    // ==================== Function Calls ====================

    #[test]
    fn lower_function_call_no_args() {
        let mir = MirModule {
            name: "main".to_string(),
            imports: Vec::new(),
            structs: Vec::new(),
            globals: vec![],
            functions: vec![
                MirFunction {
                    name: "get_value".to_string(),
                    type_params: Vec::new(),
                    parameters: vec![],
                    return_type: MirType::Int(32),
                    blocks: vec![MirBasicBlock {
                        id: 0,
                        instructions: vec![],
                        terminator: MirTerminator::Return {
                            value: Some(MirOperand::Constant(MirConstant::Int(42))),
                        },
                    }],
                    locals: vec![].into_iter().collect(),
                    name_to_local: vec![].into_iter().collect(),
                    is_extern: false,
                },
                MirFunction {
                    name: "caller".to_string(),
                    type_params: Vec::new(),
                    parameters: vec![],
                    return_type: MirType::Int(32),
                    blocks: vec![MirBasicBlock {
                        id: 0,
                        instructions: vec![MirInstruction::Call {
                            dest: Some(0),
                            func: MirOperand::Global("get_value".to_string()),
                            args: vec![],
                        }],
                        terminator: MirTerminator::Return {
                            value: Some(MirOperand::Local(0)),
                        },
                    }],
                    locals: [(0, MirType::Int(32))].into_iter().collect(),
                    name_to_local: vec![].into_iter().collect(),
                    is_extern: false,
                },
            ],
        };
        let lir = lower_mir_to_lir(&mir).expect("lowering should succeed");
        let text = lir.to_string();
        assert!(text.contains("get_value"));
        assert!(text.contains("caller"));
    }

    #[test]
    fn lower_function_call_with_args() {
        let mir = MirModule {
            name: "main".to_string(),
            imports: Vec::new(),
            structs: Vec::new(),
            globals: vec![],
            functions: vec![
                MirFunction {
                    name: "add".to_string(),
                    type_params: Vec::new(),
                    parameters: vec![
                        MirParameter {
                            name: "a".to_string(),
                            ty: MirType::Int(32),
                            index: 0,
                        },
                        MirParameter {
                            name: "b".to_string(),
                            ty: MirType::Int(32),
                            index: 1,
                        },
                    ],
                    return_type: MirType::Int(32),
                    blocks: vec![MirBasicBlock {
                        id: 0,
                        instructions: vec![MirInstruction::BinaryOp {
                            dest: 0,
                            op: x_mir::MirBinOp::Add,
                            left: MirOperand::Param(0),
                            right: MirOperand::Param(1),
                        }],
                        terminator: MirTerminator::Return {
                            value: Some(MirOperand::Local(0)),
                        },
                    }],
                    locals: [(0, MirType::Int(32))].into_iter().collect(),
                    name_to_local: vec![].into_iter().collect(),
                    is_extern: false,
                },
                MirFunction {
                    name: "caller".to_string(),
                    type_params: Vec::new(),
                    parameters: vec![],
                    return_type: MirType::Int(32),
                    blocks: vec![MirBasicBlock {
                        id: 0,
                        instructions: vec![MirInstruction::Call {
                            dest: Some(0),
                            func: MirOperand::Global("add".to_string()),
                            args: vec![
                                MirOperand::Constant(MirConstant::Int(10)),
                                MirOperand::Constant(MirConstant::Int(20)),
                            ],
                        }],
                        terminator: MirTerminator::Return {
                            value: Some(MirOperand::Local(0)),
                        },
                    }],
                    locals: [(0, MirType::Int(32))].into_iter().collect(),
                    name_to_local: vec![].into_iter().collect(),
                    is_extern: false,
                },
            ],
        };
        let lir = lower_mir_to_lir(&mir).expect("lowering should succeed");
        let text = lir.to_string();
        assert!(text.contains("add"));
        assert!(text.contains("10"));
        assert!(text.contains("20"));
    }

    // ==================== Control Flow ====================

    #[test]
    fn lower_conditional_branch() {
        let mir = MirModule {
            name: "main".to_string(),
            imports: Vec::new(),
            structs: Vec::new(),
            globals: vec![],
            functions: vec![MirFunction {
                name: "abs".to_string(),
                type_params: Vec::new(),
                parameters: vec![MirParameter {
                    name: "x".to_string(),
                    ty: MirType::Int(32),
                    index: 0,
                }],
                return_type: MirType::Int(32),
                blocks: vec![
                    MirBasicBlock {
                        id: 0,
                        instructions: vec![MirInstruction::BinaryOp {
                            dest: 1,
                            op: x_mir::MirBinOp::Lt,
                            left: MirOperand::Param(0),
                            right: MirOperand::Constant(MirConstant::Int(0)),
                        }],
                        terminator: MirTerminator::CondBranch {
                            cond: MirOperand::Local(1),
                            then_block: 1,
                            else_block: 2,
                        },
                    },
                    MirBasicBlock {
                        id: 1,
                        instructions: vec![MirInstruction::UnaryOp {
                            dest: 2,
                            op: MirUnOp::Neg,
                            operand: MirOperand::Param(0),
                        }],
                        terminator: MirTerminator::Return {
                            value: Some(MirOperand::Local(2)),
                        },
                    },
                    MirBasicBlock {
                        id: 2,
                        instructions: vec![],
                        terminator: MirTerminator::Return {
                            value: Some(MirOperand::Param(0)),
                        },
                    },
                ],
                locals: [(1, MirType::Bool), (2, MirType::Int(32))]
                    .into_iter()
                    .collect(),
                name_to_local: vec![].into_iter().collect(),
                is_extern: false,
            }],
        };
        let lir = lower_mir_to_lir(&mir).expect("lowering should succeed");
        let text = lir.to_string();
        assert!(text.contains("if"));
        // Structured lowering prefers If bodies over goto
        assert!(text.contains("return") || text.contains("goto"));
    }

    /// Regression: if nested in while must stay an If, not be misread as a while
    /// header via the outer loop back-edge (that bug caused infinite loops / OOM).
    #[test]
    fn lower_if_inside_while_stays_if() {
        // CFG matching MIR from:
        //   while (keep) { if (flag) { t = 1 } else { t = 2 } }
        // Blocks:
        //   0 entry -> 1
        //   1 header: CondBranch(keep) -> body(2) / exit(6)
        //   2 ifhead: CondBranch(flag) -> then(3) / else(4)
        //   3 then: t=1 -> merge(5)
        //   4 else: t=2 -> merge(5)
        //   5 merge -> header(1)
        //   6 exit: return t
        let mir = MirModule {
            name: "main".to_string(),
            imports: Vec::new(),
            structs: Vec::new(),
            globals: vec![],
            functions: vec![MirFunction {
                name: "if_in_while".to_string(),
                type_params: Vec::new(),
                parameters: vec![
                    MirParameter {
                        name: "keep".to_string(),
                        ty: MirType::Bool,
                        index: 0,
                    },
                    MirParameter {
                        name: "flag".to_string(),
                        ty: MirType::Bool,
                        index: 1,
                    },
                ],
                return_type: MirType::Int(32),
                blocks: vec![
                    MirBasicBlock {
                        id: 0,
                        instructions: vec![MirInstruction::Assign {
                            dest: 0,
                            value: MirOperand::Constant(MirConstant::Int(0)),
                        }],
                        terminator: MirTerminator::Branch { target: 1 },
                    },
                    MirBasicBlock {
                        id: 1,
                        instructions: vec![],
                        terminator: MirTerminator::CondBranch {
                            cond: MirOperand::Param(0),
                            then_block: 2,
                            else_block: 6,
                        },
                    },
                    MirBasicBlock {
                        id: 2,
                        instructions: vec![],
                        terminator: MirTerminator::CondBranch {
                            cond: MirOperand::Param(1),
                            then_block: 3,
                            else_block: 4,
                        },
                    },
                    MirBasicBlock {
                        id: 3,
                        instructions: vec![MirInstruction::Assign {
                            dest: 0,
                            value: MirOperand::Constant(MirConstant::Int(1)),
                        }],
                        terminator: MirTerminator::Branch { target: 5 },
                    },
                    MirBasicBlock {
                        id: 4,
                        instructions: vec![MirInstruction::Assign {
                            dest: 0,
                            value: MirOperand::Constant(MirConstant::Int(2)),
                        }],
                        terminator: MirTerminator::Branch { target: 5 },
                    },
                    MirBasicBlock {
                        id: 5,
                        instructions: vec![],
                        terminator: MirTerminator::Branch { target: 1 },
                    },
                    MirBasicBlock {
                        id: 6,
                        instructions: vec![],
                        terminator: MirTerminator::Return {
                            value: Some(MirOperand::Local(0)),
                        },
                    },
                ],
                locals: [(0, MirType::Int(32))].into_iter().collect(),
                name_to_local: vec![].into_iter().collect(),
                is_extern: false,
            }],
        };
        let lir = lower_mir_to_lir(&mir).expect("lowering should succeed");
        let text = lir.to_string();
        // Strip declarations; only look at the function body.
        let body = text
            .split("int if_in_while")
            .nth(1)
            .expect("function body");

        // Outer loop reconstructed as while(true) { if (!cond) break; ... }
        assert!(body.contains("while (true)"), "expected outer while: {body}");
        // Nested conditional must be a real if/else, not a second while(true)
        assert!(
            body.contains("if (arg1)") && body.contains("else"),
            "expected if/else for nested branch: {body}"
        );
        assert_eq!(
            body.matches("while (true)").count(),
            1,
            "nested if must not become a second while: {body}"
        );
        // Then/else arms assign distinct values
        assert!(body.contains("(t0 = 1)") && body.contains("(t0 = 2)"), "{body}");
    }

    #[test]
    fn lower_unconditional_branch() {
        let mir = MirModule {
            name: "main".to_string(),
            imports: Vec::new(),
            structs: Vec::new(),
            globals: vec![],
            functions: vec![MirFunction {
                name: "loop_example".to_string(),
                type_params: Vec::new(),
                parameters: vec![],
                return_type: MirType::Int(32),
                blocks: vec![
                    MirBasicBlock {
                        id: 0,
                        instructions: vec![MirInstruction::Assign {
                            dest: 0,
                            value: MirOperand::Constant(MirConstant::Int(0)),
                        }],
                        terminator: MirTerminator::Branch { target: 1 },
                    },
                    MirBasicBlock {
                        id: 1,
                        instructions: vec![],
                        terminator: MirTerminator::Return {
                            value: Some(MirOperand::Local(0)),
                        },
                    },
                ],
                locals: [(0, MirType::Int(32))].into_iter().collect(),
                name_to_local: vec![].into_iter().collect(),
                is_extern: false,
            }],
        };
        let lir = lower_mir_to_lir(&mir).expect("lowering should succeed");
        let text = lir.to_string();
        // Straight-line Branch is linearized (no goto needed)
        assert!(text.contains("return") || text.contains("goto"));
    }

    // ==================== Switch Statement ====================

    #[test]
    fn lower_switch_statement() {
        let mir = MirModule {
            name: "main".to_string(),
            imports: Vec::new(),
            structs: Vec::new(),
            globals: vec![],
            functions: vec![MirFunction {
                name: "switch_test".to_string(),
                type_params: Vec::new(),
                parameters: vec![MirParameter {
                    name: "x".to_string(),
                    ty: MirType::Int(32),
                    index: 0,
                }],
                return_type: MirType::Int(32),
                blocks: vec![
                    MirBasicBlock {
                        id: 0,
                        instructions: vec![],
                        terminator: MirTerminator::Switch {
                            value: MirOperand::Param(0),
                            cases: vec![(MirConstant::Int(1), 1), (MirConstant::Int(2), 2)],
                            default: 3,
                        },
                    },
                    MirBasicBlock {
                        id: 1,
                        instructions: vec![],
                        terminator: MirTerminator::Return {
                            value: Some(MirOperand::Constant(MirConstant::Int(10))),
                        },
                    },
                    MirBasicBlock {
                        id: 2,
                        instructions: vec![],
                        terminator: MirTerminator::Return {
                            value: Some(MirOperand::Constant(MirConstant::Int(20))),
                        },
                    },
                    MirBasicBlock {
                        id: 3,
                        instructions: vec![],
                        terminator: MirTerminator::Return {
                            value: Some(MirOperand::Constant(MirConstant::Int(0))),
                        },
                    },
                ],
                locals: vec![].into_iter().collect(),
                name_to_local: vec![].into_iter().collect(),
                is_extern: false,
            }],
        };
        let lir = lower_mir_to_lir(&mir).expect("lowering should succeed");
        let text = lir.to_string();
        assert!(text.contains("switch"));
    }

    // ==================== Memory Operations ====================

    #[test]
    fn lower_alloc_instruction() {
        let mir = MirModule {
            name: "main".to_string(),
            imports: Vec::new(),
            structs: Vec::new(),
            globals: vec![],
            functions: vec![MirFunction {
                name: "alloc_test".to_string(),
                type_params: Vec::new(),
                parameters: vec![],
                return_type: MirType::Pointer(Box::new(MirType::Int(32))),
                blocks: vec![MirBasicBlock {
                    id: 0,
                    instructions: vec![MirInstruction::Alloc {
                        dest: 0,
                        ty: MirType::Int(32),
                        size: 4,
                    }],
                    terminator: MirTerminator::Return {
                        value: Some(MirOperand::Local(0)),
                    },
                }],
                locals: [(0, MirType::Pointer(Box::new(MirType::Int(32))))]
                    .into_iter()
                    .collect(),
                name_to_local: vec![].into_iter().collect(),
                is_extern: false,
            }],
        };
        let lir = lower_mir_to_lir(&mir).expect("lowering should succeed");
        let text = lir.to_string();
        assert!(text.contains("malloc"));
    }

    #[test]
    fn lower_load_store_instructions() {
        let mir = MirModule {
            name: "main".to_string(),
            imports: Vec::new(),
            structs: Vec::new(),
            globals: vec![],
            functions: vec![MirFunction {
                name: "load_store_test".to_string(),
                type_params: Vec::new(),
                parameters: vec![],
                return_type: MirType::Int(32),
                blocks: vec![MirBasicBlock {
                    id: 0,
                    instructions: vec![
                        MirInstruction::Alloc {
                            dest: 0,
                            ty: MirType::Int(32),
                            size: 4,
                        },
                        MirInstruction::Assign {
                            dest: 1,
                            value: MirOperand::Constant(MirConstant::Int(42)),
                        },
                        MirInstruction::Store {
                            ptr: MirOperand::Local(0),
                            value: MirOperand::Local(1),
                        },
                        MirInstruction::Load {
                            dest: 2,
                            ptr: MirOperand::Local(0),
                        },
                    ],
                    terminator: MirTerminator::Return {
                        value: Some(MirOperand::Local(2)),
                    },
                }],
                locals: [
                    (0, MirType::Pointer(Box::new(MirType::Int(32)))),
                    (1, MirType::Int(32)),
                    (2, MirType::Int(32)),
                ]
                .into_iter()
                .collect(),
                name_to_local: vec![].into_iter().collect(),
                is_extern: false,
            }],
        };
        let lir = lower_mir_to_lir(&mir).expect("lowering should succeed");
        let text = lir.to_string();
        assert!(text.contains("malloc"));
        assert!(text.contains("*"));
    }

    // ==================== Perceus Operations ====================

    #[test]
    fn lower_dup_instruction() {
        let mir = MirModule {
            name: "main".to_string(),
            imports: Vec::new(),
            structs: Vec::new(),
            globals: vec![],
            functions: vec![MirFunction {
                name: "dup_test".to_string(),
                type_params: Vec::new(),
                parameters: vec![],
                return_type: MirType::Int(32),
                blocks: vec![MirBasicBlock {
                    id: 0,
                    instructions: vec![
                        MirInstruction::Assign {
                            dest: 0,
                            value: MirOperand::Constant(MirConstant::Int(42)),
                        },
                        MirInstruction::Dup {
                            dest: 1,
                            src: MirOperand::Local(0),
                        },
                    ],
                    terminator: MirTerminator::Return {
                        value: Some(MirOperand::Local(1)),
                    },
                }],
                locals: [(0, MirType::Int(32)), (1, MirType::Int(32))]
                    .into_iter()
                    .collect(),
                name_to_local: vec![].into_iter().collect(),
                is_extern: false,
            }],
        };
        let lir = lower_mir_to_lir(&mir).expect("lowering should succeed");
        let text = lir.to_string();
        assert!(text.contains("x_perceus_retain"));
    }

    #[test]
    fn lower_drop_instruction() {
        let mir = MirModule {
            name: "main".to_string(),
            imports: Vec::new(),
            structs: Vec::new(),
            globals: vec![],
            functions: vec![MirFunction {
                name: "drop_test".to_string(),
                type_params: Vec::new(),
                parameters: vec![],
                return_type: MirType::Int(32),
                blocks: vec![MirBasicBlock {
                    id: 0,
                    instructions: vec![
                        MirInstruction::Assign {
                            dest: 0,
                            value: MirOperand::Constant(MirConstant::Int(42)),
                        },
                        MirInstruction::Drop {
                            value: MirOperand::Local(0),
                        },
                    ],
                    terminator: MirTerminator::Return {
                        value: Some(MirOperand::Constant(MirConstant::Int(0))),
                    },
                }],
                locals: [(0, MirType::Int(32))].into_iter().collect(),
                name_to_local: vec![].into_iter().collect(),
                is_extern: false,
            }],
        };
        let lir = lower_mir_to_lir(&mir).expect("lowering should succeed");
        let text = lir.to_string();
        assert!(text.contains("x_perceus_release"));
    }

    #[test]
    fn lower_reuse_instruction() {
        let mir = MirModule {
            name: "main".to_string(),
            imports: Vec::new(),
            structs: Vec::new(),
            globals: vec![],
            functions: vec![MirFunction {
                name: "reuse_test".to_string(),
                type_params: Vec::new(),
                parameters: vec![],
                return_type: MirType::Int(32),
                blocks: vec![MirBasicBlock {
                    id: 0,
                    instructions: vec![
                        MirInstruction::Assign {
                            dest: 0,
                            value: MirOperand::Constant(MirConstant::Int(42)),
                        },
                        MirInstruction::Reuse {
                            dest: 1,
                            src: MirOperand::Local(0),
                        },
                    ],
                    terminator: MirTerminator::Return {
                        value: Some(MirOperand::Local(1)),
                    },
                }],
                locals: [(0, MirType::Int(32)), (1, MirType::Int(32))]
                    .into_iter()
                    .collect(),
                name_to_local: vec![].into_iter().collect(),
                is_extern: false,
            }],
        };
        let lir = lower_mir_to_lir(&mir).expect("lowering should succeed");
        let text = lir.to_string();

        // Find the function body - Reuse should generate simple assignment, not function calls
        // The function body should contain "t1 = t0;" style assignment, not x_perceus_retain()
        // We check that within the reuse_test function there's no retain/release call
        let func_start = text.find("reuse_test").expect("function should exist");
        let func_body = &text[func_start..];

        // The function should have assignment like "t1 = t0"
        assert!(func_body.contains("t1 = t0") || func_body.contains("t1=t0"));

        // Reuse should NOT generate a call expression (no parentheses after retain/release in function body)
        // Note: External declarations of retain/release will exist, but no calls in this function
        assert!(!func_body.contains("x_perceus_retain("));
        assert!(!func_body.contains("x_perceus_release("));
    }

    // ==================== Types ====================

    #[test]
    fn lower_float_type() {
        let mir = MirModule {
            name: "main".to_string(),
            imports: Vec::new(),
            structs: Vec::new(),
            globals: vec![],
            functions: vec![MirFunction {
                name: "float_test".to_string(),
                type_params: Vec::new(),
                parameters: vec![],
                return_type: MirType::Float(64),
                blocks: vec![MirBasicBlock {
                    id: 0,
                    instructions: vec![MirInstruction::Assign {
                        dest: 0,
                        value: MirOperand::Constant(MirConstant::Float(1.25)),
                    }],
                    terminator: MirTerminator::Return {
                        value: Some(MirOperand::Local(0)),
                    },
                }],
                locals: [(0, MirType::Float(64))].into_iter().collect(),
                name_to_local: vec![].into_iter().collect(),
                is_extern: false,
            }],
        };
        let lir = lower_mir_to_lir(&mir).expect("lowering should succeed");
        let text = lir.to_string();
        assert!(text.contains("double"));
        assert!(text.contains("1.25"));
    }

    #[test]
    fn lower_bool_type() {
        let mir = MirModule {
            name: "main".to_string(),
            imports: Vec::new(),
            structs: Vec::new(),
            globals: vec![],
            functions: vec![MirFunction {
                name: "bool_test".to_string(),
                type_params: Vec::new(),
                parameters: vec![],
                return_type: MirType::Bool,
                blocks: vec![MirBasicBlock {
                    id: 0,
                    instructions: vec![MirInstruction::Assign {
                        dest: 0,
                        value: MirOperand::Constant(MirConstant::Bool(true)),
                    }],
                    terminator: MirTerminator::Return {
                        value: Some(MirOperand::Local(0)),
                    },
                }],
                locals: [(0, MirType::Bool)].into_iter().collect(),
                name_to_local: vec![].into_iter().collect(),
                is_extern: false,
            }],
        };
        let lir = lower_mir_to_lir(&mir).expect("lowering should succeed");
        let text = lir.to_string();
        assert!(text.contains("bool"));
        assert!(text.contains("true"));
    }

    #[test]
    fn lower_string_type() {
        let mir = MirModule {
            name: "main".to_string(),
            imports: Vec::new(),
            structs: Vec::new(),
            globals: vec![],
            functions: vec![MirFunction {
                name: "string_test".to_string(),
                type_params: Vec::new(),
                parameters: vec![],
                return_type: MirType::String,
                blocks: vec![MirBasicBlock {
                    id: 0,
                    instructions: vec![MirInstruction::Assign {
                        dest: 0,
                        value: MirOperand::Constant(MirConstant::String(
                            "Hello, World!".to_string(),
                        )),
                    }],
                    terminator: MirTerminator::Return {
                        value: Some(MirOperand::Local(0)),
                    },
                }],
                locals: [(0, MirType::String)].into_iter().collect(),
                name_to_local: vec![].into_iter().collect(),
                is_extern: false,
            }],
        };
        let lir = lower_mir_to_lir(&mir).expect("lowering should succeed");
        let text = lir.to_string();
        assert!(text.contains("Hello, World!"));
    }

    // ==================== External Functions ====================

    #[test]
    fn lower_extern_function() {
        let mir = MirModule {
            name: "main".to_string(),
            imports: Vec::new(),
            structs: Vec::new(),
            globals: vec![],
            functions: vec![MirFunction {
                name: "external_func".to_string(),
                type_params: Vec::new(),
                parameters: vec![MirParameter {
                    name: "x".to_string(),
                    ty: MirType::Int(32),
                    index: 0,
                }],
                return_type: MirType::Int(32),
                blocks: vec![],
                locals: vec![].into_iter().collect(),
                name_to_local: vec![].into_iter().collect(),
                is_extern: true,
            }],
        };
        let lir = lower_mir_to_lir(&mir).expect("lowering should succeed");
        let text = lir.to_string();
        assert!(text.contains("external"));
        assert!(text.contains("external_func"));
    }

    // ==================== Imports ====================

    #[test]
    fn lower_import_declaration() {
        use x_mir::Import;

        let mir = MirModule {
            name: "main".to_string(),
            imports: vec![Import {
                module_path: "std.io".to_string(),
                symbols: vec![("println".to_string(), None)],
                import_all: false,
            }],
            globals: vec![],
            functions: vec![],
            structs: vec![],
        };
        let lir = lower_mir_to_lir(&mir).expect("lowering should succeed");
        let text = lir.to_string();
        assert!(text.contains("import"));
        assert!(text.contains("std.io"));
        assert!(text.contains("println"));
    }

    #[test]
    fn lower_import_all() {
        use x_mir::Import;

        let mir = MirModule {
            name: "main".to_string(),
            imports: vec![Import {
                module_path: "std.collections".to_string(),
                symbols: vec![],
                import_all: true,
            }],
            globals: vec![],
            functions: vec![],
            structs: vec![],
        };
        let lir = lower_mir_to_lir(&mir).expect("lowering should succeed");
        let text = lir.to_string();
        assert!(text.contains("import"));
        assert!(text.contains("std.collections"));
        assert!(text.contains("*"));
    }

    // ==================== Field Access ====================

    #[test]
    fn lower_field_access() {
        let mir = MirModule {
            name: "main".to_string(),
            imports: Vec::new(),
            structs: Vec::new(),
            globals: vec![],
            functions: vec![MirFunction {
                name: "field_test".to_string(),
                type_params: Vec::new(),
                parameters: vec![],
                return_type: MirType::Int(32),
                blocks: vec![MirBasicBlock {
                    id: 0,
                    instructions: vec![
                        MirInstruction::Assign {
                            dest: 0,
                            value: MirOperand::Constant(MirConstant::Int(0)), // placeholder for struct
                        },
                        MirInstruction::FieldAccess {
                            dest: 1,
                            object: MirOperand::Local(0),
                            field: "x".to_string(),
                        },
                    ],
                    terminator: MirTerminator::Return {
                        value: Some(MirOperand::Local(1)),
                    },
                }],
                locals: [(0, MirType::Int(32)), (1, MirType::Int(32))]
                    .into_iter()
                    .collect(),
                name_to_local: vec![].into_iter().collect(),
                is_extern: false,
            }],
        };
        let lir = lower_mir_to_lir(&mir).expect("lowering should succeed");
        let text = lir.to_string();
        assert!(text.contains(".x"));
    }

    // ==================== Array Access ====================

    #[test]
    fn lower_array_access() {
        let mir = MirModule {
            name: "main".to_string(),
            imports: Vec::new(),
            structs: Vec::new(),
            globals: vec![],
            functions: vec![MirFunction {
                name: "array_test".to_string(),
                type_params: Vec::new(),
                parameters: vec![],
                return_type: MirType::Int(32),
                blocks: vec![MirBasicBlock {
                    id: 0,
                    instructions: vec![
                        MirInstruction::Assign {
                            dest: 0,
                            value: MirOperand::Constant(MirConstant::Int(0)), // placeholder for array
                        },
                        MirInstruction::Assign {
                            dest: 1,
                            value: MirOperand::Constant(MirConstant::Int(0)), // index
                        },
                        MirInstruction::ArrayAccess {
                            dest: 2,
                            array: MirOperand::Local(0),
                            index: MirOperand::Local(1),
                        },
                    ],
                    terminator: MirTerminator::Return {
                        value: Some(MirOperand::Local(2)),
                    },
                }],
                locals: [
                    (0, MirType::Int(32)),
                    (1, MirType::Int(32)),
                    (2, MirType::Int(32)),
                ]
                .into_iter()
                .collect(),
                name_to_local: vec![].into_iter().collect(),
                is_extern: false,
            }],
        };
        let lir = lower_mir_to_lir(&mir).expect("lowering should succeed");
        let text = lir.to_string();
        assert!(text.contains("[") && text.contains("]"));
    }

    // ==================== Cast ====================

    #[test]
    fn lower_cast_instruction() {
        let mir = MirModule {
            name: "main".to_string(),
            imports: Vec::new(),
            structs: Vec::new(),
            globals: vec![],
            functions: vec![MirFunction {
                name: "cast_test".to_string(),
                type_params: Vec::new(),
                parameters: vec![],
                return_type: MirType::Float(64),
                blocks: vec![MirBasicBlock {
                    id: 0,
                    instructions: vec![
                        MirInstruction::Assign {
                            dest: 0,
                            value: MirOperand::Constant(MirConstant::Int(42)),
                        },
                        MirInstruction::Cast {
                            dest: 1,
                            value: MirOperand::Local(0),
                            ty: MirType::Float(64),
                        },
                    ],
                    terminator: MirTerminator::Return {
                        value: Some(MirOperand::Local(1)),
                    },
                }],
                locals: [(0, MirType::Int(32)), (1, MirType::Float(64))]
                    .into_iter()
                    .collect(),
                name_to_local: vec![].into_iter().collect(),
                is_extern: false,
            }],
        };
        let lir = lower_mir_to_lir(&mir).expect("lowering should succeed");
        let text = lir.to_string();
        assert!(text.contains("cast") || text.contains("double"));
    }

    // ==================== Unreachable ====================

    #[test]
    fn lower_unreachable_terminator() {
        let mir = MirModule {
            name: "main".to_string(),
            imports: Vec::new(),
            structs: Vec::new(),
            globals: vec![],
            functions: vec![MirFunction {
                name: "unreachable_test".to_string(),
                type_params: Vec::new(),
                parameters: vec![],
                return_type: MirType::Int(32),
                blocks: vec![MirBasicBlock {
                    id: 0,
                    instructions: vec![],
                    terminator: MirTerminator::Unreachable,
                }],
                locals: vec![].into_iter().collect(),
                name_to_local: vec![].into_iter().collect(),
                is_extern: false,
            }],
        };
        let lir = lower_mir_to_lir(&mir).expect("lowering should succeed");
        let text = lir.to_string();
        assert!(text.contains("abort"));
    }

    // ==================== Helper Functions ====================

    fn create_binary_op_module(op: x_mir::MirBinOp, left: i64, right: i64) -> MirModule {
        MirModule {
            name: "main".to_string(),
            imports: Vec::new(),
            structs: Vec::new(),
            globals: vec![],
            functions: vec![MirFunction {
                name: "binary_op".to_string(),
                type_params: Vec::new(),
                parameters: vec![],
                return_type: MirType::Int(32),
                blocks: vec![MirBasicBlock {
                    id: 0,
                    instructions: vec![
                        MirInstruction::Assign {
                            dest: 0,
                            value: MirOperand::Constant(MirConstant::Int(left)),
                        },
                        MirInstruction::Assign {
                            dest: 1,
                            value: MirOperand::Constant(MirConstant::Int(right)),
                        },
                        MirInstruction::BinaryOp {
                            dest: 2,
                            op,
                            left: MirOperand::Local(0),
                            right: MirOperand::Local(1),
                        },
                    ],
                    terminator: MirTerminator::Return {
                        value: Some(MirOperand::Local(2)),
                    },
                }],
                locals: [
                    (0, MirType::Int(32)),
                    (1, MirType::Int(32)),
                    (2, MirType::Int(32)),
                ]
                .into_iter()
                .collect(),
                name_to_local: vec![].into_iter().collect(),
                is_extern: false,
            }],
        }
    }
}
