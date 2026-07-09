//@ # Misc Expression Desugarings
//@
//@ > This section is a work-in-progress experiment about making the book executable.
//@
//@ This step handles some small initial desugarings of expressions.
//@
use crate::desugarings::*;

pub fn desugar_misc_exprs(program: &mut Program) -> Result<(), CompilationError> {
    program.visit_all_mut(|expression: &mut Expression| {
        flatten_grouped_expression(expression);
        Ok(())
    })
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
