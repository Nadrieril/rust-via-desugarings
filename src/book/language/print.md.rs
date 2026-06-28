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
        match self {
            BlockExpression::Empty => write!(f, "{{}}"),
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
