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
//@         ( `->` return_type=Type )? where_clause=WhereClause?
//@         body=FunctionBody
//@     => Function {
//@         qualifiers,
//@         name,
//@         generic_params: generic_params.unwrap_or_default(),
//@         parameters: parameters.unwrap_or_default(),
//@         return_type,
//@         where_clause,
//@         body,
//@     }
//@ ```
//@
#[derive(Clone, Debug, PartialEq, Eq)] //#
pub struct Function {
    pub qualifiers: FunctionQualifiers,
    pub name: Identifier,
    pub generic_params: GenericParams,
    pub parameters: Vec<FunctionParam>,
    pub return_type: Option<Type>,
    pub where_clause: Option<WhereClause>,
    pub body: FunctionBody,
}

//@ ```grammar
//@ FunctionQualifiers:
//@     is_const=`const`? is_async=`async`? ItemSafety? ExternAbi?
//@     => FunctionQualifiers {
//@         is_const: is_const.is_some(),
//@         is_async: is_async.is_some(),
//@         safety: ItemSafety,
//@         extern_abi: ExternAbi,
//@     }
//@ ```
#[derive(Clone, Debug, Default, PartialEq, Eq)] //#
pub struct FunctionQualifiers {
    pub is_const: bool,
    pub is_async: bool,
    pub safety: Option<ItemSafety>,
    pub extern_abi: Option<ExternAbi>,
}

//@ ```grammar
//@ ItemSafety:
//@     | `safe` => ItemSafety::Safe,
//@     | `unsafe` => ItemSafety::Unsafe,
//@ ```
#[derive(Clone, Debug, PartialEq, Eq)] //#
pub enum ItemSafety {
    Safe,
    Unsafe,
}

//@ ```grammar
//@ ExternAbi: `extern` Abi?
//@     => ExternAbi { abi: Abi }
//@ ```
#[derive(Clone, Debug, PartialEq, Eq)] //#
pub struct ExternAbi {
    pub abi: Option<String>,
}

//@ ```grammar
//@ FunctionParameters:
//@     first_arg=FunctionParam args=(`,` FunctionParam)* `,`?
//@     => [first_arg].into_iter().chain(args).collect()
//@
//@ FunctionParam: attrs=OuterAttribute* kind=FunctionParamKind
//@     => FunctionParam { attrs, kind }
//@ ```
#[derive(Clone, Debug, PartialEq, Eq)] //#
pub struct FunctionParam {
    pub attrs: Vec<OuterAttribute>,
    pub kind: FunctionParamKind,
}

pub type FunctionParameters = Vec<FunctionParam>;

//@ ```grammar
//@ FunctionParamKind:
//@     | `&` Lifetime? Mutability `self` => FunctionParamKind::RefSelfShorthand { lifetime: Lifetime, mutability: Mutability },
//@     | Mutability `self` (`:` Type)? => FunctionParamKind::SelfParam { mutability: Mutability, ty: Type },
//@     | pattern=( PatternNoTopAlt `:` )? ty=FunctionParamType => FunctionParamKind::Regular { pattern, ty }
//@ ```
#[derive(Clone, Debug, PartialEq, Eq)] //#
pub enum FunctionParamKind {
    Regular {
        pattern: Option<PatternNoTopAlt>,
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

//@ ```grammar
//@ FunctionParamType:
//@     | Type => FunctionParamType::Type(Type),
//@     | `...` => FunctionParamType::Variadic,
//@ ```
#[derive(Clone, Debug, PartialEq, Eq)] //#
pub enum FunctionParamType {
    Type(Type),
    Variadic,
}

//@ ```grammar
//@ FunctionBody:
//@     | BlockExpression => FunctionBody::Block(BlockExpression)
//@     | `;` => FunctionBody::Missing
//@ ```
#[derive(Clone, Debug, PartialEq, Eq)] //#
pub enum FunctionBody {
    Block(BlockExpression),
    Missing,
}
