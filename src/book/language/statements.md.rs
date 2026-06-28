use crate::language::*; //#
//@ # Statements
//@
//@ > This section is a work-in-progress experiment about making the book executable.
//@
//@ ```grammar
//@ Statement:
//@     | `;` => Statement::Empty,
//@     | expr=ExpressionStatement => Statement::Expr(expr),
//@     | statement=LetStatement => statement,
//@
//@ LetStatement -> Statement:
//@     attrs=OuterAttribute* `let` pattern=PatternNoTopAlt ( `:` ty=Type )?
//@     ( `=` initial_value=Expression )?
//@     ( `else` else_branch=BlockExpressionNoInnerAttributes )?
//@     `;`
//@     => Statement::Let { attrs, pattern, ty, initial_value, else_branch },
//@
//@ ExpressionStatement -> Expression:
//@     | expr=ExpressionWithoutBlock `;` => expr,
//@     | expr=ExpressionWithBlock `;`? => expr,
//@ ```
#[derive(Debug, Clone, PartialEq, Eq)] //#
pub enum Statement {
    Empty,
    Let {
        attrs: Vec<OuterAttribute>,
        pattern: Pattern,
        ty: Option<Type>,
        initial_value: Option<Expression>,
        else_branch: Option<BlockExpression>,
    },
    Expr(Expression),
}
