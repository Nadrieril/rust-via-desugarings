//@ # Printing
//@
//@ > This section is a work-in-progress experiment about making the book executable.
//@
//@ These printers turn the AST back into Rust syntax.
use crate::language::*; //#
use itertools::Itertools; //#
use std::fmt::{self, Display, Formatter}; //#
//@
pub fn print_program(program: &Program) -> String {
    program.to_string() + "\n"
}

impl Display for Program {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.functions.iter().format("\n\n"))
    }
}

impl Display for Function {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} ", self.qualifiers)?;
        write!(
            f,
            "fn {}{}({})",
            self.name,
            self.generic_params,
            self.parameters.iter().format(", ")
        )?;

        if let Some(return_type) = &self.return_type {
            write!(f, " -> {return_type}")?;
        }
        write!(f, " {}", self.where_clauses)?;
        match &self.body {
            FunctionBody::Block(block) => write!(f, " {block}"),
            FunctionBody::Missing => f.write_str(";"),
        }
    }
}

impl Display for FunctionQualifiers {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if self.is_const {
            f.write_str("const")?;
        }
        if self.is_async {
            f.write_str(" async")?;
        }
        if let Some(safety) = &self.safety {
            write!(f, " {safety}")?;
        }
        if let Some(extern_abi) = &self.extern_abi {
            write!(f, " {extern_abi}")?;
        }
        Ok(())
    }
}

impl Display for ItemSafety {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            ItemSafety::Safe => "safe",
            ItemSafety::Unsafe => "unsafe",
        })
    }
}

impl Display for Mutability {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Mutable => "mut ",
            Self::Immutable => "",
        })
    }
}

impl Display for ExternAbi {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match &self.abi {
            Some(abi) => write!(f, "extern \"{abi}\""),
            None => f.write_str("extern"),
        }
    }
}

impl Display for FunctionParam {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if !self.attrs.is_empty() {
            write!(f, "{} ", self.attrs.iter().format(" "))?;
        }
        write!(f, "{}", self.kind)
    }
}

impl Display for FunctionParamKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            FunctionParamKind::Regular { pattern, ty } => {
                if let Some(pattern) = pattern {
                    write!(f, "{pattern}: ")?;
                }
                write!(f, "{ty}")
            }
            FunctionParamKind::RefSelfShorthand {
                lifetime,
                mutability,
            } => {
                write!(f, "&")?;
                if let Some(lifetime) = lifetime {
                    write!(f, "{lifetime} ")?;
                }
                write!(f, "{mutability}")?;
                f.write_str("self")
            }
            FunctionParamKind::SelfParam { mutability, ty } => {
                write!(f, "{mutability}")?;
                f.write_str("self")?;
                if let Some(ty) = ty {
                    write!(f, ": {ty}")?;
                }
                Ok(())
            }
        }
    }
}

impl Display for FunctionParamType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            FunctionParamType::Type(ty) => write!(f, "{ty}"),
            FunctionParamType::Variadic => f.write_str("..."),
        }
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Type::Unit => write!(f, "()"),
            Type::Bool => write!(f, "bool"),
            Type::Str => write!(f, "str"),
            Type::TraitSelf => write!(f, "Self"),
            Type::Ref(lifetime, mutability, ty) => {
                f.write_str("&")?;
                if let Some(lifetime) = lifetime {
                    write!(f, "{lifetime} ")?;
                }
                write!(f, "{mutability}")?;
                write!(f, "{ty}")
            }
        }
    }
}

impl Display for BlockExpression {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str("{")?;
        for statement in &self.statements {
            write!(f, " {statement}")?;
        }
        if let Some(tail) = &self.tail {
            write!(f, " {tail}")?;
        }
        f.write_str(" }")
    }
}

impl Display for Statement {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Statement::Empty => f.write_str(";"),
            Statement::Let {
                attrs,
                pattern,
                ty,
                initial_value,
                else_branch,
            } => {
                write!(f, "{} ", attrs.iter().format(" "))?;
                write!(f, "let {pattern}")?;
                if let Some(ty) = ty {
                    write!(f, ": {ty}")?;
                }
                if let Some(initial_value) = initial_value {
                    write!(f, " = {initial_value}")?;
                }
                if let Some(else_branch) = else_branch {
                    write!(f, " else {else_branch}")?;
                }
                f.write_str(";")
            }
            Statement::Expr(expression) => write!(f, "{expression};"),
        }
    }
}

impl Display for Expression {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} ", self.attrs.iter().format(" "))?;
        write!(f, "{}", self.kind)
    }
}

impl Display for ExpressionKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ExpressionKind::Literal(literal) => write!(f, "{literal}"),
            ExpressionKind::Path(path) => write!(f, "{path}"),
            ExpressionKind::Operator(operator) => write!(f, "{operator}"),
            ExpressionKind::Block(block) => write!(f, "{block}"),
            ExpressionKind::Tuple(tuple) => write!(f, "{tuple}"),
            ExpressionKind::Call(call) => write!(f, "{call}"),
        }
    }
}

impl Display for LiteralExpression {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            LiteralExpression::String(value) => write!(f, "\"{value}\""),
            LiteralExpression::Integer(value) => write!(f, "{value}"),
            LiteralExpression::Bool(value) => write!(f, "{value}"),
        }
    }
}

impl Display for TupleExpression {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            TupleExpression::Unit => f.write_str("()"),
        }
    }
}

impl Display for CallExpression {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}({})", self.callee, self.args.iter().format(", "))
    }
}

impl Display for OperatorExpression {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            OperatorExpression::Add(left, right) => write!(f, "{left} + {right}"),
            OperatorExpression::Assignment(left, right) => write!(f, "{left} = {right}"),
        }
    }
}

impl Display for GenericParams {
    fn fmt(&self, _f: &mut Formatter<'_>) -> fmt::Result {
        Ok(())
    }
}

impl Display for WhereClauses {
    fn fmt(&self, _f: &mut Formatter<'_>) -> fmt::Result {
        Ok(())
    }
}

impl Display for OuterAttribute {
    fn fmt(&self, _f: &mut Formatter<'_>) -> fmt::Result {
        Ok(())
    }
}

impl Display for Lifetime {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str("'_")
    }
}

impl Display for Pattern {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Pattern::Identifier(name) => f.write_str(name),
            Pattern::Wildcard => f.write_str("_"),
        }
    }
}
