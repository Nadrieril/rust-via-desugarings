//@ # Printing
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
            self.name, self.generic_params, self.parameters
        )?;
        if let Some(return_type) = &self.return_type {
            write!(f, " -> {return_type}")?;
        }
        if let Some(where_clause) = &self.where_clause {
            write!(f, " {where_clause}")?;
        }
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

impl Display for ExternAbi {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match &self.abi {
            Some(abi) => write!(f, "extern \"{abi}\""),
            None => f.write_str("extern"),
        }
    }
}

impl Display for FunctionParameters {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if let Some(self_param) = &self.self_param {
            write!(f, "{self_param},")?
        }
        write!(f, "{}", self.params.iter().format(", "))
    }
}

impl Display for SelfParam {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} ", self.attrs.iter().format(" "))?;
        write!(f, "{}", self.kind)
    }
}

impl Display for SelfParamKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            SelfParamKind::Shorthand(shorthand) => Display::fmt(shorthand, f),
            SelfParamKind::Typed(typed) => Display::fmt(typed, f),
        }
    }
}

impl Display for ShorthandSelf {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} ", self.receiver)?;
        if self.is_mut {
            f.write_str("mut ")?;
        }
        f.write_str("self")
    }
}

impl Display for SelfReceiver {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            SelfReceiver::Value => {}
            SelfReceiver::Reference { lifetime } => {
                f.write_str("&")?;
                if let Some(lifetime) = lifetime {
                    write!(f, "{lifetime}")?
                }
            }
        }
        Ok(())
    }
}

impl Display for TypedSelf {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if self.is_mut {
            f.write_str("mut ")?;
        }
        write!(f, "self: {}", self.ty)
    }
}

impl Display for FunctionParam {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} ", self.attrs.iter().format(" "))?;
        write!(f, "{}", self.kind)
    }
}

impl Display for FunctionParamKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            FunctionParamKind::Pattern(pattern) => write!(f, "{pattern}"),
            FunctionParamKind::Variadic => f.write_str("..."),
            FunctionParamKind::Type(ty) => write!(f, "{ty}"),
        }
    }
}

impl Display for FunctionParamPattern {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.pattern, self.ty)
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
            Type::Bool => write!(f, "bool"),
        }
    }
}

impl Display for BlockExpression {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            BlockExpression::BoolLiteral(value) => write!(f, "{{ {} }}", value),
        }
    }
}

impl Display for GenericParams {
    fn fmt(&self, _f: &mut Formatter<'_>) -> fmt::Result {
        Ok(())
    }
}

impl Display for WhereClause {
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

impl Display for PatternNoTopAlt {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str("_")
    }
}
