//@ # Grouped Expressions
//@
//@ > This section is a work-in-progress experiment about making the book executable.
//@
//@ ```grammar
//@ GroupedExpression -> Box<Expression>:
//@     `(` expression=Expression `)`
//@     => Box::new(expression)
//@ ```
