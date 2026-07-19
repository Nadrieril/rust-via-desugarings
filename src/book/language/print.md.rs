//@ # Printing
//@
//@ > This section is a work-in-progress experiment about making the book executable.
//@
//@ These printers turn the AST back into Rust syntax.
//@
//@ AI disclaimer: this section is LLM-generated.
use crate::language::*; //#
use itertools::Itertools; //#
use std::fmt::{self, Display, Formatter}; //#
//@
pub fn print_program(program: &Program) -> String {
    let mut printer = Printer::new();
    printer.program(program);
    printer.finish()
}

fn write_tuple<T: Display>(f: &mut Formatter<'_>, elements: &[T]) -> fmt::Result {
    f.write_str("(")?;
    write!(f, "{}", elements.iter().format(", "))?;
    if elements.len() == 1 {
        f.write_str(",")?;
    }
    f.write_str(")")
}

struct Printer {
    output: String,
    indent: usize,
    at_line_start: bool,
}

impl Printer {
    fn new() -> Self {
        Self {
            output: String::new(),
            indent: 0,
            at_line_start: true,
        }
    }

    fn finish(mut self) -> String {
        self.newline();
        self.output
    }

    fn token(&mut self, token: impl AsRef<str>) {
        if self.at_line_start {
            for _ in 0..self.indent {
                self.output.push_str("    ");
            }
            self.at_line_start = false;
        }
        self.output.push_str(token.as_ref());
    }

    fn display(&mut self, value: impl Display) {
        self.token(value.to_string());
    }

    fn space(&mut self) {
        self.output.push(' ');
    }

    fn newline(&mut self) {
        if !self.output.ends_with('\n') {
            self.output.push('\n');
        }
        self.at_line_start = true;
    }

    fn indented(&mut self, f: impl FnOnce(&mut Self)) {
        self.indent += 1;
        f(self);
        self.indent -= 1;
    }

    fn comma_separated<T>(&mut self, elements: &[T], mut print: impl FnMut(&mut Self, &T)) {
        for (index, element) in elements.iter().enumerate() {
            if index > 0 {
                self.token(", ");
            }
            print(self, element);
        }
    }

    fn program(&mut self, program: &Program) {
        for (index, item) in program.items.iter().enumerate() {
            if index > 0 {
                self.newline();
                self.newline();
            }
            self.item(item);
        }
    }

    fn item(&mut self, item: &Item) {
        self.attrs(&item.attrs);
        if let Some(visibility) = &item.visibility {
            self.display(visibility);
            self.space();
        }
        match &item.kind {
            ItemKind::Function(function) => self.function(function),
        }
    }

    fn attrs(&mut self, attrs: &[OuterAttribute]) {
        for attr in attrs {
            self.display(attr);
            self.space();
        }
    }

    fn function(&mut self, function: &Function) {
        self.function_qualifiers(&function.qualifiers);
        self.token("fn ");
        self.token(&function.name);
        self.display(&function.generic_params);
        self.token("(");
        self.comma_separated(&function.parameters, |printer, parameter| {
            printer.display(parameter);
        });
        self.token(")");
        if let Some(return_type) = &function.return_type {
            self.token(" -> ");
            self.display(return_type);
        }
        self.display(&function.where_clauses);
        match &function.body {
            FunctionBody::Block(block) => {
                self.space();
                self.block(block);
            }
            FunctionBody::Missing => self.token(";"),
        }
    }

    fn function_qualifiers(&mut self, qualifiers: &FunctionQualifiers) {
        let mut parts = Vec::new();
        if qualifiers.is_const {
            parts.push("const".to_owned());
        }
        if qualifiers.is_async {
            parts.push("async".to_owned());
        }
        if let Some(safety) = &qualifiers.safety {
            parts.push(safety.to_string());
        }
        if let Some(extern_abi) = &qualifiers.extern_abi {
            parts.push(extern_abi.to_string());
        }
        if !parts.is_empty() {
            self.display(parts.iter().format(" "));
            self.space();
        }
    }

    fn block(&mut self, block: &BlockExpression) {
        if let Some(label) = &block.label {
            self.display(label);
            self.token(": ");
        }
        if block.statements.is_empty() && block.tail.is_none() {
            self.token("{}");
            return;
        }

        self.token("{");
        self.indented(|printer| {
            for statement in &block.statements {
                printer.newline();
                printer.statement(statement);
            }
            if let Some(tail) = &block.tail {
                printer.newline();
                printer.expression(tail);
            }
        });
        self.newline();
        self.token("}");
    }

    fn statement(&mut self, statement: &Statement) {
        match statement {
            Statement::Empty => self.token(";"),
            Statement::Item(item) => self.item(item),
            Statement::Let {
                attrs,
                pattern,
                ty,
                initial_value,
                else_branch,
            } => {
                self.attrs(attrs);
                self.token("let ");
                self.display(pattern);
                if let Some(ty) = ty {
                    self.token(": ");
                    self.display(ty);
                }
                if let Some(initial_value) = initial_value {
                    self.token(" = ");
                    self.expression(initial_value);
                }
                if let Some(else_branch) = else_branch {
                    self.token(" else ");
                    self.block(else_branch);
                }
                self.token(";");
            }
            Statement::Expr(expression) => {
                self.expression(expression);
                if !expression.is_with_block() {
                    self.token(";");
                }
            }
        }
    }

    fn expression(&mut self, expression: &Expression) {
        self.attrs(&expression.attrs);
        match &expression.kind {
            ExpressionKind::Literal(literal) => self.display(literal),
            ExpressionKind::Path(path) => self.token(path),
            ExpressionKind::Operator(operator) => self.operator_expression(operator),
            ExpressionKind::Virtual(virtual_expression) => {
                self.virtual_expression(virtual_expression)
            }
            ExpressionKind::Grouped(grouped) => {
                self.token("(");
                self.expression(grouped);
                self.token(")");
            }
            ExpressionKind::Block(block) => self.block(block),
            ExpressionKind::If(if_expression) => self.if_expression(if_expression),
            ExpressionKind::Tuple(elements) => self.tuple(elements),
            ExpressionKind::TupleIndexing(tuple_indexing) => {
                self.expression(&tuple_indexing.expression);
                self.token(".");
                self.token(tuple_indexing.index.to_string());
            }
            ExpressionKind::Call(call) => {
                self.expression(&call.callee);
                self.token("(");
                self.comma_separated(&call.args, |printer, argument| {
                    printer.expression(argument);
                });
                self.token(")");
            }
        }
    }

    fn tuple(&mut self, elements: &[Expression]) {
        self.token("(");
        self.comma_separated(elements, |printer, element| {
            printer.expression(element);
        });
        if elements.len() == 1 {
            self.token(",");
        }
        self.token(")");
    }

    fn if_expression(&mut self, if_expression: &IfExpression) {
        self.token("if ");
        self.expression(&if_expression.condition);
        self.space();
        self.expression(&if_expression.then_branch);
        if let Some(else_branch) = &if_expression.else_branch {
            match &else_branch.kind {
                ExpressionKind::If(nested) if else_branch.attrs.is_empty() => {
                    self.token(" else ");
                    self.if_expression(nested);
                }
                ExpressionKind::Block(block) if else_branch.attrs.is_empty() => {
                    self.token(" else ");
                    self.block(block);
                }
                _ => {
                    self.token(" else ");
                    self.expression(else_branch);
                }
            }
        }
    }

    fn operator_expression(&mut self, operator: &OperatorExpression) {
        match operator {
            OperatorExpression::Borrow(borrow) => {
                self.token("&");
                self.display(borrow.mutability);
                self.expression(&borrow.expression);
            }
            OperatorExpression::Dereference(dereference) => {
                self.token("*");
                self.expression(&dereference.expression);
            }
            OperatorExpression::Add(left, right) => {
                self.expression(left);
                self.token(" + ");
                self.expression(right);
            }
            OperatorExpression::Assignment(left, right) => {
                self.expression(left);
                self.token(" = ");
                self.expression(right);
            }
        }
    }

    fn virtual_expression(&mut self, virtual_expression: &VirtualExpression) {
        match virtual_expression {
            VirtualExpression::ValueToPlaceCoercion(expression) => {
                self.token("value_to_place!(");
                self.expression(expression);
                self.token(")");
            }
            VirtualExpression::PlaceToValueCoercion(expression) => {
                self.token("place_to_value!(");
                self.expression(expression);
                self.token(")");
            }
        }
    }
}

impl Display for Program {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.items.iter().format("\n\n"))
    }
}

impl Display for Function {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} ", self.qualifiers)?;
        write!(
            f,
            "fn {}{}({})",
            self.name,
            self.generic_params,
            self.parameters.iter().format(", ")
        )?;

        if let Some(return_type) = &self.return_type {
            write!(f, " -> {return_type}")?;
        }
        write!(f, " {}", self.where_clauses)?;
        match &self.body {
            FunctionBody::Block(block) => write!(f, " {block}"),
            FunctionBody::Missing => f.write_str(";"),
        }
    }
}

impl Display for Item {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if !self.attrs.is_empty() {
            write!(f, "{} ", self.attrs.iter().format(" "))?;
        }
        if let Some(visibility) = &self.visibility {
            write!(f, "{visibility} ")?;
        }
        write!(f, "{}", self.kind)
    }
}

impl Display for ItemKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ItemKind::Function(function) => write!(f, "{function}"),
        }
    }
}

impl Display for FunctionQualifiers {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if self.is_const {
            f.write_str("const")?;
        }
        if self.is_async {
            f.write_str(" async")?;
        }
        if let Some(safety) = &self.safety {
            write!(f, " {safety}")?;
        }
        if let Some(extern_abi) = &self.extern_abi {
            write!(f, " {extern_abi}")?;
        }
        Ok(())
    }
}

impl Display for ItemSafety {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            ItemSafety::Safe => "safe",
            ItemSafety::Unsafe => "unsafe",
        })
    }
}

impl Display for Visibility {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Visibility::Pub => f.write_str("pub"),
            Visibility::PubCrate => f.write_str("pub(crate)"),
            Visibility::PubSelf => f.write_str("pub(self)"),
            Visibility::PubSuper => f.write_str("pub(super)"),
            Visibility::InPath(path) => write!(f, "pub(in {path})"),
        }
    }
}

impl Display for Mutability {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Mutable => "mut ",
            Self::Immutable => "",
        })
    }
}

impl Display for ExternAbi {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match &self.abi {
            Some(abi) => write!(f, "extern \"{abi}\""),
            None => f.write_str("extern"),
        }
    }
}

impl Display for FunctionParam {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if !self.attrs.is_empty() {
            write!(f, "{} ", self.attrs.iter().format(" "))?;
        }
        write!(f, "{}", self.kind)
    }
}

impl Display for FunctionParamKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            FunctionParamKind::Regular { pattern, ty } => {
                if let Some(pattern) = pattern {
                    write!(f, "{pattern}: ")?;
                }
                write!(f, "{ty}")
            }
            FunctionParamKind::RefSelfShorthand {
                lifetime,
                mutability,
            } => {
                write!(f, "&")?;
                if let Some(lifetime) = lifetime {
                    write!(f, "{lifetime} ")?;
                }
                write!(f, "{mutability}")?;
                f.write_str("self")
            }
            FunctionParamKind::SelfParam { mutability, ty } => {
                write!(f, "{mutability}")?;
                f.write_str("self")?;
                if let Some(ty) = ty {
                    write!(f, ": {ty}")?;
                }
                Ok(())
            }
        }
    }
}

impl Display for FunctionParamType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            FunctionParamType::Type(ty) => write!(f, "{ty}"),
            FunctionParamType::Variadic => f.write_str("..."),
        }
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Type::Tuple(types) => write_tuple(f, types),
            Type::Bool => write!(f, "bool"),
            Type::Str => write!(f, "str"),
            Type::TraitSelf => write!(f, "Self"),
            Type::Ref(lifetime, mutability, ty) => {
                f.write_str("&")?;
                if let Some(lifetime) = lifetime {
                    write!(f, "{lifetime} ")?;
                }
                write!(f, "{mutability}")?;
                write!(f, "{ty}")
            }
        }
    }
}

impl Display for BlockExpression {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if let Some(label) = &self.label {
            write!(f, "{label}: ")?;
        }
        f.write_str("{")?;
        for statement in &self.statements {
            write!(f, " {statement}")?;
        }
        if let Some(tail) = &self.tail {
            write!(f, " {tail}")?;
        }
        f.write_str(" }")
    }
}

impl Display for Statement {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Statement::Empty => f.write_str(";"),
            Statement::Item(item) => write!(f, "{item}"),
            Statement::Let {
                attrs,
                pattern,
                ty,
                initial_value,
                else_branch,
            } => {
                write!(f, "{} ", attrs.iter().format(" "))?;
                write!(f, "let {pattern}")?;
                if let Some(ty) = ty {
                    write!(f, ": {ty}")?;
                }
                if let Some(initial_value) = initial_value {
                    write!(f, " = {initial_value}")?;
                }
                if let Some(else_branch) = else_branch {
                    write!(f, " else {else_branch}")?;
                }
                f.write_str(";")
            }
            Statement::Expr(expression) if expression.is_with_block() => write!(f, "{expression}"),
            Statement::Expr(expression) => write!(f, "{expression};"),
        }
    }
}

impl Expression {
    fn is_with_block(&self) -> bool {
        matches!(self.kind, ExpressionKind::Block(_) | ExpressionKind::If(_))
    }
}

impl Display for Expression {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} ", self.attrs.iter().format(" "))?;
        write!(f, "{}", self.kind)
    }
}

impl Display for ExpressionKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ExpressionKind::Literal(literal) => write!(f, "{literal}"),
            ExpressionKind::Path(path) => write!(f, "{path}"),
            ExpressionKind::Operator(operator) => write!(f, "{operator}"),
            ExpressionKind::Virtual(virtual_expression) => write!(f, "{virtual_expression}"),
            ExpressionKind::Grouped(grouped) => write!(f, "({grouped})"),
            ExpressionKind::Block(block) => write!(f, "{block}"),
            ExpressionKind::If(if_expression) => write!(f, "{if_expression}"),
            ExpressionKind::Tuple(elements) => write_tuple(f, elements),
            ExpressionKind::Call(call) => write!(f, "{call}"),
            ExpressionKind::TupleIndexing(tuple_indexing) => write!(f, "{tuple_indexing}"),
        }
    }
}

impl Display for LiteralExpression {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            LiteralExpression::String(value) => write!(f, "\"{value}\""),
            LiteralExpression::Integer(value) => write!(f, "{value}"),
            LiteralExpression::Bool(value) => write!(f, "{value}"),
        }
    }
}

impl Display for IfExpression {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "if{}{}", self.condition, self.then_branch)?;
        if let Some(else_branch) = &self.else_branch {
            match &else_branch.kind {
                ExpressionKind::If(if_expression) if else_branch.attrs.is_empty() => {
                    write!(f, " else {if_expression}")?;
                }
                ExpressionKind::Block(block) if else_branch.attrs.is_empty() => {
                    write!(f, " else {block}")?;
                }
                _ => write!(f, " else {else_branch}")?,
            }
        }
        Ok(())
    }
}

impl Display for TupleIndexingExpression {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.expression, self.index)
    }
}

impl Display for CallExpression {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}({})", self.callee, self.args.iter().format(", "))
    }
}

impl Display for VirtualExpression {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            VirtualExpression::ValueToPlaceCoercion(expression) => {
                write!(f, "value_to_place!({expression})")
            }
            VirtualExpression::PlaceToValueCoercion(expression) => {
                write!(f, "place_to_value!({expression})")
            }
        }
    }
}

impl Display for OperatorExpression {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            OperatorExpression::Borrow(borrow) => write!(f, "{borrow}"),
            OperatorExpression::Dereference(dereference) => write!(f, "{dereference}"),
            OperatorExpression::Add(left, right) => write!(f, "{left} + {right}"),
            OperatorExpression::Assignment(left, right) => write!(f, "{left} = {right}"),
        }
    }
}

impl Display for BorrowExpression {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "&{}{}", self.mutability, self.expression)
    }
}

impl Display for DereferenceExpression {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "*{}", self.expression)
    }
}

impl Display for GenericParams {
    fn fmt(&self, _f: &mut Formatter<'_>) -> fmt::Result {
        Ok(())
    }
}

impl Display for WhereClauses {
    fn fmt(&self, _f: &mut Formatter<'_>) -> fmt::Result {
        Ok(())
    }
}

impl Display for OuterAttribute {
    fn fmt(&self, _f: &mut Formatter<'_>) -> fmt::Result {
        Ok(())
    }
}

impl Display for Lifetime {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str("'_")
    }
}

impl Display for Pattern {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Pattern::Identifier(name) => f.write_str(name),
            Pattern::Wildcard => f.write_str("_"),
        }
    }
}
