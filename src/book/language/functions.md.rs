//@ # Functions
//@
//@ > This section is a work-in-progress experiment about making the book executable.
//@
use crate::language::*; //#
//@ A function consists of a block (that's the body of the function), along with a name, a set of
//@ parameters, and an output type. Other than a name, all these are optional.
//@
//@ ```grammar
//@ Function:
//@     qualifiers=FunctionQualifiers `fn` name=IDENTIFIER generic_params=GenericParams?
//@         `(` parameters=FunctionParameters? `)`
//@         ( `->` return_type=Type )? where_clauses=WhereClauses?
//@         body=FunctionBody
//@     => Function {
//@         qualifiers,
//@         name,
//@         generic_params: generic_params.unwrap_or_default(),
//@         parameters: parameters.unwrap_or_default(),
//@         return_type,
//@         where_clauses: where_clauses.unwrap_or_default(),
//@         body,
//@     }
//@
//@ FunctionBody:
//@     | BlockExpression => FunctionBody::Block(BlockExpression)
//@     | `;` => FunctionBody::Missing
//@ ```
//@
#[derive(Debug, Clone, PartialEq, Eq)] //#
pub struct Function {
    pub qualifiers: FunctionQualifiers,
    /// The name of the function.
    pub name: Identifier,
    /// Generic parameters, for polymorphic functions.
    pub generic_params: GenericParams,
    /// The arguments that the function takes.
    pub parameters: Vec<FunctionParam>,
    /// The type returned by the function.
    pub return_type: Option<Type>,
    /// Additional predicates that must hold when calling the function.
    pub where_clauses: WhereClauses,
    /// The function body.
    pub body: FunctionBody,
}

#[derive(Debug, Clone, PartialEq, Eq)] //#
pub enum FunctionBody {
    /// The body of a function is a block.
    Block(BlockExpression),
    /// Method declarations and `extern` definitions need not have a body.
    Missing,
}

//@ ```grammar
//@ FunctionQualifiers:
//@     is_const=`const`? is_async=`async`? safety=ItemSafety? extern_abi=ExternAbi?
//@     => FunctionQualifiers {
//@         is_const: is_const.is_some(),
//@         is_async: is_async.is_some(),
//@         safety,
//@         extern_abi,
//@     }
//@
//@ ItemSafety:
//@     | `safe` => ItemSafety::Safe,
//@     | `unsafe` => ItemSafety::Unsafe,
//@
//@ ExternAbi: `extern` abi=Abi?
//@     => ExternAbi { abi }
//@ ```
#[derive(Debug, Default, Clone, PartialEq, Eq)] //#
pub struct FunctionQualifiers {
    pub is_const: bool,
    pub is_async: bool,
    pub safety: Option<ItemSafety>,
    pub extern_abi: Option<ExternAbi>,
}

#[derive(Debug, Clone, PartialEq, Eq)] //#
pub enum ItemSafety {
    Safe,
    Unsafe,
}

#[derive(Debug, Clone, PartialEq, Eq)] //#
pub struct ExternAbi {
    pub abi: Option<String>,
}

//@ ```grammar
//@ FunctionParameters -> Vec<FunctionParam>:
//@     first_arg=FunctionParam remaining_args=(`,` FunctionParam)* `,`?
//@     => [first_arg].into_iter().chain(remaining_args).collect()
//@
//@ FunctionParam: attrs=OuterAttribute* kind=FunctionParamKind
//@     => FunctionParam { attrs, kind }
//@
//@ FunctionParamKind:
//@     | `&` lifetime=Lifetime? mutability=Mutability `self` => FunctionParamKind::RefSelfShorthand { lifetime, mutability },
//@     | mutability=Mutability `self` ty=(`:` Type)? => FunctionParamKind::SelfParam { mutability, ty },
//@     | pattern=( PatternNoTopAlt `:` )? ty=FunctionParamType => FunctionParamKind::Regular { pattern, ty }
//@
//@ FunctionParamType:
//@     | Type => FunctionParamType::Type(Type),
//@     | `...` => FunctionParamType::Variadic,
//@ ```

#[derive(Debug, Clone, PartialEq, Eq)] //#
pub struct FunctionParam {
    pub attrs: Vec<OuterAttribute>,
    pub kind: FunctionParamKind,
}

#[derive(Debug, Clone, PartialEq, Eq)] //#
pub enum FunctionParamKind {
    Regular {
        pattern: Option<Pattern>,
        ty: FunctionParamType,
    },
    SelfParam {
        mutability: Mutability,
        ty: Option<Type>,
    },
    RefSelfShorthand {
        lifetime: Option<Lifetime>,
        mutability: Mutability,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)] //#
pub enum FunctionParamType {
    Type(Type),
    Variadic,
}
