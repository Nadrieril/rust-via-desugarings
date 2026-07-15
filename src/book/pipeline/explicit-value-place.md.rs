//@ # Place-to-Value and Value-to-Place Coercions
//@
//@ Expressions can be categorized into two kinds: "value expressions" denote values, while
//@ "place expressions" denote memory locations [ref:expr.place-value].
use crate::CompilationError; //#
use crate::interactive_example; //#
use crate::language::*; //#
enum ExprCategory {
    Value,
    Place,
}

impl Expression {
    fn categorize(&self) -> ExprCategory {
        match &self.kind {
            //@ These are all the place expressions [ref:expr.place-value.place-expr-kinds]:
            ExpressionKind::Path(_) => ExprCategory::Place,
            ExpressionKind::TupleIndexing(..) => ExprCategory::Place,
            ExpressionKind::Operator(OperatorExpression::Dereference(_)) => ExprCategory::Place,
            ExpressionKind::Virtual(VirtualExpression::ValueToPlaceCoercion(_)) => {
                ExprCategory::Place
            }
            //@ Parentheses don't change the category of an expression:
            ExpressionKind::Grouped(expr) => expr.categorize(),
            //@ Anything else is a value expression [ref:expr.place-value.value-expr-kinds]:
            ExpressionKind::Operator(
                OperatorExpression::Borrow(..)
                | OperatorExpression::Add(..)
                | OperatorExpression::Assignment(..),
            )
            | ExpressionKind::Literal(..)
            | ExpressionKind::Block(..)
            | ExpressionKind::If(..)
            | ExpressionKind::Tuple(..)
            | ExpressionKind::Call(..)
            | ExpressionKind::Virtual(VirtualExpression::PlaceToValueCoercion(_)) => {
                ExprCategory::Value
            }
        }
    }
}

//@ Every subexpression location is also either a "place context", which expects a place
//@ expression, or a "value context", which expects a value expression
//@ [ref:expr.place-value.place-context]. When that expectation is unmet, Rust can convert between
//@ the two by introducing a value-to-place coercion or a place-to-value coercion, respectively.
//@
interactive_example! {
    make_place_coercions_explicit,
    fn main() {
        let x = 1 + &2;
        print(x);
    }
}
//@
//@ In [Virtual Expressions](../language/expressions/virtual-exprs.md.rs), we added expression
//@ kinds that represent these two coercions. They will be desugared further in subsequent passes.
pub fn make_place_coercions_explicit(program: &mut Program) -> Result<(), CompilationError> {
    // Add place/value coercions to all the subexpressions of each expression.
    program.visit_all_mut_infallible(|expression: &mut Expression| match &mut expression.kind {
        ExpressionKind::Literal(_) | ExpressionKind::Path(_) => {}
        ExpressionKind::Operator(operator) => match &mut **operator {
            OperatorExpression::Borrow(borrow) => expect_place(&mut borrow.expression),
            OperatorExpression::Dereference(dereference) => {
                expect_place(&mut dereference.expression)
            }
            OperatorExpression::Add(left, right) => {
                expect_value(left);
                expect_value(right);
            }
            OperatorExpression::Assignment(left, right) => {
                expect_place(left);
                expect_value(right);
            }
        },
        ExpressionKind::Grouped(_) => {}
        ExpressionKind::Block(block) => {
            if let Some(tail) = &mut block.tail {
                expect_value(tail);
            }
        }
        ExpressionKind::If(if_expression) => {
            expect_value(&mut if_expression.condition);
            expect_value(&mut if_expression.then_branch);
            if let Some(else_branch) = &mut if_expression.else_branch {
                expect_value(else_branch);
            }
        }
        ExpressionKind::Tuple(elements) => {
            for element in elements {
                expect_value(element);
            }
        }
        ExpressionKind::TupleIndexing(tuple_indexing) => {
            expect_place(&mut tuple_indexing.expression);
        }
        ExpressionKind::Call(call) => {
            for argument in &mut call.args {
                expect_value(argument);
            }
        }
        ExpressionKind::Virtual(virtual_expression) => match virtual_expression {
            VirtualExpression::ValueToPlaceCoercion(expression) => expect_value(expression),
            VirtualExpression::PlaceToValueCoercion(expression) => expect_place(expression),
        },
    });

    program.visit_all_mut_infallible(|statement: &mut Statement| match statement {
        // In theory `let` is a form of pattern matching so the scrutinee should be a place
        // expression. However `let` is also the primitive with which we make place expressions,
        // so we treat `let x = expr;` specially.
        Statement::Let {
            initial_value: Some(expr),
            pattern,
            ..
        } => match pattern {
            Pattern::Identifier(_) => expect_value(expr),
            _ => expect_place(expr),
        },
        Statement::Let { .. } => {}
        Statement::Empty | Statement::Item(_) | Statement::Expr(_) => {}
    });

    // TODO: catch the other places where an expression is mentioned: Function.body, Const.body, etc?

    Ok(())
}

fn expect(expression: &mut Expression, cat: ExprCategory) {
    match (expression.categorize(), cat) {
        (ExprCategory::Value, ExprCategory::Value) | (ExprCategory::Place, ExprCategory::Place) => {
        }
        (ExprCategory::Place, ExprCategory::Value) => {
            *expression = Expression::new(ExpressionKind::Virtual(
                VirtualExpression::PlaceToValueCoercion(Box::new(expression.clone())),
            ));
        }
        (ExprCategory::Value, ExprCategory::Place) => {
            *expression = Expression::new(ExpressionKind::Virtual(
                VirtualExpression::ValueToPlaceCoercion(Box::new(expression.clone())),
            ));
        }
    }
}
/// Turn the expression into a value expression, if needed.
fn expect_value(expression: &mut Expression) {
    expect(expression, ExprCategory::Value);
}
/// Turn the expression into a place expression, if needed.
fn expect_place(expression: &mut Expression) {
    expect(expression, ExprCategory::Place);
}
