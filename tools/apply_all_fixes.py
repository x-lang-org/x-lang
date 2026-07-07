#!/usr/bin/env python3
"""Apply all working Zig backend fixes for X language benchmarks."""
with open('/home/xiongdi/workspace/x-lang/compiler/x-codegen-zig/src/lib.rs', 'r') as f:
    c = f.read()

# ====== Fix 1: Char literal escaping (both expr and pattern) ======
old = """x_lir::Literal::Char(c) => Ok(format!("'{}'", c)),
                x_lir::Literal::Bool(b) => Ok(format!("{}", b)),
                _ => Ok("_".to_string()),"""
new = """x_lir::Literal::Char(c) => {
                    let escaped = match c {
                        '\\n' => "\\\\n",
                        '\\r' => "\\\\r",
                        '\\t' => "\\\\t",
                        '\\\\' => "\\\\\\\\",
                        '\\'' => "\\\\'",
                        _ if c.is_ascii_graphic() || *c == ' ' => {
                            return Ok(format!("'{}'", c));
                        }
                        _ => {
                            return Ok(format!("'\\\\x{:02x}'", *c as u8));
                        }
                    };
                    Ok(format!("'{}'", escaped))
                },
                x_lir::Literal::Bool(b) => Ok(format!("{}", b)),
                _ => Ok("_".to_string()),"""
c = c.replace(old, new)

# Expression char literal
old2 = """x_lir::Literal::Char(c) => Ok(format!("'{}'", c)),
                x_lir::Literal::Bool(b) => Ok(format!("{}", b)),
                x_lir::Literal::NullPointer => Ok("null".to_string()),"""
new2 = """x_lir::Literal::Char(c) => {
                    let escaped = match c {
                        '\\n' => "\\\\n",
                        '\\r' => "\\\\r",
                        '\\t' => "\\\\t",
                        '\\\\' => "\\\\\\\\",
                        '\\'' => "\\\\'",
                        _ if c.is_ascii_graphic() || *c == ' ' => {
                            return Ok(format!("'{}'", c));
                        }
                        _ => {
                            return Ok(format!("'\\\\x{:02x}'", *c as u8));
                        }
                    };
                    Ok(format!("'{}'", escaped))
                },
                x_lir::Literal::Bool(b) => Ok(format!("{}", b)),
                x_lir::Literal::NullPointer => Ok("null".to_string()),"""
c = c.replace(old2, new2)

# ====== Fix 2: XValue type mapping ======
c = c.replace(
    'x_lir::Type::Named(name) => name.clone(),',
    """x_lir::Type::Named(name) => {
                if matches!(name.as_str(), "XValue" | "Option" | "Result" | "Box" | "Ref")
                    || (name.len() == 1 && name.chars().next().map_or(false, |ch| ch.is_uppercase()))
                { "i32".to_string() } else { name.clone() }
            }"""
)

# ====== Fix 3: Cast - use @floatFromInt for float targets ======
old3 = """x_lir::Expression::Cast(type_, expr) => {
                let expr_str = self.emit_lir_expression(expr)?;
                let type_str = self.emit_lir_type(type_);
                Ok(format!("@as({}, {})", type_str, expr_str))
            }"""
new3 = """x_lir::Expression::Cast(type_, expr) => {
                let expr_str = self.emit_lir_expression(expr)?;
                let type_str = self.emit_lir_type(type_);
                if matches!(type_str.as_str(), "f32" | "f64" | "f128") {
                    Ok(format!("@floatFromInt({})", expr_str))
                } else {
                    Ok(format!("@as({}, {})", type_str, expr_str))
                }
            }"""
c = c.replace(old3, new3)

# ====== Fix 4: __index__ builtin (use x_list_get for XValue types) ======
c = c.replace(
    """"len" => format!("{}.len", args.first().map(|s| s.as_str()).unwrap_or("null")),
            _ => {""",
    """"len" => format!("{}.len", args.first().map(|s| s.as_str()).unwrap_or("null")),
            "__index__" => {
                if args.len() == 2 {
                    format!("x_list_get({}, @intCast({}))", args[0], args[1])
                } else { "null".to_string() }
            }
            _ => {"""
)

# ====== Fix 5: Division and Modulo operators (use function syntax) ======
c = c.replace('x_lir::BinaryOp::Divide => "/",', 'x_lir::BinaryOp::Divide => "@divTrunc",')
c = c.replace('x_lir::BinaryOp::Modulo => "%",', 'x_lir::BinaryOp::Modulo => "@mod",')

old_bin = 'Ok(format!("({} {} {})", lhs_str, op_str, rhs_str))'
new_bin = """let is_func = matches!(op, x_lir::BinaryOp::Divide | x_lir::BinaryOp::Modulo);
                if is_func { Ok(format!("{}({}, {})", op_str, lhs_str, rhs_str)) }
                else { Ok(format!("({} {} {})", lhs_str, op_str, rhs_str)) }"""
c = c.replace(old_bin, new_bin)

# ====== Fix 6: comptime_int - temp var type tracking ======
c = c.replace(
    'temp_use_counts: std::collections::HashMap<String, usize>,\n    used_params:',
    'temp_use_counts: std::collections::HashMap<String, usize>,\n    temp_var_types: std::collections::HashMap<String, String>,\n    used_params:'
)
c = c.replace(
    'temp_use_counts: std::collections::HashMap::new(),\n            used_params:',
    'temp_use_counts: std::collections::HashMap::new(),\n            temp_var_types: std::collections::HashMap::new(),\n            used_params:'
)
c = c.replace(
    'self.temp_use_counts.clear();\n        self.used_params.clear();',
    'self.temp_use_counts.clear();\n        self.temp_var_types.clear();\n        self.used_params.clear();'
)

old_col = """self.temp_assignment_counts = Self::collect_temp_assignment_counts(&func.body);
        self.temp_use_counts = Self::collect_temp_use_counts(&func.body);
        self.used_params"""
new_col = """self.temp_assignment_counts = Self::collect_temp_assignment_counts(&func.body);
        self.temp_use_counts = Self::collect_temp_use_counts(&func.body);
        self.temp_var_types.clear();
        for stmt in &func.body.statements {
            if let x_lir::Statement::Variable(var) = stmt {
                if var.name.starts_with('t') && var.name.len() > 1
                    && var.name[1..].chars().all(|c| c.is_ascii_digit())
                { self.temp_var_types.insert(var.name.clone(), self.emit_lir_type(&var.type_)); }
            }
        }
        self.used_params"""
c = c.replace(old_col, new_col)

old_lazy = """if use_count == 0 {
                            self.line(&format!("_ = {};", value_part))?;
                        } else if self.declared_temp_vars.insert(var_name.clone()) {
                            let decl_keyword = if assignment_count > 1 { "var" } else { "const" };
                            self.line(&format!("{} {} = {};", decl_keyword, var_name, value_part))?;
                        } else {
                            self.line(&format!("{} = {};", var_name, value_part))?;
                        }"""
new_lazy = """if use_count == 0 {
                            self.line(&format!("_ = {};", value_part))?;
                        } else if self.declared_temp_vars.insert(var_name.clone()) {
                            let decl_keyword = if assignment_count > 1 { "var" } else { "const" };
                            let lir_name = var_name.strip_prefix('_').unwrap_or(&var_name);
                            if let Some(type_str) = self.temp_var_types.get(lir_name) {
                                self.line(&format!("{} {} : {} = {};", decl_keyword, var_name, type_str, value_part))?;
                            } else {
                                self.line(&format!("{} {} = {};", decl_keyword, var_name, value_part))?;
                            }
                        } else {
                            self.line(&format!("{} = {};", var_name, value_part))?;
                        }"""
c = c.replace(old_lazy, new_lazy)

# ====== Fix 7: XValue runtime wrappers in Zig header ======
old_hdr = """        self.line("}")?;
        self.line("")?;

        // HTTP Server runtime"""
new_hdr = """        self.line("}")?;
        self.line("")?;

        // XValue wrappers
        self.line("fn __x_list_push(list: i32, value: anytype) void {")?;
        self.indent();
        self.line("const xv = switch (@typeInfo(@TypeOf(value))) {")?;
        self.indent();
        self.line(".int, .comptime_int => x_from_int(@intCast(value)),")?;
        self.line(".float, .comptime_float => x_from_double(@floatCast(value)),")?;
        self.line("else => @compileError(\\\"push unsupported\\\"),")?;
        self.dedent();
        self.line("};")?;
        self.line("_ = x_list_push(list, xv);")?;
        self.dedent();
        self.line("}")?;
        self.line("")?;

        self.line("fn __x_list_get(list: i32, idx: anytype) i64 {")?;
        self.indent();
        self.line("return x_as_int(x_list_get(list, @intCast(idx)));")?;
        self.dedent();
        self.line("}")?;
        self.line("")?;

        // HTTP Server runtime"""
c = c.replace(old_hdr, new_hdr)

# ====== Fix 8: Deduplicate extern functions ======
c = c.replace(
    """        for f in &extern_funcs {
            self.emit_lir_extern_function(f)?;
        }""",
    """        let mut seen_ext: std::collections::HashSet<String> = std::collections::HashSet::new();
        for f in &extern_funcs {
            if seen_ext.insert(f.name.clone()) { self.emit_lir_extern_function(f)?; }
        }"""
)

# ====== Fix 9: XValue method call transform ======
old_pre = """    fn collect_temp_assignment_counts("""
new_pre = """    fn transform_xvalue_method_calls(mut block: x_lir::Block, var_types: &std::collections::HashMap<String, String>) -> x_lir::Block {
        use x_lir::{Expression, Statement};
        let known = ["push", "get", "length", "put"];
        let mut out: Vec<Statement> = Vec::with_capacity(block.statements.len());
        let mut i = 0;
        while i < block.statements.len() {
            if let Statement::Expression(Expression::Assign(lhs, rhs)) = &block.statements[i] {
                if let (Expression::Variable(tn), Expression::Member(obj, m)) = (lhs.as_ref(), rhs.as_ref()) {
                    if known.contains(&m.as_str()) {
                        let mut found = false;
                        for j in (i+1)..std::cmp::min(i+4, block.statements.len()) {
                            let call_match: Option<_> = match &block.statements[j] {
                                Statement::Expression(Expression::Call(c, a)) if matches!(c.as_ref(), Expression::Variable(n) if n == tn) =>
                                    Some((None, a.clone())),
                                Statement::Expression(Expression::Assign(r, ce)) => {
                                    if let Expression::Call(c, a) = ce.as_ref() {
                                        if matches!(c.as_ref(), Expression::Variable(n) if n == tn) {
                                            Some((Some(r.as_ref().clone()), a.clone()))
                                        } else { None }
                                    } else { None }
                                }
                                _ => None,
                            };
                            if let Some((res, args)) = call_match {
                                let rf = match m.as_str() {
                                    "push" => "__x_list_push",
                                    "get" => "__x_list_get",
                                    "length" => {
                                        if let Expression::Variable(vn) = obj.as_ref() {
                                            if var_types.get(vn.as_str()).map_or(false, |t| t == "[*:0]const u8") {
                                                "std.mem.len"
                                            } else { "x_list_len" }
                                        } else { "x_list_len" }
                                    }
                                    "put" => "x_map_put",
                                    _ => unreachable!(),
                                };
                                let mut na = vec![obj.as_ref().clone()];
                                na.extend(args);
                                let ce = Expression::Call(Box::new(Expression::Variable(rf.to_string())), na);
                                if let Some(r) = res {
                                    out.push(Statement::Expression(Expression::Assign(Box::new(r), Box::new(ce))));
                                } else {
                                    out.push(Statement::Expression(ce));
                                }
                                i = j + 1;
                                found = true;
                                break;
                            }
                        }
                        if found { continue; }
                    }
                }
            }
            out.push(block.statements[i].clone());
            i += 1;
        }
        block.statements = out;
        block
    }

    fn collect_temp_assignment_counts("""
c = c.replace(old_pre, new_pre)

# Apply transform
c = c.replace(
    'self.emit_lir_block(&func.body)?;',
    'let transformed_body = Self::transform_xvalue_method_calls(func.body.clone(), &self.temp_var_types);\n        self.emit_lir_block(&transformed_body)?;'
)

with open('/home/xiongdi/workspace/x-lang/compiler/x-codegen-zig/src/lib.rs', 'w') as f:
    f.write(c)
print('All 9 fixes applied!')
