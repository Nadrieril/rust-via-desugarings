//@ # Final Desugarings
//@
//@ At the end of these series of steps, everything is explicit and we've reached the final
//@ language.
//@
//@ > The rest of this section is a work-in-progress experiment about making the book executable.
use crate::desugarings::*; //#

pub fn desugar_final(program: &mut Program) -> Result<(), CompilationError> {
    program.visit_all_mut_infallible(|block: &mut BlockExpression| {
        split_let_initializers(block);
    });
    Ok(())
}

//@ As a final cleanup, we split let-statements with an initializer into a declaration followed by
//@ assignment:
//@
//@ ```rust,example
//@ let x = val;
//@ // becomes
//@ let x;
//@ x = val;
//@ ```
fn split_let_initializers(block: &mut BlockExpression) {
    let statements = std::mem::take(&mut block.statements);
    for statement in statements {
        if let Statement::Let {
            attrs,
            pattern: Pattern::Identifier(name),
            ty,
            initial_value: Some(value),
            else_branch: None,
        } = statement
        {
            block.statements.push(Statement::Let {
                attrs,
                pattern: Pattern::Identifier(name.clone()),
                ty,
                initial_value: None,
                else_branch: None,
            });
            let assignment = Expression::new(ExpressionKind::Operator(Box::new(
                OperatorExpression::Assignment(Expression::new(ExpressionKind::Path(name)), value),
            )));
            block.statements.push(Statement::Expr(assignment));
        } else {
            block.statements.push(statement)
        }
    }
}
