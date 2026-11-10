use crate::ast::*;
use crate::lexer::Lexer;
use crate::parser::Parser;

pub struct Codegen {
    output: String,
    indent: usize,
}

impl Codegen {
    pub fn new() -> Self {
        Self {
            output: String::new(),
            indent: 0,
        }
    }

    pub fn generate(mut self, program: &Program) -> String {
        self.emit_prelude(program);
        for item in &program.items {
            self.emit_item(item);
            self.output.push('\n');
        }
        self.output
    }

    fn emit_prelude(&mut self, program: &Program) {
        self.writeln("#![allow(unused_variables, unused_mut, dead_code, unused_imports, unused_parens)]");
        self.writeln("use std::io::Write;");

        // Collect user-specified use paths to avoid duplicates
        let user_imports: Vec<String> = program
            .items
            .iter()
            .filter_map(|item| {
                if let Item::Use(u) = item {
                    Some(u.path.join("::"))
                } else {
                    None
                }
            })
            .collect();

        let code_str = format!("{:?}", program.items);
        if code_str.contains("HashMap") && !user_imports.iter().any(|p| p.contains("HashMap")) {
            self.writeln("use std::collections::HashMap;");
        }
        if code_str.contains("HashSet") && !user_imports.iter().any(|p| p.contains("HashSet")) {
            self.writeln("use std::collections::HashSet;");
        }

        self.writeln("");
    }

    fn emit_item(&mut self, item: &Item) {
        match item {
            Item::Function(f) => self.emit_function(f),
            Item::Struct(s) => self.emit_struct(s),
            Item::Impl(i) => self.emit_impl(i),
            Item::Enum(e) => self.emit_enum(e),
            Item::Trait(t) => self.emit_trait(t),
            Item::Use(u) => self.emit_use(u),
            Item::Const(c) => self.emit_const(c),
            Item::TypeAlias(t) => self.emit_type_alias(t),
        }
    }

    // --- Functions ---

    fn emit_function(&mut self, f: &Function) {
        let generics = self.generic_params_to_rust(&f.generic_params);
        let params: Vec<String> = f
            .params
            .iter()
            .map(|p| {
                if p.is_self {
                    if p.is_mut_self {
                        "&mut self".into()
                    } else {
                        "&self".into()
                    }
                } else {
                    let ty = self.type_to_rust(&p.ty);
                    format!("{}: {ty}", p.name)
                }
            })
            .collect();

        let ret = match &f.return_type {
            Type::Void => String::new(),
            ty => format!(" -> {}", self.type_to_rust(ty)),
        };

        self.writeln(&format!(
            "fn {}{generics}({}){ret} {{",
            f.name,
            params.join(", ")
        ));
        self.indent += 1;
        self.emit_body(&f.body);
        self.indent -= 1;
        self.writeln("}");
    }

    // --- Structs ---

    fn emit_struct(&mut self, s: &Struct) {
        let generics = self.generic_params_to_rust(&s.generic_params);
        self.writeln("#[derive(Debug, Clone)]");
        self.writeln(&format!("struct {}{generics} {{", s.name));
        self.indent += 1;
        for field in &s.fields {
            let ty = self.type_to_rust(&field.ty);
            self.writeln(&format!("{}: {ty},", field.name));
        }
        self.indent -= 1;
        self.writeln("}");
    }

    // --- Impl ---

    fn emit_impl(&mut self, i: &ImplBlock) {
        let generics = self.generic_params_to_rust(&i.generic_params);
        let type_args = if i.type_args.is_empty() {
            String::new()
        } else {
            let args: Vec<String> = i.type_args.iter().map(|t| self.type_to_rust(t)).collect();
            format!("<{}>", args.join(", "))
        };

        if let Some(trait_name) = &i.trait_name {
            self.writeln(&format!(
                "impl{generics} {trait_name} for {}{type_args} {{",
                i.type_name
            ));
        } else {
            self.writeln(&format!(
                "impl{generics} {}{type_args} {{",
                i.type_name
            ));
        }
        self.indent += 1;
        for method in &i.methods {
            self.emit_function(method);
            self.output.push('\n');
        }
        self.indent -= 1;
        self.writeln("}");
    }

    // --- Enum ---

    fn emit_enum(&mut self, e: &Enum) {
        let generics = self.generic_params_to_rust(&e.generic_params);
        self.writeln("#[derive(Debug, Clone)]");
        self.writeln(&format!("enum {}{generics} {{", e.name));
        self.indent += 1;
        for variant in &e.variants {
            if variant.fields.is_empty() {
                self.writeln(&format!("{},", variant.name));
            } else {
                let types: Vec<String> =
                    variant.fields.iter().map(|t| self.type_to_rust(t)).collect();
                self.writeln(&format!("{}({}),", variant.name, types.join(", ")));
            }
        }
        self.indent -= 1;
        self.writeln("}");
    }

    // --- Trait ---

    fn emit_trait(&mut self, t: &Trait) {
        let generics = self.generic_params_to_rust(&t.generic_params);
        self.writeln(&format!("trait {}{generics} {{", t.name));
        self.indent += 1;
        for method in &t.methods {
            let m_generics = self.generic_params_to_rust(&method.generic_params);
            let params: Vec<String> = method
                .params
                .iter()
                .map(|p| {
                    if p.is_self {
                        if p.is_mut_self {
                            "&mut self".into()
                        } else {
                            "&self".into()
                        }
                    } else {
                        let ty = self.type_to_rust(&p.ty);
                        format!("{}: {ty}", p.name)
                    }
                })
                .collect();

            let ret = match &method.return_type {
                Type::Void => String::new(),
                ty => format!(" -> {}", self.type_to_rust(ty)),
            };

            if let Some(body) = &method.body {
                self.writeln(&format!(
                    "fn {}{m_generics}({}){ret} {{",
                    method.name,
                    params.join(", ")
                ));
                self.indent += 1;
                self.emit_body(body);
                self.indent -= 1;
                self.writeln("}");
            } else {
                self.writeln(&format!(
                    "fn {}{m_generics}({}){ret};",
                    method.name,
                    params.join(", ")
                ));
            }
            self.output.push('\n');
        }
        self.indent -= 1;
        self.writeln("}");
    }

    // --- Use ---

    fn emit_use(&mut self, u: &UseDecl) {
        let path = u.path.join("::");
        if u.glob {
            self.writeln(&format!("use {path}::*;"));
        } else if let Some(group) = &u.group {
            self.writeln(&format!("use {path}::{{{}}};", group.join(", ")));
        } else if let Some(alias) = &u.alias {
            self.writeln(&format!("use {path} as {alias};"));
        } else {
            self.writeln(&format!("use {path};"));
        }
    }

    // --- Const ---

    fn emit_const(&mut self, c: &Const) {
        let val = self.expr_to_rust(&c.value);
        let ty = match &c.ty {
            Type::Inferred => self.infer_const_type(&c.value),
            ty => self.type_to_rust(ty),
        };
        // String consts use &str
        let (ty, val) = if ty == "String" {
            if let Expr::StringLit(s) = &c.value {
                let escaped = Self::escape_string_for_rust(s);
                ("&str".to_string(), format!("\"{escaped}\""))
            } else {
                (ty, val)
            }
        } else {
            (ty, val)
        };
        self.writeln(&format!("const {}: {ty} = {val};", c.name));
    }

    fn infer_const_type(&self, expr: &Expr) -> String {
        match expr {
            Expr::IntLit(_) => "i64".into(),
            Expr::FloatLit(_) => "f64".into(),
            Expr::StringLit(_) => "String".into(),
            Expr::BoolLit(_) => "bool".into(),
            Expr::CharLit(_) => "char".into(),
            Expr::UnaryOp { op: UnaryOp::Neg, expr } => self.infer_const_type(expr),
            _ => "_".into(),
        }
    }

    // --- Type alias ---

    fn emit_type_alias(&mut self, t: &TypeAlias) {
        let generics = self.generic_params_to_rust(&t.generic_params);
        let target = self.type_to_rust(&t.target);
        self.writeln(&format!("type {}{generics} = {target};", t.name));
    }

    // --- Generics ---

    fn generic_params_to_rust(&self, params: &[GenericParam]) -> String {
        if params.is_empty() {
            return String::new();
        }
        let parts: Vec<String> = params
            .iter()
            .map(|p| {
                if p.bounds.is_empty() {
                    p.name.clone()
                } else {
                    format!("{}: {}", p.name, p.bounds.join(" + "))
                }
            })
            .collect();
        format!("<{}>", parts.join(", "))
    }

    // --- Statements ---

    fn emit_body(&mut self, stmts: &[Statement]) {
        for stmt in stmts {
            self.emit_statement(stmt);
        }
    }

    fn emit_statement(&mut self, stmt: &Statement) {
        match stmt {
            Statement::Var { name, ty, value } => {
                let val = self.expr_to_rust(value);
                let val = Self::strip_outer_parens(&val);
                match ty {
                    Type::Inferred => self.writeln(&format!("let {name} = {val};")),
                    ty => {
                        let ty = self.type_to_rust(ty);
                        self.writeln(&format!("let {name}: {ty} = {val};"));
                    }
                }
            }
            Statement::Mut { name, ty, value } => {
                let val = self.expr_to_rust(value);
                let val = Self::strip_outer_parens(&val);
                match ty {
                    Type::Inferred => self.writeln(&format!("let mut {name} = {val};")),
                    ty => {
                        let ty = self.type_to_rust(ty);
                        self.writeln(&format!("let mut {name}: {ty} = {val};"));
                    }
                }
            }
            Statement::Assign { target, value } => {
                let t = self.expr_to_rust(target);
                let v = self.expr_to_rust(value);
                let v = Self::strip_outer_parens(&v);
                self.writeln(&format!("{t} = {v};"));
            }
            Statement::CompoundAssign { target, op, value } => {
                let t = self.expr_to_rust(target);
                let v = self.expr_to_rust(value);
                let v = Self::strip_outer_parens(&v);
                let op_str = match op {
                    BinOp::Add => "+=",
                    BinOp::Sub => "-=",
                    BinOp::Mul => "*=",
                    BinOp::Div => "/=",
                    _ => unreachable!(),
                };
                self.writeln(&format!("{t} {op_str} {v};"));
            }
            Statement::Expr(expr) => {
                let e = self.expr_to_rust(expr);
                self.writeln(&format!("{e};"));
            }
            Statement::Return(None) => {
                self.writeln("return;");
            }
            Statement::Return(Some(expr)) => {
                let e = self.expr_to_rust(expr);
                let e = Self::strip_outer_parens(&e);
                self.writeln(&format!("return {e};"));
            }
            Statement::If {
                condition,
                body,
                else_if_branches,
                else_body,
            } => {
                let cond = self.expr_to_rust(condition);
                let cond = Self::strip_outer_parens(&cond);
                self.writeln(&format!("if {cond} {{"));
                self.indent += 1;
                self.emit_body(body);
                self.indent -= 1;
                for (cond, b) in else_if_branches {
                    let c = self.expr_to_rust(cond);
                    let c = Self::strip_outer_parens(&c);
                    self.writeln(&format!("}} else if {c} {{"));
                    self.indent += 1;
                    self.emit_body(b);
                    self.indent -= 1;
                }
                if let Some(eb) = else_body {
                    self.writeln("} else {");
                    self.indent += 1;
                    self.emit_body(eb);
                    self.indent -= 1;
                }
                self.writeln("}");
            }
            Statement::While { condition, body } => {
                let cond = self.expr_to_rust(condition);
                let cond = Self::strip_outer_parens(&cond);
                self.writeln(&format!("while {cond} {{"));
                self.indent += 1;
                self.emit_body(body);
                self.indent -= 1;
                self.writeln("}");
            }
            Statement::For { var, iter, body } => {
                let it = self.expr_to_rust(iter);
                self.writeln(&format!("for {var} in {it} {{"));
                self.indent += 1;
                self.emit_body(body);
                self.indent -= 1;
                self.writeln("}");
            }
            Statement::Match { value, arms } => {
                let val = self.expr_to_rust(value);
                self.writeln(&format!("match {val} {{"));
                self.indent += 1;
                for arm in arms {
                    let pat = self.pattern_to_rust(&arm.pattern);
                    if arm.body.len() == 1 {
                        if let Statement::Expr(expr) = &arm.body[0] {
                            let e = self.expr_to_rust(expr);
                            self.writeln(&format!("{pat} => {{ {e}; }}"));
                            continue;
                        }
                    }
                    self.writeln(&format!("{pat} => {{"));
                    self.indent += 1;
                    self.emit_body(&arm.body);
                    self.indent -= 1;
                    self.writeln("}");
                }
                self.indent -= 1;
                self.writeln("}");
            }
            Statement::Break => self.writeln("break;"),
            Statement::Continue => self.writeln("continue;"),
        }
    }

    // --- Expressions ---

    fn expr_to_rust(&self, expr: &Expr) -> String {
        match expr {
            Expr::IntLit(n) => format!("{n}_i64"),
            Expr::FloatLit(n) => {
                let s = format!("{n}");
                if s.contains('.') {
                    format!("{s}_f64")
                } else {
                    format!("{s}.0_f64")
                }
            }
            Expr::StringLit(s) => self.string_interpolation(s),
            Expr::BoolLit(b) => format!("{b}"),
            Expr::CharLit(c) => {
                let escaped = match c {
                    '\n' => "\\n".to_string(),
                    '\t' => "\\t".to_string(),
                    '\r' => "\\r".to_string(),
                    '\\' => "\\\\".to_string(),
                    '\'' => "\\'".to_string(),
                    '\0' => "\\0".to_string(),
                    c => c.to_string(),
                };
                format!("'{escaped}'")
            }
            Expr::Ident(name) => self.translate_builtin_ident(name),
            Expr::BinaryOp { left, op, right } => {
                let l = self.expr_to_rust(left);
                let r = self.expr_to_rust(right);
                match op {
                    BinOp::Power => format!("({l} as f64).powf({r} as f64)"),
                    _ => {
                        let op_str = match op {
                            BinOp::Add => "+",
                            BinOp::Sub => "-",
                            BinOp::Mul => "*",
                            BinOp::Div => "/",
                            BinOp::Mod => "%",
                            BinOp::Eq => "==",
                            BinOp::Neq => "!=",
                            BinOp::Lt => "<",
                            BinOp::Gt => ">",
                            BinOp::Lte => "<=",
                            BinOp::Gte => ">=",
                            BinOp::And => "&&",
                            BinOp::Or => "||",
                            BinOp::Power => unreachable!(),
                        };
                        format!("({l} {op_str} {r})")
                    }
                }
            }
            Expr::UnaryOp { op, expr } => {
                let e = self.expr_to_rust(expr);
                match op {
                    UnaryOp::Neg => format!("(-{e})"),
                    UnaryOp::Not => format!("(!{e})"),
                }
            }
            Expr::Call { func, args } => self.emit_call(func, args),
            Expr::MethodCall { object, method, args } => {
                let obj = self.expr_to_rust(object);
                let a: Vec<String> = args.iter().map(|a| self.expr_to_rust(a)).collect();
                // Type::method() → Type::method() (associated function)
                if let Expr::Ident(name) = object.as_ref() {
                    if name.chars().next().is_some_and(|c| c.is_uppercase()) {
                        return format!("{obj}::{method}({})", a.join(", "));
                    }
                }
                format!("{obj}.{method}({})", a.join(", "))
            }
            Expr::FieldAccess { object, field } => {
                let obj = self.expr_to_rust(object);
                format!("{obj}.{field}")
            }
            Expr::Index { object, index } => {
                let obj = self.expr_to_rust(object);
                let idx = self.expr_to_rust(index);
                // Only cast to usize for integer expressions (not strings, etc.)
                if Self::is_integer_expr(index) {
                    format!("{obj}[{idx} as usize]")
                } else {
                    format!("{obj}[&{idx}]")
                }
            }
            Expr::StructInit { name, fields } => {
                let fs: Vec<String> = fields
                    .iter()
                    .map(|(k, v)| {
                        if let Expr::Ident(id) = v {
                            if id == k {
                                return k.clone();
                            }
                        }
                        let val = self.expr_to_rust(v);
                        format!("{k}: {val}")
                    })
                    .collect();
                format!("{name} {{ {} }}", fs.join(", "))
            }
            Expr::ArrayLit(elems) => {
                let es: Vec<String> = elems.iter().map(|e| self.expr_to_rust(e)).collect();
                format!("vec![{}]", es.join(", "))
            }
            Expr::Range { start, end, inclusive } => {
                let e = self.expr_to_rust(end);
                if let Some(s) = start {
                    let s = self.expr_to_rust(s);
                    if *inclusive {
                        format!("{s}..={e}")
                    } else {
                        format!("{s}..{e}")
                    }
                } else if *inclusive {
                    format!("0..={e}")
                } else {
                    format!("0..{e}")
                }
            }
            Expr::Lambda { params, body } => {
                let b = self.expr_to_rust(body);
                if params.is_empty() {
                    format!("|| {b}")
                } else {
                    format!("|{}| {b}", params.join(", "))
                }
            }
            Expr::Cast { expr, ty } => {
                let e = self.expr_to_rust(expr);
                let t = self.type_to_rust(ty);
                format!("({e} as {t})")
            }
        }
    }

    // --- Builtin identifiers ---

    fn translate_builtin_ident(&self, name: &str) -> String {
        match name {
            "PI" => "std::f64::consts::PI".into(),
            "E" => "std::f64::consts::E".into(),
            "TAU" => "std::f64::consts::TAU".into(),
            "INFINITY" => "f64::INFINITY".into(),
            "NEG_INFINITY" => "f64::NEG_INFINITY".into(),
            "NAN" => "f64::NAN".into(),
            "MAX_INT" => "i64::MAX".into(),
            "MIN_INT" => "i64::MIN".into(),
            _ => name.into(),
        }
    }

    // --- Builtin function calls ---

    fn emit_call(&self, func: &Expr, args: &[Expr]) -> String {
        if let Expr::Ident(name) = func {
            match name.as_str() {
                "print" => return self.emit_print_call(args),
                "println" => return self.emit_println_call(args),
                "input" => return self.emit_input_call(args),
                "range" => return self.emit_range(args),
                "len" => return self.emit_builtin_method(args, "len()"),
                "push" => return self.emit_builtin_2arg(args, "push"),
                "pop" => return self.emit_builtin_method(args, "pop().unwrap()"),
                "str" => return self.emit_to_string(args),
                "int" => return self.emit_parse_int(args),
                "float" => return self.emit_parse_float(args),

                // Math
                "sqrt" => return self.emit_float_method_1(args, "sqrt"),
                "abs" => return self.emit_abs(args),
                "floor" => return self.emit_float_method_1(args, "floor"),
                "ceil" => return self.emit_float_method_1(args, "ceil"),
                "round" => return self.emit_float_method_1(args, "round"),
                "sin" => return self.emit_float_method_1(args, "sin"),
                "cos" => return self.emit_float_method_1(args, "cos"),
                "tan" => return self.emit_float_method_1(args, "tan"),
                "log" => return self.emit_float_method_1(args, "ln"),
                "log2" => return self.emit_float_method_1(args, "log2"),
                "log10" => return self.emit_float_method_1(args, "log10"),
                "pow" => return self.emit_pow(args),
                "min" => return self.emit_min_max(args, "min"),
                "max" => return self.emit_min_max(args, "max"),
                "clamp" => return self.emit_clamp(args),

                // String operations
                "split" => return self.emit_split(args),
                "join" => return self.emit_join(args),
                "trim" => return self.emit_str_method_0(args, "trim"),
                "to_upper" => return self.emit_str_method_0(args, "to_uppercase"),
                "to_lower" => return self.emit_str_method_0(args, "to_lowercase"),
                "contains" => return self.emit_str_method_1(args, "contains"),
                "starts_with" => return self.emit_str_method_1(args, "starts_with"),
                "ends_with" => return self.emit_str_method_1(args, "ends_with"),
                "replace" => return self.emit_replace(args),
                "chars" => return self.emit_chars(args),

                // Collections
                "sort" => return self.emit_sort(args),
                "reverse" => return self.emit_reverse(args),
                "map" => return self.emit_map(args),
                "filter" => return self.emit_filter(args),
                "enumerate" => return self.emit_enumerate(args),
                "zip" => return self.emit_zip(args),

                // IO
                "read_file" => return self.emit_read_file(args),
                "write_file" => return self.emit_write_file(args),
                "file_exists" => return self.emit_file_exists(args),
                "append_file" => return self.emit_append_file(args),

                // Misc
                "exit" => {
                    if args.len() == 1 {
                        let a = self.expr_to_rust(&args[0]);
                        return format!("std::process::exit({a} as i32)");
                    }
                    return "std::process::exit(0)".into();
                }
                "type_of" => {
                    if args.len() == 1 {
                        let a = self.expr_to_rust(&args[0]);
                        return format!("std::any::type_name_of_val(&{a})");
                    }
                }
                "assert" => {
                    if args.len() == 1 {
                        let a = self.expr_to_rust(&args[0]);
                        return format!("assert!({a})");
                    } else if args.len() == 2 {
                        let a = self.expr_to_rust(&args[0]);
                        let msg = self.expr_to_rust(&args[1]);
                        return format!("assert!({a}, \"{{}}\", {msg})");
                    }
                }
                _ => {}
            }
        }
        let f = self.expr_to_rust(func);
        let a: Vec<String> = args.iter().map(|a| self.expr_to_rust(a)).collect();
        format!("{f}({})", a.join(", "))
    }

    // --- Builtin helpers ---

    fn emit_range(&self, args: &[Expr]) -> String {
        match args.len() {
            1 => {
                let end = self.expr_to_rust(&args[0]);
                format!("0..{end}")
            }
            2 => {
                let start = self.expr_to_rust(&args[0]);
                let end = self.expr_to_rust(&args[1]);
                format!("{start}..{end}")
            }
            3 => {
                let start = self.expr_to_rust(&args[0]);
                let end = self.expr_to_rust(&args[1]);
                let step = self.expr_to_rust(&args[2]);
                format!("({start}..{end}).step_by({step} as usize)")
            }
            _ => "0..0".into(),
        }
    }

    fn emit_to_string(&self, args: &[Expr]) -> String {
        if args.len() == 1 {
            let a = self.expr_to_rust(&args[0]);
            format!("{a}.to_string()")
        } else {
            "String::new()".into()
        }
    }

    fn emit_parse_int(&self, args: &[Expr]) -> String {
        if args.len() == 1 {
            let a = self.expr_to_rust(&args[0]);
            format!("{a}.trim().parse::<i64>().unwrap()")
        } else {
            "0_i64".into()
        }
    }

    fn emit_parse_float(&self, args: &[Expr]) -> String {
        if args.len() == 1 {
            let a = self.expr_to_rust(&args[0]);
            format!("{a}.trim().parse::<f64>().unwrap()")
        } else {
            "0.0_f64".into()
        }
    }

    fn emit_builtin_method(&self, args: &[Expr], method: &str) -> String {
        if args.len() == 1 {
            let a = self.expr_to_rust(&args[0]);
            format!("{a}.{method}")
        } else {
            format!("/* len: expected 1 arg */ 0")
        }
    }

    fn emit_builtin_2arg(&self, args: &[Expr], method: &str) -> String {
        if args.len() == 2 {
            let a = self.expr_to_rust(&args[0]);
            let b = self.expr_to_rust(&args[1]);
            format!("{a}.{method}({b})")
        } else {
            format!("/* {method}: expected 2 args */")
        }
    }

    fn emit_float_method_1(&self, args: &[Expr], method: &str) -> String {
        if args.len() == 1 {
            let a = self.expr_to_rust(&args[0]);
            format!("({a} as f64).{method}()")
        } else {
            format!("/* {method}: expected 1 arg */ 0.0")
        }
    }

    fn emit_abs(&self, args: &[Expr]) -> String {
        if args.len() == 1 {
            let a = self.expr_to_rust(&args[0]);
            format!("{a}.abs()")
        } else {
            "0".into()
        }
    }

    fn emit_pow(&self, args: &[Expr]) -> String {
        if args.len() == 2 {
            let base = self.expr_to_rust(&args[0]);
            let exp = self.expr_to_rust(&args[1]);
            format!("({base} as f64).powf({exp} as f64)")
        } else {
            "0.0".into()
        }
    }

    fn emit_min_max(&self, args: &[Expr], which: &str) -> String {
        if args.len() == 2 {
            let a = self.expr_to_rust(&args[0]);
            let b = self.expr_to_rust(&args[1]);
            format!("{{ let (__a, __b) = ({a}, {b}); if __a {op} __b {{ __a }} else {{ __b }} }}",
                op = if which == "min" { "<" } else { ">" })
        } else {
            "0".into()
        }
    }

    fn emit_clamp(&self, args: &[Expr]) -> String {
        if args.len() == 3 {
            let val = self.expr_to_rust(&args[0]);
            let lo = self.expr_to_rust(&args[1]);
            let hi = self.expr_to_rust(&args[2]);
            format!("{{ let __v = {val}; let __lo = {lo}; let __hi = {hi}; if __v < __lo {{ __lo }} else if __v > __hi {{ __hi }} else {{ __v }} }}")
        } else {
            "0".into()
        }
    }

    fn emit_split(&self, args: &[Expr]) -> String {
        if args.len() == 2 {
            let s = self.expr_to_rust(&args[0]);
            let delim = self.expr_to_rust(&args[1]);
            format!("{s}.split(&*{delim}).map(|s| s.to_string()).collect::<Vec<_>>()")
        } else {
            "vec![]".into()
        }
    }

    fn emit_join(&self, args: &[Expr]) -> String {
        if args.len() == 2 {
            let items = self.expr_to_rust(&args[0]);
            let delim = self.expr_to_rust(&args[1]);
            format!("{items}.join(&*{delim})")
        } else {
            "String::new()".into()
        }
    }

    fn emit_str_method_0(&self, args: &[Expr], method: &str) -> String {
        if args.len() == 1 {
            let s = self.expr_to_rust(&args[0]);
            format!("{s}.{method}().to_string()")
        } else {
            "String::new()".into()
        }
    }

    fn emit_str_method_1(&self, args: &[Expr], method: &str) -> String {
        if args.len() == 2 {
            let s = self.expr_to_rust(&args[0]);
            let arg = self.expr_to_rust(&args[1]);
            format!("{s}.{method}(&*{arg})")
        } else {
            "false".into()
        }
    }

    fn emit_replace(&self, args: &[Expr]) -> String {
        if args.len() == 3 {
            let s = self.expr_to_rust(&args[0]);
            let from = self.expr_to_rust(&args[1]);
            let to = self.expr_to_rust(&args[2]);
            format!("{s}.replace(&*{from}, &*{to})")
        } else {
            "String::new()".into()
        }
    }

    fn emit_chars(&self, args: &[Expr]) -> String {
        if args.len() == 1 {
            let s = self.expr_to_rust(&args[0]);
            format!("{s}.chars().collect::<Vec<_>>()")
        } else {
            "vec![]".into()
        }
    }

    fn emit_sort(&self, args: &[Expr]) -> String {
        if args.len() == 1 {
            let a = self.expr_to_rust(&args[0]);
            format!("{{ let mut __v = {a}; __v.sort(); __v }}")
        } else {
            "vec![]".into()
        }
    }

    fn emit_reverse(&self, args: &[Expr]) -> String {
        if args.len() == 1 {
            let a = self.expr_to_rust(&args[0]);
            format!("{{ let mut __v = {a}; __v.reverse(); __v }}")
        } else {
            "vec![]".into()
        }
    }

    fn emit_map(&self, args: &[Expr]) -> String {
        if args.len() == 2 {
            let collection = self.expr_to_rust(&args[0]);
            let func = self.expr_to_rust(&args[1]);
            format!("{collection}.into_iter().map({func}).collect::<Vec<_>>()")
        } else {
            "vec![]".into()
        }
    }

    fn emit_filter(&self, args: &[Expr]) -> String {
        if args.len() == 2 {
            let collection = self.expr_to_rust(&args[0]);
            let func = self.expr_to_rust(&args[1]);
            format!("{collection}.into_iter().filter({func}).collect::<Vec<_>>()")
        } else {
            "vec![]".into()
        }
    }

    fn emit_enumerate(&self, args: &[Expr]) -> String {
        if args.len() == 1 {
            let collection = self.expr_to_rust(&args[0]);
            format!("{collection}.into_iter().enumerate().collect::<Vec<_>>()")
        } else {
            "vec![]".into()
        }
    }

    fn emit_zip(&self, args: &[Expr]) -> String {
        if args.len() == 2 {
            let a = self.expr_to_rust(&args[0]);
            let b = self.expr_to_rust(&args[1]);
            format!("{a}.into_iter().zip({b}.into_iter()).collect::<Vec<_>>()")
        } else {
            "vec![]".into()
        }
    }

    fn emit_read_file(&self, args: &[Expr]) -> String {
        if args.len() == 1 {
            let path = self.expr_to_rust(&args[0]);
            format!("std::fs::read_to_string(&*{path}).unwrap()")
        } else {
            "String::new()".into()
        }
    }

    fn emit_write_file(&self, args: &[Expr]) -> String {
        if args.len() == 2 {
            let path = self.expr_to_rust(&args[0]);
            let content = self.expr_to_rust(&args[1]);
            format!("std::fs::write(&*{path}, &*{content}).unwrap()")
        } else {
            "()".into()
        }
    }

    fn emit_file_exists(&self, args: &[Expr]) -> String {
        if args.len() == 1 {
            let path = self.expr_to_rust(&args[0]);
            format!("std::path::Path::new(&*{path}).exists()")
        } else {
            "false".into()
        }
    }

    fn emit_append_file(&self, args: &[Expr]) -> String {
        if args.len() == 2 {
            let path = self.expr_to_rust(&args[0]);
            let content = self.expr_to_rust(&args[1]);
            format!("{{ use std::io::Write; let mut __f = std::fs::OpenOptions::new().append(true).create(true).open(&*{path}).unwrap(); write!(__f, \"{{}}\", {content}).unwrap(); }}")
        } else {
            "()".into()
        }
    }

    // --- Print / println / input ---

    fn emit_print_call(&self, args: &[Expr]) -> String {
        if args.is_empty() {
            return "print!(\"\")".into();
        }
        if let Expr::StringLit(s) = &args[0] {
            let (parts_fmt, fmt_args) = self.parse_format_string(s);
            if fmt_args.is_empty() {
                format!("print!(\"{parts_fmt}\")")
            } else {
                format!("print!(\"{parts_fmt}\", {})", fmt_args.join(", "))
            }
        } else {
            let a: Vec<String> = args.iter().map(|a| self.expr_to_rust(a)).collect();
            format!("print!(\"{{}}\", {})", a.join(", "))
        }
    }

    fn emit_println_call(&self, args: &[Expr]) -> String {
        if args.is_empty() {
            return "println!()".into();
        }
        if let Expr::StringLit(s) = &args[0] {
            let (parts_fmt, fmt_args) = self.parse_format_string(s);
            if fmt_args.is_empty() {
                format!("println!(\"{parts_fmt}\")")
            } else {
                format!("println!(\"{parts_fmt}\", {})", fmt_args.join(", "))
            }
        } else {
            let a: Vec<String> = args.iter().map(|a| self.expr_to_rust(a)).collect();
            format!("println!(\"{{}}\", {})", a.join(", "))
        }
    }

    fn emit_input_call(&self, args: &[Expr]) -> String {
        let prompt = if !args.is_empty() {
            if let Expr::StringLit(s) = &args[0] {
                let escaped = Self::escape_string_for_rust(s);
                format!("print!(\"{escaped}\"); std::io::stdout().flush().unwrap(); ")
            } else {
                let a = self.expr_to_rust(&args[0]);
                format!("print!(\"{{}}\", {a}); std::io::stdout().flush().unwrap(); ")
            }
        } else {
            String::new()
        };
        format!("{{ {prompt}let mut __input = String::new(); std::io::stdin().read_line(&mut __input).unwrap(); __input.trim().to_string() }}")
    }

    // --- String interpolation ---

    fn translate_interpolation_arg(&self, arg_str: &str) -> String {
        let mut lexer = Lexer::new(arg_str);
        match lexer.tokenize() {
            Ok(tokens) => {
                let mut parser = Parser::new(tokens);
                match parser.parse_expr() {
                    Ok(expr) => self.expr_to_rust(&expr),
                    Err(_) => arg_str.to_string(),
                }
            }
            Err(_) => arg_str.to_string(),
        }
    }

    fn parse_format_string(&self, s: &str) -> (String, Vec<String>) {
        let mut parts_fmt = String::new();
        let mut fmt_args = Vec::new();
        let mut chars = s.chars().peekable();
        while let Some(ch) = chars.next() {
            if ch == '{' {
                let mut expr = String::new();
                let mut depth = 0;
                for c in chars.by_ref() {
                    if c == '{' {
                        depth += 1;
                    }
                    if c == '}' {
                        if depth == 0 {
                            break;
                        }
                        depth -= 1;
                    }
                    expr.push(c);
                }
                if expr.is_empty() {
                    parts_fmt.push_str("{{}}");
                } else {
                    parts_fmt.push_str("{}");
                    fmt_args.push(self.translate_interpolation_arg(&expr));
                }
            } else if ch == '}' {
                parts_fmt.push_str("}}");
            } else {
                parts_fmt.push_str(&Self::escape_for_format(ch));
            }
        }
        (parts_fmt, fmt_args)
    }

    fn string_interpolation(&self, s: &str) -> String {
        let (parts_fmt, args) = self.parse_format_string(s);
        if args.is_empty() {
            format!("\"{parts_fmt}\".to_string()")
        } else {
            format!("format!(\"{parts_fmt}\", {})", args.join(", "))
        }
    }

    // --- Utility ---

    fn is_integer_expr(expr: &Expr) -> bool {
        match expr {
            Expr::IntLit(_) => true,
            Expr::Ident(_) => true, // assume integer for simple idents (common case)
            Expr::BinaryOp { left, op: _, right } => {
                Self::is_integer_expr(left) && Self::is_integer_expr(right)
            }
            Expr::UnaryOp { op: UnaryOp::Neg, expr } => Self::is_integer_expr(expr),
            Expr::Call { .. } => true, // assume integer
            // String-like things
            Expr::StringLit(_) => false,
            Expr::MethodCall { .. } => false, // e.g., alice.clone() → not integer
            Expr::Cast { .. } => false,
            _ => true,
        }
    }

    fn strip_outer_parens(s: &str) -> &str {
        if s.starts_with('(') && s.ends_with(')') {
            let inner = &s[1..s.len() - 1];
            let mut depth = 0;
            for ch in inner.chars() {
                match ch {
                    '(' => depth += 1,
                    ')' => {
                        depth -= 1;
                        if depth < 0 {
                            return s;
                        }
                    }
                    _ => {}
                }
            }
            if depth == 0 {
                inner
            } else {
                s
            }
        } else {
            s
        }
    }

    fn escape_for_format(ch: char) -> String {
        match ch {
            '\n' => "\\n".into(),
            '\t' => "\\t".into(),
            '\r' => "\\r".into(),
            '\\' => "\\\\".into(),
            '"' => "\\\"".into(),
            c => c.to_string(),
        }
    }

    fn escape_string_for_rust(s: &str) -> String {
        let mut out = String::new();
        for ch in s.chars() {
            out.push_str(&Self::escape_for_format(ch));
        }
        out
    }

    fn pattern_to_rust(&self, pattern: &Pattern) -> String {
        match pattern {
            Pattern::Literal(expr) => self.expr_to_rust(expr),
            Pattern::Wildcard => "_".into(),
            Pattern::Ident(name) => name.clone(),
            Pattern::EnumVariant { name, fields } => {
                if fields.is_empty() {
                    name.clone()
                } else {
                    format!("{name}({})", fields.join(", "))
                }
            }
        }
    }

    fn type_to_rust(&self, ty: &Type) -> String {
        match ty {
            Type::Int => "i64".into(),
            Type::Float => "f64".into(),
            Type::String => "String".into(),
            Type::Bool => "bool".into(),
            Type::Char => "char".into(),
            Type::Named(name) => name.clone(),
            Type::Parameterized(name, args) => {
                let args: Vec<String> = args.iter().map(|t| self.type_to_rust(t)).collect();
                format!("{name}<{}>", args.join(", "))
            }
            Type::Void => "()".into(),
            Type::Inferred => "_".into(),
        }
    }

    fn writeln(&mut self, s: &str) {
        for _ in 0..self.indent {
            self.output.push_str("    ");
        }
        self.output.push_str(s);
        self.output.push('\n');
    }
}
