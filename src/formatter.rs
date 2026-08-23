use crate::ast::{Expression, Program, Statement};

pub fn format_program(program: &Program) -> String {
    let mut out = String::new();
    for stmt in &program.statements {
        out.push_str(&format_statement(stmt, 0));
        out.push('\n');
    }
    out
}

fn indent(level: usize) -> String {
    "    ".repeat(level)
}

fn format_statement(stmt: &Statement, level: usize) -> String {
    match stmt {
        Statement::Let { name, value, is_mutable } => {
            let keyword = if *is_mutable { "var" } else { "let" };
            format!("{}{} {} = {}", indent(level), keyword, name, format_expression(value, level))
        }
        Statement::Assign { name, value } => {
            format!("{}{} = {}", indent(level), name, format_expression(value, level))
        }
        Statement::IndexAssign { left, index, value } => {
            format!("{}{}[{}] = {}", indent(level), format_expression(left, level), format_expression(index, level), format_expression(value, level))
        }
        Statement::FieldAssign { object, field, value } => {
            format!("{}{}.{} = {}", indent(level), format_expression(object, level), field, format_expression(value, level))
        }
        Statement::StructDef { name, fields } => {
            let mut out = format!("{}struct {} {{\n", indent(level), name);
            for (f_name, f_type) in fields {
                if let Some(t) = f_type {
                    out.push_str(&format!("{}{}: {},\n", indent(level + 1), f_name, t));
                } else {
                    out.push_str(&format!("{}{},\n", indent(level + 1), f_name));
                }
            }
            out.push_str(&format!("{}}}", indent(level)));
            out
        }
        Statement::Return(expr) => {
            format!("{}return {}", indent(level), format_expression(expr, level))
        }
        Statement::Expression(expr) => {
            format!("{}{}", indent(level), format_expression(expr, level))
        }
        Statement::Block(statements) => {
            let mut out = String::new();
            for s in statements {
                out.push_str(&format_statement(s, level));
                out.push('\n');
            }
            out
        }
        Statement::Break => {
            format!("{}break", indent(level))
        }
        Statement::Continue => {
            format!("{}continue", indent(level))
        }
    }
}

fn format_expression(expr: &Expression, level: usize) -> String {
    match expr {
        Expression::Identifier(name) => name.clone(),
        Expression::IntegerLiteral(val) => val.to_string(),
        Expression::FloatLiteral(val) => val.to_string(),
        Expression::Boolean(val) => val.to_string(),
        Expression::StringLiteral(val) => format!("\"{}\"", val),
        Expression::NullLiteral => "null".to_string(),
        Expression::Prefix { operator, right } => {
            format!("{}{}", operator, format_expression(right, level))
        }
        Expression::Infix { left, operator, right } => {
            format!("{} {} {}", format_expression(left, level), operator, format_expression(right, level))
        }
        Expression::Range { start, end, inclusive } => {
            let op = if *inclusive { "..=" } else { ".." };
            format!("{}{}{}", format_expression(start, level), op, format_expression(end, level))
        }
        Expression::Array(elements) => {
            let els: Vec<String> = elements.iter().map(|e| format_expression(e, level)).collect();
            format!("[{}]", els.join(", "))
        }
        Expression::HashLiteral(pairs) => {
            let mut els = Vec::new();
            for (k, v) in pairs {
                els.push(format!("{}: {}", format_expression(k, level), format_expression(v, level)));
            }
            format!("{{{}}}", els.join(", "))
        }
        Expression::FieldAccess { object, field } => {
            format!("{}.{}", format_expression(object, level), field)
        }
        Expression::Index { left, index } => {
            format!("{}[{}]", format_expression(left, level), format_expression(index, level))
        }
        Expression::Call { function, arguments } => {
            let args: Vec<String> = arguments.iter().map(|a| format_expression(a, level)).collect();
            format!("{}({})", format_expression(function, level), args.join(", "))
        }
        Expression::If { condition, consequence, alternative } => {
            let mut out = format!("if {} {{\n{}", format_expression(condition, level), format_statement(consequence, level + 1));
            if let Some(alt) = alternative {
                out.push_str(&format!("{}}} else {{\n{}", indent(level), format_statement(alt, level + 1)));
            }
            out.push_str(&format!("{}}}", indent(level)));
            out
        }
        Expression::While { condition, body } => {
            format!("while {} {{\n{}{}}}", format_expression(condition, level), format_statement(body, level + 1), indent(level))
        }
        Expression::For { variable, iterable, body } => {
            format!("for {} in {} {{\n{}{}}}", variable, format_expression(iterable, level), format_statement(body, level + 1), indent(level))
        }
        Expression::Match { value, cases } => {
            let mut out = format!("match {} {{\n", format_expression(value, level));
            for (pattern, consequence) in cases {
                out.push_str(&format!("{}{}", indent(level + 1), format_expression(pattern, level + 1)));
                out.push_str(&format!(" => {{\n{}{}}},\n", format_statement(consequence, level + 2), indent(level + 1)));
            }
            out.push_str(&format!("{}}}", indent(level)));
            out
        }
        Expression::TryCatch { try_body, catch_param, catch_body } => {
            let mut out = format!("try {{\n{}", format_statement(try_body, level + 1));
            out.push_str(&format!("{}}} catch {} {{\n", indent(level), catch_param));
            out.push_str(&format!("{}", format_statement(catch_body, level + 1)));
            out.push_str(&format!("{}}}", indent(level)));
            out
        }
        Expression::Throw(expr) => {
            format!("throw {}", format_expression(expr, level))
        }
        Expression::FunctionLiteral { parameters, return_type, body, .. } => {
            let mut params = Vec::new();
            for (p_name, p_type) in parameters {
                if let Some(t) = p_type {
                    params.push(format!("{}: {}", p_name, t));
                } else {
                    params.push(p_name.clone());
                }
            }
            
            let ret = if let Some(rt) = return_type {
                format!(" -> {}", rt)
            } else {
                "".to_string()
            };

            let func_decl = format!("func({}){}", params.join(", "), ret);
            format!("{} {{\n{}{}}}", func_decl, format_statement(body, level + 1), indent(level))
        }
    }
}
