use crate::language::*; //#
//@ # Statements
//@
//@ > This section is a work-in-progress experiment about making the book executable.
//@
//@ ```grammar
//@ Statement:
//@     | `;` => Statement::Empty,
//@     | item=Item => Statement::Item(item),
//@     | statement=LetStatement => statement,
//@     | expr=ExpressionStatement => Statement::Expr(expr),
//@
//@ LetStatement -> Statement:
//@     attrs=OuterAttribute* `let` pattern=PatternNoTopAlt ( `:` ty=Type )?
//@     ( `=` initial_value=Expression )?
//@     ( `else` else_branch=BlockExpressionNoInnerAttributes )?
//@     `;`
//@     => Statement::Let { attrs, scope: None, pattern, ty, initial_value, else_branch },
//@
//@ ExpressionStatement -> Expression:
//@     | expr=ExpressionWithoutBlock `;` => expr,
//@     | expr=ExpressionWithBlock `;`? => expr,
//@ ```
#[derive(Debug, Clone, PartialEq, Eq)] //#
#[derive(Drive, DriveMut)] //#
pub enum Statement {
    Empty,
    Item(Item),
    Let {
        attrs: Vec<OuterAttribute>,
        /// A block label, to make a `let` defined in another scope than the current one. This is a
        /// made-up feature to make desugarings easier, see "Scoped Let" for a description.
        scope: Option<Identifier>,
        /// The "binding" part of a `let` can be an arbitrary pattern.
        pattern: Pattern,
        /// Optional type annotation.
        ty: Option<Type>,
        /// Optional initial value.
        initial_value: Option<Expression>,
        /// Optional `else` branch, executed if the pattern fails to match the initial value
        /// provided.
        else_branch: Option<BlockExpression>,
    },
    Expr(Expression),
}
