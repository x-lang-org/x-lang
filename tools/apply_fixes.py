#!/usr/bin/env python3
"""Apply all Zig backend fixes in one pass."""
import re

with open('/home/xiongdi/workspace/x-lang/compiler/x-codegen-zig/src/lib.rs', 'r') as f:
    c = f.read()

# ============================================================
# Fix 1: Character literal escaping (expressions)
# ============================================================
old = 'x_lir::Literal::Char(c) => Ok(format!("\'{}\'", c)),'
new = '''x_lir::Literal::Char(c) => {
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
                }'''
c = c.replace(old, new)

# Fix 1b: Character literal escaping (patterns)
old2 = '''x_lir::Literal::Char(c) => Ok(format!("'{}'", c)),'''
# Need to find the one in Pattern context - let's use context
old2ctx = '''x_lir::Literal::String(s) => Ok(format!("\"{}\"", s)),
                x_lir::Literal::Char(c) => Ok(format!("'{}'", c)),
                x_lir::Literal::Bool(b) => Ok(format!("{}", b)),
                _ => Ok("_".to_string()),'''
new2ctx = '''x_lir::Literal::String(s) => Ok(format!("\"{}\"", s)),
                x_lir::Literal::Char(c) => {
                    let escaped = match c {
                        '\\n' => "\\\\n",
                        '\\r' => "\\\\r",
                        '\\t' => "\\\\t",
                        '\\\\' => "\\\\\\\\",
                        '\\''' => "\\\\'",
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
                _ => Ok("_".to_string()),'''
c = c.replace(old2ctx, new2ctx)

# ============================================================
# Fix 2: XValue type mapping
# ============================================================
old3 = 'x_lir::Type::Named(name) => name.clone(),'
new3 = '''x_lir::Type::Named(name) => {
                if matches!(name.as_str(), "XValue" | "Option" | "Result" | "Box" | "Ref")
                    || (name.len() == 1 && name.chars().next().map_or(false, |ch| ch.is_uppercase()))
                {
                    "i32".to_string()
                } else {
                    name.clone()
                }
            }'''
c = c.replace(old3, new3)

# ============================================================
# Fix 3: Cast for float targets
# ============================================================
old4 = '''x_lir::Expression::Cast(type_, expr) => {
                let expr_str = self.emit_lir_expression(expr)?;
                let type_str = self.emit_lir_type(type_);
                Ok(format!("@as({}, {})", type_str, expr_str))
            }'''
new4 = '''x_lir::Expression::Cast(type_, expr) => {
                let expr_str = self.emit_lir_expression(expr)?;
                let type_str = self.emit_lir_type(type_);
                let is_float_target = matches!(type_str.as_str(), "f32" | "f64" | "f128");
                if is_float_target {
                    Ok(format!("@floatFromInt({})", expr_str))
                } else {
                    Ok(format!("@as({}, {})", type_str, expr_str))
                }
            }'''
c = c.replace(old4, new4)

# ============================================================
# Fix 4: __index__ builtin
# ============================================================
old5 = '"len" => format!("{}.len", args.first().map(|s| s.as_str()).unwrap_or("null")),\n            _ => {'
new5 = '''"len" => format!("{}.len", args.first().map(|s| s.as_str()).unwrap_or("null")),
            "__index__" => {
                if args.len() == 2 {
                    format!("{}[@intCast({})]", args[0], args[1])
                } else {
                    "null".to_string()
                }
            }
            _ => {'''
c = c.replace(old5, new5)

# ============================================================
# Fix 5: Division operator
# ============================================================
old6 = 'x_lir::BinaryOp::Divide => "/",'
new6 = 'x_lir::BinaryOp::Divide => "@divTrunc",'
c = c.replace(old6, new6)

old7 = 'x_lir::BinaryOp::Modulo => "%",'
new7 = 'x_lir::BinaryOp::Modulo => "@mod",'
c = c.replace(old7, new7)

# ============================================================
# Fix 6: comptime_int - temp var type tracking
# ============================================================
# Add field
old8 = 'temp_use_counts: std::collections::HashMap<String, usize>,\n    used_params: std::collections::HashSet<String>,'
new8 = '''temp_use_counts: std::collections::HashMap<String, usize>,
    temp_var_types: std::collections::HashMap<String, String>,
    used_params: std::collections::HashSet<String>,'''
c = c.replace(old8, new8)

# Init
old9 = 'temp_use_counts: std::collections::HashMap::new(),\n            used_params: std::collections::HashSet::new(),'
new9 = '''temp_use_counts: std::collections::HashMap::new(),
            temp_var_types: std::collections::HashMap::new(),
            used_params: std::collections::HashSet::new(),'''
c = c.replace(old9, new9)

# Clear
old10 = 'self.temp_use_counts.clear();\n        self.used_params.clear();'
new10 = '''self.temp_use_counts.clear();
        self.temp_var_types.clear();
        self.used_params.clear();'''
c = c.replace(old10, new10)

# Collect before function body
old11 = '''self.temp_assignment_counts = Self::collect_temp_assignment_counts(&func.body);
        self.temp_use_counts = Self::collect_temp_use_counts(&func.body);
        self.used_params'''
new11 = '''self.temp_assignment_counts = Self::collect_temp_assignment_counts(&func.body);
        self.temp_use_counts = Self::collect_temp_use_counts(&func.body);
        self.temp_var_types.clear();
        for stmt in &func.body.statements {
            if let x_lir::Statement::Variable(var) = stmt {
                if var.name.starts_with('t')
                    && var.name.len() > 1
                    && var.name[1..].chars().all(|c| c.is_ascii_digit())
                {
                    self.temp_var_types.insert(var.name.clone(), self.emit_lir_type(&var.type_));
                }
            }
        }
        self.used_params'''
c = c.replace(old11, new11)

# Use type in lazy declarations
old12 = '''if use_count == 0 {
                            self.line(&format!("_ = {};", value_part))?;
                        } else if self.declared_temp_vars.insert(var_name.clone()) {
                            let decl_keyword = if assignment_count > 1 { "var" } else { "const" };
                            self.line(&format!("{} {} = {};", decl_keyword, var_name, value_part))?;
                        } else {
                            self.line(&format!("{} = {};", var_name, value_part))?;
                        }'''
new12 = '''if use_count == 0 {
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
                        }'''
c = c.replace(old12, new12)

with open('/home/xiongdi/workspace/x-lang/compiler/x-codegen-zig/src/lib.rs', 'w') as f:
    f.write(c)
print('All 6 fixes applied successfully!')
