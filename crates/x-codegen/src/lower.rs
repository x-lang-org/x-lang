// AST → LLVM  lowering：为当前支持的子集生成可链接的 main 与用户函数

use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::values::{FunctionValue, IntValue, PointerValue};
use inkwell::AddressSpace;
use std::collections::HashMap;
use x_parser::ast::{
    BinaryOp, Block, Expression, Literal, Program, Statement,
};

use crate::{CodeGenConfig, CodeGenError, write_object_or_ir};

pub fn generate_code(program: &Program, config: &CodeGenConfig) -> Result<Vec<u8>, CodeGenError> {
    let context = Context::create();
    let module = context.create_module("x_program");
    let builder = context.create_builder();
    let i64_type = context.i64_type();
    let i32_type = context.i32_type();
    let i1_type = context.bool_type();

    // 声明 printf: i32 (i8*, ...)
    let i8_ptr = context.i8_type().ptr_type(AddressSpace::default());
    let printf_type = i32_type.fn_type(&[i8_ptr.into()], true);
    let printf = module.add_function("printf", printf_type, None);

    // 格式串 "%lld\n" 用于 print(整数)
    let fmt_bytes = "%lld\n".as_bytes();
    let fmt_len = fmt_bytes.len() as u32;
    let fmt_str = context.const_string(fmt_bytes, false);
    let arr_type = context.i8_type().array_type(fmt_len);
    let fmt_global = module.add_global(arr_type, None, "fmt_lld");
    fmt_global.set_initializer(&fmt_str);
    fmt_global.set_unnamed_addr(true);
    fmt_global.set_constant(true);
    let zero = context.i32_type().const_int(0, false);
    let fmt_ptr = unsafe {
        builder.build_gep(
            arr_type,
            fmt_global.as_pointer_value(),
            &[zero, zero],
            "fmt_ptr",
        ).map_err(|e| CodeGenError::GenerationError(e.to_string()))?
    };

    let mut func_map: HashMap<String, FunctionValue> = HashMap::new();

    // 第一遍：创建所有 X 函数（名称为 x_main 或原名）
    for decl in &program.declarations {
        if let x_parser::ast::Declaration::Function(f) = decl {
            let name = if f.name == "main" { "x_main" } else { f.name.as_str() };
            let param_count = f.parameters.len();
            let fn_type = i64_type.fn_type(
                &std::iter::repeat(i64_type.into()).take(param_count).collect::<Vec<_>>(),
                false,
            );
            let func = module.add_function(name, fn_type, None);
            func_map.insert(f.name.clone(), func);
        }
    }

    // 第二遍：为每个函数生成函数体
    for decl in &program.declarations {
        if let x_parser::ast::Declaration::Function(f) = decl {
            let name = if f.name == "main" { "x_main" } else { f.name.as_str() };
            let func = *func_map.get(&f.name).unwrap();
            let entry = context.append_basic_block(func, "entry");
            builder.position_at_end(entry);

            let mut locals: HashMap<String, PointerValue> = HashMap::new();
            for (i, param) in f.parameters.iter().enumerate() {
                let param_val = func.get_nth_param(i as u32).unwrap();
                let alloca = builder.build_alloca(i64_type, &param.name).map_err(|e| CodeGenError::GenerationError(e.to_string()))?;
                builder.build_store(alloca, param_val).map_err(|e| CodeGenError::GenerationError(e.to_string()))?;
                locals.insert(param.name.clone(), alloca);
            }

            match lower_block(
                &context,
                &module,
                &builder,
                &f.body,
                &mut locals,
                &func_map,
                &fmt_ptr,
                &printf,
                &i64_type,
                &i1_type,
            ) {
                Ok(()) => {}
                Err(e) => return Err(e),
            }

            // 若块末没有 return，补 ret 0
            if entry.get_last_instruction().is_none()
                || !entry.get_last_instruction().unwrap().is_terminator()
            {
                let zero = i64_type.const_int(0, false);
                builder.build_return(Some(&zero));
            }
        }
    }

    // C 入口 main：调用 x_main，返回 (i32)x_main()
    let main_type = i32_type.fn_type(&[], false);
    let main_func = module.add_function("main", main_type, None);
    let main_entry = context.append_basic_block(main_func, "entry");
    builder.position_at_end(main_entry);
    if let Some(&x_main) = func_map.get("main") {
        let ret = builder.build_call(x_main, &[], "ret").map_err(|e| CodeGenError::GenerationError(e.to_string()))?;
        let ret_val = ret.try_as_basic_value().unwrap_basic().into_int_value();
        let i64_ret = ret_val;
        let i32_ret = builder.build_int_truncate(i64_ret, i32_type, "ret32").map_err(|e| CodeGenError::GenerationError(e.to_string()))?;
        builder.build_return(Some(&i32_ret));
    } else {
        builder.build_return(Some(&i32_type.const_int(0, false)));
    }

    // 优化（inkwell 0.8 PassManager API 有变，暂时跳过以通过编译）
    // let pass_manager = PassManager::create(());
    // pass_manager.add_instruction_combining_pass();
    // pass_manager.add_reassociation_pass();
    // pass_manager.add_gvn_pass();
    // pass_manager.add_cfg_simplification_pass();
    // pass_manager.run_on(&module);

    write_object_or_ir(&module, config)
}

fn lower_block<'ctx>(
    context: &'ctx Context,
    module: &Module<'ctx>,
    builder: &Builder<'ctx>,
    block: &Block,
    locals: &mut HashMap<String, PointerValue<'ctx>>,
    func_map: &HashMap<String, FunctionValue<'ctx>>,
    fmt_ptr: &PointerValue<'ctx>,
    printf: &FunctionValue<'ctx>,
    i64_type: &inkwell::types::IntType<'ctx>,
    i1_type: &inkwell::types::IntType<'ctx>,
) -> Result<(), CodeGenError> {
    for stmt in &block.statements {
        lower_statement(
            context,
            module,
            builder,
            stmt,
            locals,
            func_map,
            fmt_ptr,
            printf,
            i64_type,
            i1_type,
        )?;
    }
    Ok(())
}

fn lower_statement<'ctx>(
    context: &'ctx Context,
    module: &Module<'ctx>,
    builder: &Builder<'ctx>,
    stmt: &Statement,
    locals: &mut HashMap<String, PointerValue<'ctx>>,
    func_map: &HashMap<String, FunctionValue<'ctx>>,
    fmt_ptr: &PointerValue<'ctx>,
    printf: &FunctionValue<'ctx>,
    i64_type: &inkwell::types::IntType<'ctx>,
    i1_type: &inkwell::types::IntType<'ctx>,
) -> Result<(), CodeGenError> {
    match stmt {
        Statement::Variable(var) => {
            let alloca = builder.build_alloca(*i64_type, &var.name).map_err(|e| CodeGenError::GenerationError(e.to_string()))?;
            let init = if let Some(ref e) = var.initializer {
                lower_expr(context, module, builder, e, locals, func_map, fmt_ptr, printf, i64_type, i1_type)?
            } else {
                i64_type.const_int(0, false)
            };
            builder.build_store(alloca, init).map_err(|e| CodeGenError::GenerationError(e.to_string()))?;
            locals.insert(var.name.clone(), alloca);
        }
        Statement::Expression(expr) => {
            let _ = lower_expr(context, module, builder, expr, locals, func_map, fmt_ptr, printf, i64_type, i1_type)?;
        }
        Statement::Return(Some(expr)) => {
            let v = lower_expr(context, module, builder, expr, locals, func_map, fmt_ptr, printf, i64_type, i1_type)?;
            builder.build_return(Some(&v));
        }
        Statement::Return(None) => {
            builder.build_return(Some(&i64_type.const_int(0, false)));
        }
        Statement::If(if_stmt) => {
            let cond = lower_expr(
                context,
                module,
                builder,
                &if_stmt.condition,
                locals,
                func_map,
                fmt_ptr,
                printf,
                i64_type,
                i1_type,
            )?;
            let cond_bool = builder
                .build_int_compare(
                    inkwell::IntPredicate::NE,
                    cond,
                    i64_type.const_int(0, false),
                    "cond",
                )
                .map_err(|e| CodeGenError::GenerationError(e.to_string()))?;

            let then_bb = context.append_basic_block(builder.get_insert_block().unwrap().get_parent().unwrap(), "then");
            let else_bb = if if_stmt.else_block.is_some() {
                context.append_basic_block(builder.get_insert_block().unwrap().get_parent().unwrap(), "else")
            } else {
                context.append_basic_block(builder.get_insert_block().unwrap().get_parent().unwrap(), "endif")
            };
            let merge_bb = context.append_basic_block(builder.get_insert_block().unwrap().get_parent().unwrap(), "merge");

            if if_stmt.else_block.is_some() {
                builder.build_conditional_branch(cond_bool, then_bb, else_bb);
            } else {
                builder.build_conditional_branch(cond_bool, then_bb, merge_bb);
            }

            builder.position_at_end(then_bb);
            lower_block(
                context,
                module,
                builder,
                &if_stmt.then_block,
                locals,
                func_map,
                fmt_ptr,
                printf,
                i64_type,
                i1_type,
            )?;
            if builder.get_insert_block().unwrap().get_last_instruction().map(|i| !i.is_terminator()).unwrap_or(true) {
                builder.build_unconditional_branch(merge_bb);
            }

            if let Some(ref else_block) = if_stmt.else_block {
                builder.position_at_end(else_bb);
                lower_block(
                    context,
                    module,
                    builder,
                    else_block,
                    locals,
                    func_map,
                    fmt_ptr,
                    printf,
                    i64_type,
                    i1_type,
                )?;
                if builder.get_insert_block().unwrap().get_last_instruction().map(|i| !i.is_terminator()).unwrap_or(true) {
                    builder.build_unconditional_branch(merge_bb);
                }
            } else if if_stmt.else_block.is_some() {
                builder.position_at_end(else_bb);
                builder.build_unconditional_branch(merge_bb);
            }

            builder.position_at_end(merge_bb);
        }
        _ => return Err(CodeGenError::GenerationError(format!("未实现的语句: {:?}", stmt))),
    }
    Ok(())
}

fn lower_expr<'ctx>(
    context: &'ctx Context,
    module: &Module<'ctx>,
    builder: &Builder<'ctx>,
    expr: &Expression,
    locals: &HashMap<String, PointerValue<'ctx>>,
    func_map: &HashMap<String, FunctionValue<'ctx>>,
    fmt_ptr: &PointerValue<'ctx>,
    printf: &FunctionValue<'ctx>,
    i64_type: &inkwell::types::IntType<'ctx>,
    i1_type: &inkwell::types::IntType<'ctx>,
) -> Result<IntValue<'ctx>, CodeGenError> {
    match expr {
        Expression::Literal(Literal::Integer(i)) => {
            Ok(i64_type.const_int(*i as u64, true))
        }
        Expression::Literal(Literal::Float(_)) => {
            Err(CodeGenError::GenerationError("float 字面量 codegen 未实现".to_string()))
        }
        Expression::Literal(Literal::Boolean(b)) => {
            Ok(i64_type.const_int(if *b { 1 } else { 0 }, false))
        }
        Expression::Literal(_) => Err(CodeGenError::GenerationError("该字面量 codegen 未实现".to_string())),
        Expression::Variable(name) => {
            let ptr = locals.get(name).ok_or_else(|| CodeGenError::GenerationError(format!("未定义变量: {}", name)))?;
            Ok(builder.build_load(*i64_type, *ptr, name).map_err(|e| CodeGenError::GenerationError(e.to_string()))?.into_int_value())
        }
        Expression::Binary(op, left, right) => {
            let l = lower_expr(context, module, builder, left, locals, func_map, fmt_ptr, printf, i64_type, i1_type)?;
            let r = lower_expr(context, module, builder, right, locals, func_map, fmt_ptr, printf, i64_type, i1_type)?;
            let v = match op {
                BinaryOp::Add => builder.build_int_add(l, r, "add").map_err(|e| CodeGenError::GenerationError(e.to_string()))?,
                BinaryOp::Sub => builder.build_int_sub(l, r, "sub").map_err(|e| CodeGenError::GenerationError(e.to_string()))?,
                BinaryOp::Mul => builder.build_int_mul(l, r, "mul").map_err(|e| CodeGenError::GenerationError(e.to_string()))?,
                BinaryOp::Div => builder.build_int_signed_div(l, r, "div").map_err(|e| CodeGenError::GenerationError(e.to_string()))?,
                BinaryOp::Mod => builder.build_int_signed_rem(l, r, "mod").map_err(|e| CodeGenError::GenerationError(e.to_string()))?,
                BinaryOp::Less => builder.build_int_compare(inkwell::IntPredicate::SLT, l, r, "lt").map_err(|e| CodeGenError::GenerationError(e.to_string()))?,
                BinaryOp::LessEqual => builder.build_int_compare(inkwell::IntPredicate::SLE, l, r, "le").map_err(|e| CodeGenError::GenerationError(e.to_string()))?,
                BinaryOp::Greater => builder.build_int_compare(inkwell::IntPredicate::SGT, l, r, "gt").map_err(|e| CodeGenError::GenerationError(e.to_string()))?,
                BinaryOp::GreaterEqual => builder.build_int_compare(inkwell::IntPredicate::SGE, l, r, "ge").map_err(|e| CodeGenError::GenerationError(e.to_string()))?,
                BinaryOp::Equal => builder.build_int_compare(inkwell::IntPredicate::EQ, l, r, "eq").map_err(|e| CodeGenError::GenerationError(e.to_string()))?,
                BinaryOp::NotEqual => builder.build_int_compare(inkwell::IntPredicate::NE, l, r, "ne").map_err(|e| CodeGenError::GenerationError(e.to_string()))?,
                _ => return Err(CodeGenError::GenerationError(format!("未实现二元运算: {:?}", op))),
            };
            let iv = v;
            if iv.get_type() == *i1_type {
                Ok(builder.build_int_z_extend(iv, *i64_type, "zext").map_err(|e| CodeGenError::GenerationError(e.to_string()))?)
            } else {
                Ok(iv)
            }
        }
        Expression::Call(callee, args) => {
            if let Expression::Variable(ref name) = callee.as_ref() {
                if name == "print" {
                    if let Some(arg) = args.first() {
                        let v = lower_expr(context, module, builder, arg, locals, func_map, fmt_ptr, printf, i64_type, i1_type)?;
                        builder.build_call(*printf, &[(*fmt_ptr).into(), v.into()], "");
                    }
                    return Ok(i64_type.const_int(0, false));
                }
                let func = func_map.get(name).ok_or_else(|| CodeGenError::GenerationError(format!("未定义函数: {}", name)))?;
                let arg_vals: Vec<inkwell::values::BasicValueEnum<'ctx>> = args
                    .iter()
                    .map(|a| lower_expr(context, module, builder, a, locals, func_map, fmt_ptr, printf, i64_type, i1_type).map(|v| v.into()))
                    .collect::<Result<Vec<_>, _>>()?;
                let call_args: Vec<inkwell::values::BasicMetadataValueEnum<'ctx>> =
                    arg_vals.iter().map(|v| (*v).into()).collect();
                let call = builder
                    .build_call(*func, call_args.as_slice(), "call")
                    .map_err(|e| CodeGenError::GenerationError(e.to_string()))?;
                let ret = call.try_as_basic_value().unwrap_basic().into_int_value();
                return Ok(ret);
            }
            Err(CodeGenError::GenerationError("仅支持命名函数调用".to_string()))
        }
        Expression::Parenthesized(inner) => {
            lower_expr(context, module, builder, inner, locals, func_map, fmt_ptr, printf, i64_type, i1_type)
        }
        _ => Err(CodeGenError::GenerationError(format!("未实现表达式: {:?}", expr))),
    }
}
