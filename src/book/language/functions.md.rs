//@ # Functions
//@
//@ > This section is a work-in-progress experiment about making the book executable.
//@
use crate::language::*; //#
//@ A function consists of a block (that's the body of the function), along with a name, a set of
//@ parameters, and an output type. Other than a name, all these are optional.
//@
//@ ```grammar
//@ Function ->
//@     qualifiers:FunctionQualifiers `fn` name:IDENTIFIER generics:GenericParams?
//@         `(` params:FunctionParameters? `)`
//@         ( `->` return_type:Type )? where_clause:WhereClause?
//@         body:FunctionBody
//@
//@ FunctionQualifiers -> `const`? `async`? ItemSafety? (`extern` Abi?)?
//@
//@ ItemSafety -> `safe` | `unsafe`
//@
//@ Abi -> STRING_LITERAL | RAW_STRING_LITERAL
//@
//@ FunctionParameters ->
//@     FunctionParam (`,` FunctionParam)* `,`?
//@
//@ FunctionParam -> OuterAttribute* FunctionParamKind
//@
//@ FunctionParamKind -> SelfParam | RefSelfShorthand | RegularFunctionParam
//@
//@ SelfParam -> `mut`? `self` (`:` Type)?
//@
//@ RefSelfShorthand -> `&` Lifetime? `mut`? `self`
//@
//@ RegularFunctionParam -> ( PatternNoTopAlt `:` )? FunctionParamType
//@
//@ FunctionParamType -> Type | `...`
//@
//@ FunctionBody -> BlockExpression | `;`
//@ ```
//@
//@ ```rustylr
//@ Function(Function)
//@     : qualifiers=FunctionQualifiers fn_! name=Identifier generics=GenericParams?
//@         lparen! parameters=FunctionParameters? rparen!
//@         return_type=(arrow! Type)? where_clause=WhereClause?
//@         body=FunctionBody {
//@         Function {
//@             qualifiers,
//@             name,
//@             generic_params: generics.unwrap_or_default(),
//@             parameters: parameters.unwrap_or_default(),
//@             return_type,
//@             where_clause,
//@             body,
//@         }
//@     }
//@     ;
//@ ```
//@
#[derive(Clone, Debug, PartialEq, Eq)] //#
pub struct Function {
    pub qualifiers: FunctionQualifiers,
    pub name: Identifier,
    pub generic_params: GenericParams,
    pub parameters: FunctionParameters,
    pub return_type: Option<Type>,
    pub where_clause: Option<WhereClause>,
    pub body: FunctionBody,
}

//@ ```rustylr
//@ FunctionQualifiers(FunctionQualifiers)
//@     : is_const=const_? is_async=async_? safety=ItemSafety?
//@         extern_abi=ExternAbi? {
//@         FunctionQualifiers {
//@             is_const: is_const.is_some(),
//@             is_async: is_async.is_some(),
//@             safety,
//@             extern_abi,
//@         }
//@     }
//@     ;
//@ ```
//@
#[derive(Clone, Debug, Default, PartialEq, Eq)] //#
pub struct FunctionQualifiers {
    pub is_const: bool,
    pub is_async: bool,
    pub safety: Option<ItemSafety>,
    pub extern_abi: Option<ExternAbi>,
}

//@ ```rustylr
//@ ItemSafety(ItemSafety)
//@     : safe! { ItemSafety::Safe }
//@     | unsafe_! { ItemSafety::Unsafe }
//@     ;
//@ ```
//@
#[derive(Clone, Debug, PartialEq, Eq)] //#
pub enum ItemSafety {
    Safe,
    Unsafe,
}

//@ ```rustylr
//@ ExternAbi(ExternAbi)
//@     : extern_! literal=string_literal? {
//@         ExternAbi {
//@             abi: literal.map(|literal| {
//@                 let Token::StringLiteral(literal) = literal else {
//@                     unreachable!("expected string literal token")
//@                 };
//@                 literal
//@             })
//@         }
//@     }
//@     ;
//@ ```
//@
#[derive(Clone, Debug, PartialEq, Eq)] //#
pub struct ExternAbi {
    pub abi: Option<String>,
}

//@ ```rustylr
//@ FunctionParameters(FunctionParameters)
//@     : args=$sep(FunctionParam, comma, +) comma? {
//@         FunctionParameters { args }
//@     }
//@     ;
//@ ```
//@
#[derive(Clone, Debug, PartialEq, Eq, Default)] //#
pub struct FunctionParameters {
    pub args: Vec<FunctionParam>,
}

//@ ```rustylr
//@ FunctionParam(FunctionParam)
//@     : attrs=OuterAttribute* kind=FunctionParamKind { FunctionParam { attrs, kind } }
//@     ;
//@ ```
//@
#[derive(Clone, Debug, PartialEq, Eq)] //#
pub struct FunctionParam {
    pub attrs: Vec<OuterAttribute>,
    pub kind: FunctionParamKind,
}

//@ ```rustylr
//@ FunctionParamKind(FunctionParamKind)
//@     : amp! lifetime=Lifetime? mutability=Mutability self_! {
//@         FunctionParamKind::RefSelfShorthand {
//@             lifetime,
//@             mutability,
//@         }
//@     }
//@     | mutability=Mutability self_! ty=(colon! Type)? {
//@         FunctionParamKind::SelfParam {
//@             mutability,
//@             ty,
//@         }
//@     }
//@     | pattern=(PatternNoTopAlt colon!)? ty=FunctionParamType {
//@         FunctionParamKind::Regular { pattern, ty }
//@     }
//@     ;
//@ ```
//@
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

//@ ```rustylr
//@ FunctionParamType(FunctionParamType)
//@     : ty=Type {
//@         FunctionParamType::Type(ty)
//@     }
//@     | ellipsis! {
//@         FunctionParamType::Variadic
//@     }
//@     ;
//@ ```
//@
#[derive(Clone, Debug, PartialEq, Eq)] //#
pub enum FunctionParamType {
    Type(Type),
    Variadic,
}

//@ ```rustylr
//@ FunctionBody(FunctionBody)
//@     : block=BlockExpression { FunctionBody::Block(block) }
//@     | semicolon! { FunctionBody::Missing }
//@     ;
//@ ```
//@
#[derive(Clone, Debug, PartialEq, Eq)] //#
pub enum FunctionBody {
    Block(BlockExpression),
    Missing,
}
