//@ # Misc Expression Desugarings
//@
//@ > This section is a work-in-progress experiment about making the book executable.
//@
//@ This step handles some small initial desugarings of expressions.
//@
use crate::desugarings::*;

pub fn misc_expr_desugarings(program: &mut Program) -> Result<(), CompilationError> {
    program.visit_all_mut(|expression: &mut Expression| {
        flatten_grouped_expression(expression);
        add_missing_else_branch(expression);
        Ok(())
    })
}

//@ ## If expressions without else
//@
//@ An omitted `else` branch evaluates to `()` [ref:expr.if.result]
fn add_missing_else_branch(expression: &mut Expression) {
    if let ExpressionKind::If(if_expression) = &mut expression.kind {
        if if_expression.else_branch.is_none() {
            if_expression.else_branch = Some(Box::new(empty_block_expression()));
        }
    }
}

fn empty_block_expression() -> Expression {
    BlockExpression {
        label: None,
        inner_attrs: vec![],
        statements: vec![],
        tail: None,
    }
    .into()
}

//@ ## Grouped expressions
//@
//@ Parenthesized expressions evaluate to the value of the enclosed operand. [ref:expr.paren.evaluation]
fn flatten_grouped_expression(expression: &mut Expression) {
    while let ExpressionKind::Grouped(inner) = &mut expression.kind {
        expression.attrs.append(&mut inner.attrs);
        expression.kind = inner.kind.clone();
    }
}
