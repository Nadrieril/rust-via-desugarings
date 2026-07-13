//@ # Formality Checks
//@
//@ > This section is a work-in-progress experiment about making the book executable.
//@
//@ This best-effort translates our supported subset into a-mir-formality's expression language to
//@ run various checks on it, such as borrow-checking.
//@
//@ Disclaimer: this is entirely vibe-coded and does not reflect how this is intended to look in
//@ the end. In particular, the desugarings should make the translation as direct as possible.
//@ While we're experimenting, this translation may take liberties with that principle, for the
//@ sake of being able to run more examples.
use std::sync::Arc;

use crate::{CompilationError, language};
use formality_rust::{
    check,
    grammar::{
        self as rust, Crate as RustCrate, CrateId, CrateItem, Crates, FieldName, Fn as RustFn,
        FnBody, FnBoundData, InputArg, Lt, MaybeFnBody, Parameter, RefKind, RigidName, ScalarId,
        Ty, ValueId, expr as rust_expr,
    },
    prove::prove::Safety,
};

pub fn translate_to_formality(program: &language::Program) -> Result<Crates, CompilationError> {
    FormalityTranslator.translate_program(program)
}

pub fn check_with_formality(program: &language::Program) -> Result<(), CompilationError> {
    let crates = translate_to_formality(program)?;
    let _checked = check::check_all_crates(crates.clone())
        .into_singleton()
        .map_err(|error| {
            formality_error(format!(
                "a-mir-formality borrow check failed: {}",
                error.format_leaves()
            ))
        })?;
    Ok(())
}

struct FormalityTranslator;

impl FormalityTranslator {
    fn translate_program(&self, program: &language::Program) -> Result<Crates, CompilationError> {
        let items = program
            .items
            .iter()
            .map(|item| {
                let language::ItemKind::Function(function) = &item.kind;
                self.translate_function(function).map(CrateItem::Fn)
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Crates {
            crates: vec![RustCrate {
                id: CrateId::new("current"),
                items,
            }],
        })
    }

    fn translate_function(
        &self,
        function: &language::Function,
    ) -> Result<RustFn, CompilationError> {
        let input_args = function
            .parameters
            .iter()
            .map(Self::translate_parameter)
            .collect::<Result<Vec<_>, _>>()?;
        let output_ty = function
            .return_type
            .as_ref()
            .map_or(Ok(Ty::unit()), translate_type)?;
        let body = match &function.body {
            language::FunctionBody::Block(block) => MaybeFnBody::FnBody(FnBody::Expr(
                FunctionTranslator::new().translate_body(block)?,
            )),
            language::FunctionBody::Missing => MaybeFnBody::NoFnBody,
        };
        Ok(RustFn {
            id: ValueId::new(&function.name),
            safety: translate_item_safety(function.qualifiers.safety.as_ref()),
            binder: rust::Binder::dummy(FnBoundData {
                input_args,
                output_ty,
                where_clauses: vec![],
                body,
            }),
        })
    }

    fn translate_parameter(
        parameter: &language::FunctionParam,
    ) -> Result<InputArg, CompilationError> {
        let language::FunctionParamKind::Regular {
            pattern: Some(pattern),
            ty: language::FunctionParamType::Type(ty),
        } = &parameter.kind
        else {
            return Err(formality_error(format!(
                "formality translation only supports named regular parameters, got `{parameter}`"
            )));
        };
        Ok(InputArg {
            id: ValueId::new(pattern_name(pattern)?),
            ty: translate_type(ty)?,
        })
    }
}

#[derive(Default)]
struct FunctionTranslator {
    lifetimes: Vec<rust::BoundVar>,
}

impl FunctionTranslator {
    fn new() -> Self {
        Self::default()
    }

    fn translate_body(
        mut self,
        block: &language::BlockExpression,
    ) -> Result<rust_expr::Block, CompilationError> {
        let block = self.translate_block(block)?;
        if self.lifetimes.is_empty() {
            Ok(block)
        } else {
            Ok(rust_expr::Block {
                label: None,
                stmts: vec![rust_expr::Stmt::Exists {
                    binder: rust::Binder::new(self.lifetimes, block),
                }],
            })
        }
    }

    fn translate_block(
        &mut self,
        block: &language::BlockExpression,
    ) -> Result<rust_expr::Block, CompilationError> {
        let mut stmts = Vec::new();
        for statement in &block.statements {
            self.translate_statement(statement, &mut stmts)?;
        }
        if let Some(tail) = &block.tail {
            self.translate_expression_statement(tail, &mut stmts)?;
        }
        Ok(rust_expr::Block { label: None, stmts })
    }

    fn translate_statement(
        &mut self,
        statement: &language::Statement,
        stmts: &mut Vec<rust_expr::Stmt>,
    ) -> Result<(), CompilationError> {
        match statement {
            language::Statement::Empty | language::Statement::Item(_) => Ok(()),
            language::Statement::Let {
                pattern,
                ty,
                initial_value,
                else_branch,
                ..
            } => {
                if else_branch.is_some() {
                    return Err(formality_error(
                        "formality translation does not yet support `let else`",
                    ));
                }
                let ty = ty.as_ref().ok_or_else(|| {
                    formality_error("formality translation needs typed `let` bindings")
                })?;
                let init = initial_value
                    .as_ref()
                    .map(|value| {
                        self.translate_expression(value)
                            .map(|expr| rust_expr::Init { expr })
                    })
                    .transpose()?;
                stmts.push(rust_expr::Stmt::Let {
                    label: None,
                    id: ValueId::new(pattern_name(pattern)?),
                    ty: self.translate_type(ty)?,
                    init,
                });
                Ok(())
            }
            language::Statement::Expr(expression) => {
                self.translate_expression_statement(expression, stmts)
            }
        }
    }

    fn translate_expression_statement(
        &mut self,
        expression: &language::Expression,
        stmts: &mut Vec<rust_expr::Stmt>,
    ) -> Result<(), CompilationError> {
        if matches!(
            &expression.kind,
            language::ExpressionKind::Tuple(elements) if elements.is_empty()
        ) {
            return Ok(());
        }

        if let language::ExpressionKind::If(if_expression) = &expression.kind {
            self.translate_if_statement(if_expression, stmts)?;
            return Ok(());
        }

        if let language::ExpressionKind::Call(call) = &expression.kind {
            if expression_path(&call.callee)? == "print" {
                if call.args.len() != 1 {
                    return Err(formality_error(format!(
                        "`print` expects one argument, got {}",
                        call.args.len()
                    )));
                }
                stmts.push(rust_expr::Stmt::Print {
                    expr: self.translate_expression(&call.args[0])?,
                });
                return Ok(());
            }
        }

        stmts.push(rust_expr::Stmt::Expr {
            expr: self.translate_expression(expression)?,
        });
        Ok(())
    }

    fn translate_if_statement(
        &mut self,
        if_expression: &language::IfExpression,
        stmts: &mut Vec<rust_expr::Stmt>,
    ) -> Result<(), CompilationError> {
        stmts.push(rust_expr::Stmt::If {
            condition: self.translate_expression(&if_expression.condition)?,
            then_block: self.translate_if_branch(&if_expression.then_branch)?,
            else_block: self.translate_if_else_branch(if_expression.else_branch.as_deref())?,
        });
        Ok(())
    }

    fn translate_if_else_branch(
        &mut self,
        else_branch: Option<&language::Expression>,
    ) -> Result<rust_expr::Block, CompilationError> {
        match else_branch {
            None => Err(formality_error(
                "formality translation expects `if` expressions without `else` to be desugared",
            )),
            Some(branch) => self.translate_if_branch(branch),
        }
    }

    fn translate_if_branch(
        &mut self,
        branch: &language::Expression,
    ) -> Result<rust_expr::Block, CompilationError> {
        match &branch.kind {
            language::ExpressionKind::Block(block) if branch.attrs.is_empty() => {
                self.translate_block(block)
            }
            language::ExpressionKind::If(if_expression) if branch.attrs.is_empty() => {
                let mut stmts = Vec::new();
                self.translate_if_statement(if_expression, &mut stmts)?;
                Ok(rust_expr::Block { label: None, stmts })
            }
            _ => Err(formality_error(format!(
                "formality translation expected an `if` branch, got `{branch:?}`"
            ))),
        }
    }

    fn translate_expression(
        &mut self,
        expression: &language::Expression,
    ) -> Result<rust_expr::Expr, CompilationError> {
        match &expression.kind {
            language::ExpressionKind::Literal(language::LiteralExpression::Bool(true)) => {
                Ok(rust_expr::Expr::True)
            }
            language::ExpressionKind::Literal(language::LiteralExpression::Bool(false)) => {
                Ok(rust_expr::Expr::False)
            }
            language::ExpressionKind::Literal(language::LiteralExpression::Integer(value)) => {
                let value = usize::try_from(*value).map_err(|_| {
                    formality_error(format!(
                        "formality translation only supports integer literals that fit in `usize`, got `{value}`"
                    ))
                })?;
                Ok(rust_expr::Expr::Literal {
                    value,
                    ty: ScalarId::Usize,
                })
            }
            language::ExpressionKind::Literal(language::LiteralExpression::String(_)) => Err(
                formality_error("formality translation does not yet support string literals"),
            ),
            language::ExpressionKind::Path(path) => {
                Ok(rust_expr::Expr::Place(Self::translate_simple_path(path)))
            }
            language::ExpressionKind::Call(call) => Ok(rust_expr::Expr::Call {
                callee: Arc::new(self.translate_expression(&call.callee)?),
                args: call
                    .args
                    .iter()
                    .map(|argument| self.translate_expression(argument))
                    .collect::<Result<Vec<_>, _>>()?,
            }),
            language::ExpressionKind::Operator(operator) => match &**operator {
                language::OperatorExpression::Borrow(borrow) => Ok(rust_expr::Expr::Ref {
                    kind: translate_ref_kind(borrow.mutability),
                    lt: self.fresh_lifetime(),
                    place: self.translate_place(&borrow.expression)?,
                }),
                language::OperatorExpression::Dereference(dereference) => Ok(
                    rust_expr::Expr::Place(self.translate_dereference(dereference)?),
                ),
                language::OperatorExpression::Assignment(target, value) => {
                    Ok(rust_expr::Expr::Assign {
                        place: self.translate_place(target)?,
                        expr: Arc::new(self.translate_expression(value)?),
                    })
                }
                language::OperatorExpression::Add(..) => Err(formality_error(
                    "formality translation does not yet support `+`",
                )),
            },
            language::ExpressionKind::Grouped(_) => Err(formality_error(
                "formality translation expects grouped expressions to be desugared",
            )),
            language::ExpressionKind::Block(_) => Err(formality_error(
                "formality translation does not yet support nested block expressions",
            )),
            language::ExpressionKind::If(_) => Err(formality_error(
                "formality translation does not yet support `if` as a value",
            )),
            language::ExpressionKind::Tuple(_) => Err(formality_error(
                "formality translation does not yet support tuple expressions",
            )),
            language::ExpressionKind::TupleIndexing(tuple_indexing) => Ok(rust_expr::Expr::Place(
                self.translate_tuple_indexing(tuple_indexing)?,
            )),
        }
    }

    fn translate_place(
        &mut self,
        expression: &language::Expression,
    ) -> Result<rust_expr::PlaceExpr, CompilationError> {
        match &expression.kind {
            language::ExpressionKind::Path(path) => Ok(Self::translate_simple_path(path)),
            language::ExpressionKind::TupleIndexing(tuple_indexing) => {
                self.translate_tuple_indexing(tuple_indexing)
            }
            language::ExpressionKind::Operator(operator) => match &**operator {
                language::OperatorExpression::Dereference(dereference) => {
                    self.translate_dereference(dereference)
                }
                other => Err(formality_error(format!(
                    "formality translation expected a place expression, got `{other}`"
                ))),
            },
            language::ExpressionKind::Grouped(_) => Err(formality_error(
                "formality translation expects grouped expressions to be desugared",
            )),
            other => Err(formality_error(format!(
                "formality translation expected a place expression, got `{other:?}`"
            ))),
        }
    }

    fn translate_dereference(
        &mut self,
        dereference: &language::DereferenceExpression,
    ) -> Result<rust_expr::PlaceExpr, CompilationError> {
        Ok(rust_expr::PlaceExpr::Deref {
            prefix: Arc::new(self.translate_place(&dereference.expression)?),
        })
    }

    fn translate_tuple_indexing(
        &mut self,
        tuple_indexing: &language::TupleIndexingExpression,
    ) -> Result<rust_expr::PlaceExpr, CompilationError> {
        Ok(rust_expr::PlaceExpr::Field {
            prefix: Arc::new(self.translate_place(&tuple_indexing.expression)?),
            field_name: FieldName::Index(tuple_indexing.index),
        })
    }

    fn translate_simple_path(path: &language::PathExpression) -> rust_expr::PlaceExpr {
        rust_expr::PlaceExpr::Var(ValueId::new(path))
    }

    fn translate_type(&mut self, ty: &language::Type) -> Result<Ty, CompilationError> {
        match ty {
            language::Type::Tuple(types) => {
                let parameters = types
                    .iter()
                    .map(|ty| {
                        self.translate_type(ty)
                            .map(|ty| Parameter::Ty(Arc::new(ty)))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(Ty::rigid(RigidName::Tuple(types.len()), parameters))
            }
            language::Type::Bool => Ok(Ty::bool()),
            language::Type::Ref(_, mutability, inner) => {
                let inner = self.translate_type(inner)?;
                Ok(match mutability {
                    language::Mutability::Immutable => inner.ref_ty(self.fresh_lifetime()),
                    language::Mutability::Mutable => inner.ref_mut_ty(self.fresh_lifetime()),
                })
            }
            language::Type::Str => Err(formality_error(
                "formality translation does not yet support `str`",
            )),
            language::Type::TraitSelf => Err(formality_error(format!(
                "formality translation does not yet support type `{ty}`"
            ))),
        }
    }

    fn fresh_lifetime(&mut self) -> Lt {
        let lifetime = rust::BoundVar::fresh(rust::ParameterKind::Lt);
        self.lifetimes.push(lifetime);
        Lt::Variable(rust::Variable::BoundVar(lifetime))
    }
}

fn translate_type(ty: &language::Type) -> Result<Ty, CompilationError> {
    match ty {
        language::Type::Tuple(types) => {
            let parameters = types
                .iter()
                .map(|ty| translate_type(ty).map(|ty| Parameter::Ty(Arc::new(ty))))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(Ty::rigid(RigidName::Tuple(types.len()), parameters))
        }
        language::Type::Bool => Ok(Ty::bool()),
        language::Type::Ref(_, mutability, inner) => {
            let inner = translate_type(inner)?;
            Ok(match mutability {
                language::Mutability::Immutable => inner.ref_ty(Lt::Erased),
                language::Mutability::Mutable => inner.ref_mut_ty(Lt::Erased),
            })
        }
        language::Type::Str => Err(formality_error(
            "formality translation does not yet support `str`",
        )),
        language::Type::TraitSelf => Err(formality_error(format!(
            "formality translation does not yet support type `{ty}`"
        ))),
    }
}

fn translate_ref_kind(mutability: language::Mutability) -> RefKind {
    match mutability {
        language::Mutability::Mutable => RefKind::Mut,
        language::Mutability::Immutable => RefKind::Shared,
    }
}

fn translate_item_safety(safety: Option<&language::ItemSafety>) -> Safety {
    match safety {
        Some(language::ItemSafety::Unsafe) => Safety::Unsafe,
        Some(language::ItemSafety::Safe) | None => Safety::Safe,
    }
}

fn expression_path(expression: &language::Expression) -> Result<&str, CompilationError> {
    match &expression.kind {
        language::ExpressionKind::Path(path) => Ok(path),
        language::ExpressionKind::Grouped(_) => Err(formality_error(
            "formality translation expects grouped expressions to be desugared",
        )),
        other => Err(formality_error(format!(
            "formality translation expected a path expression, got `{other:?}`"
        ))),
    }
}

fn pattern_name(pattern: &language::Pattern) -> Result<&str, CompilationError> {
    match pattern {
        language::Pattern::Identifier(name) => Ok(name),
        language::Pattern::Wildcard => Err(formality_error(
            "formality translation needs named bindings",
        )),
    }
}

fn formality_error(message: impl Into<String>) -> CompilationError {
    CompilationError::Formality(message.into())
}
