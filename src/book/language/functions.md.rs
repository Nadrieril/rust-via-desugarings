//@ # Functions
//@
//@ > This section is a work-in-progress experiment about making the book executable.
//@
use crate::language::*; //#
//@ A function consists of a block (that’s the body of the function), along with a name, a set of
//@ parameters, and an output type. Other than a name, all these are optional.
//@
//@ ```lalrpop
//@ pub Function: Function = {
//@     <qualifiers:FunctionQualifiers> "fn" <name:Identifier> <generic_params:GenericParams?>
//@         "(" <parameters:FunctionParameters?> ")"
//@         <return_type:FunctionReturnType?> <where_clause:WhereClause?>
//@         <body:FunctionBody> => Function {
//@             qualifiers,
//@             name,
//@             generic_params: generic_params.unwrap_or_default(),
//@             parameters: parameters.unwrap_or_default(),
//@             return_type,
//@             where_clause,
//@             body,
//@         },
//@ };
//@
//@ FunctionReturnType: Type = {
//@     "->" <ty:Type> => ty,
//@ };
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

//@ ```lalrpop
//@ FunctionBody: FunctionBody = {
//@     <block:BlockExpression> => FunctionBody::Block(block),
//@     ";" => FunctionBody::Missing,
//@ };
//@ ```
//@
#[derive(Clone, Debug, PartialEq, Eq)] //#
pub enum FunctionBody {
    Block(BlockExpression),
    Missing,
}

//@ ```lalrpop
//@ FunctionQualifiers: FunctionQualifiers = {
//@     <is_const:ConstQualifier?> <is_async:AsyncQualifier?> <safety:ItemSafety?>
//@         <extern_abi:ExternAbi?> => FunctionQualifiers {
//@             is_const: is_const.unwrap_or(false),
//@             is_async: is_async.unwrap_or(false),
//@             safety,
//@             extern_abi,
//@         },
//@ };
//@
//@ ConstQualifier: bool = {
//@     "const" => true,
//@ };
//@
//@ AsyncQualifier: bool = {
//@     "async" => true,
//@ };
//@ ```
//@
#[derive(Clone, Debug, Default, PartialEq, Eq)] //#
pub struct FunctionQualifiers {
    pub is_const: bool,
    pub is_async: bool,
    pub safety: Option<ItemSafety>,
    pub extern_abi: Option<ExternAbi>,
}

//@ ```lalrpop
//@ ItemSafety: ItemSafety = {
//@     "safe" => ItemSafety::Safe,
//@     "unsafe" => ItemSafety::Unsafe,
//@ };
//@ ```
//@
#[derive(Clone, Debug, PartialEq, Eq)] //#
pub enum ItemSafety {
    Safe,
    Unsafe,
}

//@ ```lalrpop
//@ ExternAbi: ExternAbi = {
//@     "extern" <abi:Abi?> => ExternAbi { abi },
//@ };
//@ ```
//@ ```lalrpop
//@ Abi: String = {
//@     <literal:STRING_LITERAL> => literal,
//@     <literal:RAW_STRING_LITERAL> => literal,
//@ };
//@ ```
#[derive(Clone, Debug, PartialEq, Eq)] //#
pub struct ExternAbi {
    pub abi: Option<String>,
}

//@ ```lalrpop
//@ FunctionParameters: FunctionParameters = {
//@     <self_param:SelfParam> => FunctionParameters {
//@         self_param: Some(self_param),
//@         params: Vec::new(),
//@     },
//@     <self_param:SelfParam> "," => FunctionParameters {
//@         self_param: Some(self_param),
//@         params: Vec::new(),
//@     },
//@     <self_param:SelfParam> "," <params:FunctionParamList> => FunctionParameters {
//@         self_param: Some(self_param),
//@         params,
//@     },
//@     <params:FunctionParamList> => FunctionParameters {
//@         self_param: None,
//@         params,
//@     },
//@ };
//@
//@ FunctionParamList: Vec<FunctionParam> = {
//@     <first:FunctionParam> "," <rest:FunctionParamList> => {
//@         [first].into_iter().chain(rest).collect()
//@     },
//@     <param:FunctionParam> "," => vec![param],
//@     <param:FunctionParam> => vec![param],
//@ };
//@ ```
//@
#[derive(Clone, Debug, Default, PartialEq, Eq)] //#
pub struct FunctionParameters {
    pub self_param: Option<SelfParam>,
    pub params: Vec<FunctionParam>,
}

//@ ```lalrpop
//@ SelfParam: SelfParam = {
//@     <attrs:OuterAttribute*> <kind:SelfParamKind> => SelfParam { attrs, kind },
//@ };
//@ ```
//@
#[derive(Clone, Debug, PartialEq, Eq)] //#
pub struct SelfParam {
    pub attrs: Vec<OuterAttribute>,
    pub kind: SelfParamKind,
}

//@ ```lalrpop
//@ SelfParamKind: SelfParamKind = {
//@     <lifetime:ReferenceReceiver> <is_mut:Mutability?> "self" =>
//@         SelfParamKind::Shorthand(ShorthandSelf {
//@             receiver: SelfReceiver::Reference { lifetime },
//@             is_mut: is_mut.unwrap_or(false),
//@         }),
//@     <is_mut:Mutability?> "self" <tail:SelfParamKindTail> => match tail {
//@         Some(ty) => SelfParamKind::Typed(TypedSelf {
//@             is_mut: is_mut.unwrap_or(false),
//@             ty,
//@         }),
//@         None => SelfParamKind::Shorthand(ShorthandSelf {
//@             receiver: SelfReceiver::Value,
//@             is_mut: is_mut.unwrap_or(false),
//@         }),
//@     },
//@ };
//@
//@ SelfParamKindTail: Option<Type> = {
//@     ":" <ty:Type> => Some(ty),
//@     => None,
//@ };
//@ ```
//@
#[derive(Clone, Debug, PartialEq, Eq)] //#
pub enum SelfParamKind {
    Shorthand(ShorthandSelf),
    Typed(TypedSelf),
}

//@ ```lalrpop
//@ ShorthandSelf: ShorthandSelf = {
//@     <receiver:ReferenceReceiver?> <is_mut:Mutability?> "self" => ShorthandSelf {
//@         receiver: receiver
//@             .map(|lifetime| SelfReceiver::Reference { lifetime })
//@             .unwrap_or(SelfReceiver::Value),
//@         is_mut: is_mut.unwrap_or(false),
//@     },
//@ };
//@ ```
//@
#[derive(Clone, Debug, PartialEq, Eq)] //#
pub struct ShorthandSelf {
    pub receiver: SelfReceiver,
    pub is_mut: bool,
}

//@ ```lalrpop
//@ ReferenceReceiver: Option<Lifetime> = {
//@     "&" <lifetime:MaybeLifetime> => lifetime,
//@ };
//@
//@ MaybeLifetime: Option<Lifetime> = {
//@     <lifetime:Lifetime> => Some(lifetime),
//@     => None,
//@ };
//@ ```
//@
#[derive(Clone, Debug, PartialEq, Eq)] //#
pub enum SelfReceiver {
    Value,
    Reference { lifetime: Option<Lifetime> },
}

//@ ```lalrpop
//@ TypedSelf: TypedSelf = {
//@     <is_mut:Mutability?> "self" ":" <ty:Type> => TypedSelf {
//@         is_mut: is_mut.unwrap_or(false),
//@         ty,
//@     },
//@ };
//@ ```
//@
#[derive(Clone, Debug, PartialEq, Eq)] //#
pub struct TypedSelf {
    pub is_mut: bool,
    pub ty: Type,
}

//@ ```lalrpop
//@ Mutability: bool = {
//@     "mut" => true,
//@ };
//@ ```
//@
pub type Mutability = bool;

//@ ```lalrpop
//@ FunctionParam: FunctionParam = {
//@     <attrs:OuterAttribute*> <kind:FunctionParamKind> => FunctionParam { attrs, kind },
//@ };
//@ ```
//@
#[derive(Clone, Debug, PartialEq, Eq)] //#
pub struct FunctionParam {
    pub attrs: Vec<OuterAttribute>,
    pub kind: FunctionParamKind,
}

//@ ```lalrpop
//@ FunctionParamKind: FunctionParamKind = {
//@     <pattern:FunctionParamPattern> => FunctionParamKind::Pattern(pattern),
//@     "..." => FunctionParamKind::Variadic,
//@     <ty:Type> => FunctionParamKind::Type(ty),
//@ };
//@ ```
//@
#[derive(Clone, Debug, PartialEq, Eq)] //#
pub enum FunctionParamKind {
    Pattern(FunctionParamPattern),
    Variadic,
    Type(Type),
}

//@ ```lalrpop
//@ FunctionParamPattern: FunctionParamPattern = {
//@     <pattern:PatternNoTopAlt> ":" <ty:FunctionParamType> =>
//@         FunctionParamPattern { pattern, ty },
//@ };
//@ ```
//@
#[derive(Clone, Debug, PartialEq, Eq)] //#
pub struct FunctionParamPattern {
    pub pattern: PatternNoTopAlt,
    pub ty: FunctionParamType,
}

//@ ```lalrpop
//@ FunctionParamType: FunctionParamType = {
//@     <ty:Type> => FunctionParamType::Type(ty),
//@     "..." => FunctionParamType::Variadic,
//@ };
//@ ```
//@
#[derive(Clone, Debug, PartialEq, Eq)] //#
pub enum FunctionParamType {
    Type(Type),
    Variadic,
}
