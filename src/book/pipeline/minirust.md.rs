//@ # MiniRust Translation
//@
//@ > This section is a work-in-progress experiment about making the book executable.
//@
//@ This best-effort translates our supported subset into MiniRust.
//@
//@ Disclaimer: this is entirely vibe-coded and does not reflect how this is intended to look in
//@ the end. In particular, the desugarings should make the translation as direct as possible.
//@ While we're experimenting, this translation may take liberties with that principle, for the
//@ sake of being able to run more examples.
use std::{
    cell::RefCell,
    collections::BTreeMap,
    io::{self, Write},
    rc::Rc,
};

use crate::{CompilationError, language};
use minirust_rs::{
    lang as mini,
    libspecr::{
        Name,
        hidden::{GcCompat, GcCow},
    },
    mem as memory,
    prelude::{
        Align, DynWrite, Int, List, Map, Mutability as MiniMutability, Size, TerminationInfo,
        x86_64,
    },
};

type MiniMemory = memory::BasicMemory<x86_64>;

pub fn translate_to_minirust(
    program: &language::Program,
) -> Result<mini::Program, CompilationError> {
    let main = find_main(program)?;
    let mut translator = Translator::new();
    let main_function = translator.translate_main(main)?;
    let main_name = mini::FnName(Name::from_internal(0));

    let mut functions = Map::new();
    functions.insert(main_name, main_function);
    Ok(mini::Program {
        functions,
        start: main_name,
        globals: translator.globals,
        traits: Map::new(),
        vtables: Map::new(),
    })
}

pub fn run_in_minirust(program: &language::Program) -> Result<String, CompilationError> {
    let program = translate_to_minirust(program)?;
    let stdout = SharedOutput::default();
    let stderr = SharedOutput::default();
    let mut machine = mini::Machine::<MiniMemory>::new(
        program,
        DynWrite::new(stdout.clone()),
        DynWrite::new(stderr.clone()),
    )
    .get_internal()
    .map_err(minirust_runtime_error)?;

    loop {
        match machine.step().get_internal() {
            Ok(()) => {}
            Err(TerminationInfo::MachineStop) => break,
            Err(error) => return Err(minirust_runtime_error(error)),
        }
    }

    let stderr = stderr.take_string()?;
    if !stderr.is_empty() {
        return Err(minirust_error(format!(
            "MiniRust wrote to stderr: {stderr}"
        )));
    }
    stdout.take_string()
}

#[derive(Clone, Default)]
struct SharedOutput {
    bytes: Rc<RefCell<Vec<u8>>>,
}

impl SharedOutput {
    fn take_string(&self) -> Result<String, CompilationError> {
        String::from_utf8(self.bytes.borrow().clone())
            .map_err(|error| minirust_error(format!("MiniRust produced non-UTF-8 output: {error}")))
    }
}

impl Write for SharedOutput {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.bytes.borrow_mut().extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl GcCompat for SharedOutput {
    fn points_to(&self, _buffer: &mut std::collections::HashSet<usize>) {}
}

struct Translator {
    globals: Map<mini::GlobalName, mini::Global>,
    locals: Map<mini::LocalName, mini::Type>,
    local_names: BTreeMap<String, mini::LocalName>,
    source_local_types: BTreeMap<String, language::Type>,
    blocks: Map<mini::BbName, mini::BasicBlock>,
    current_block: mini::BbName,
    current_statements: Vec<mini::Statement>,
    ret: mini::LocalName,
    next_local: u32,
    next_block: u32,
    next_global: u32,
}

impl Translator {
    fn new() -> Self {
        let ret = mini::LocalName(Name::from_internal(0));
        let current_block = mini::BbName(Name::from_internal(0));
        let mut locals = Map::new();
        locals.insert(ret, mini::unit_ty());
        Self {
            globals: Map::new(),
            locals,
            local_names: BTreeMap::new(),
            source_local_types: BTreeMap::new(),
            blocks: Map::new(),
            current_block,
            current_statements: Vec::new(),
            ret,
            next_local: 1,
            next_block: 1,
            next_global: 0,
        }
    }

    fn translate_main(
        &mut self,
        function: &language::Function,
    ) -> Result<mini::Function, CompilationError> {
        if !function.parameters.is_empty() {
            return Err(minirust_error(
                "MiniRust runner only supports `main` with no parameters",
            ));
        }
        match &function.body {
            language::FunctionBody::Block(block) => self.translate_block(block)?,
            language::FunctionBody::Missing => {
                return Err(minirust_error("MiniRust runner needs a `main` body"));
            }
        }
        self.finish_current_block(mini::Terminator::Intrinsic {
            intrinsic: mini::IntrinsicOp::Exit,
            arguments: List::new(),
            ret: mini::PlaceExpr::Local(self.ret),
            next_block: None,
        });

        Ok(mini::Function {
            locals: self.locals,
            args: List::new(),
            ret: self.ret,
            calling_convention: mini::CallingConvention::C,
            blocks: self.blocks,
            start: mini::BbName(Name::from_internal(0)),
        })
    }

    fn translate_block(
        &mut self,
        block: &language::BlockExpression,
    ) -> Result<(), CompilationError> {
        for statement in &block.statements {
            self.translate_statement(statement)?;
        }
        if let Some(tail) = &block.tail {
            self.translate_tail_expression(tail)?;
        }
        Ok(())
    }

    fn translate_statement(
        &mut self,
        statement: &language::Statement,
    ) -> Result<(), CompilationError> {
        match statement {
            language::Statement::Empty => Ok(()),
            language::Statement::Item(_) => Ok(()),
            language::Statement::Let {
                pattern,
                ty,
                initial_value,
                else_branch,
                ..
            } => {
                if else_branch.is_some() {
                    return Err(minirust_error(
                        "MiniRust runner does not yet support `let else`",
                    ));
                }
                let name = Self::pattern_name(pattern)?;
                let ty = ty
                    .as_ref()
                    .ok_or_else(|| minirust_error("MiniRust runner needs typed `let` bindings"))?;
                self.translate_let(name, ty)?;
                if let Some(initial_value) = initial_value {
                    self.translate_assignment_to(name, initial_value)?;
                }
                Ok(())
            }
            language::Statement::Expr(expression) => {
                self.translate_expression_statement(expression)
            }
        }
    }

    fn translate_expression_statement(
        &mut self,
        expression: &language::Expression,
    ) -> Result<(), CompilationError> {
        match &expression.kind {
            language::ExpressionKind::Grouped(_) => Err(minirust_error(
                "MiniRust runner expects grouped expressions to be desugared",
            )),
            language::ExpressionKind::Call(call) => self.translate_function_call(call),
            language::ExpressionKind::Operator(operator) => match &**operator {
                language::OperatorExpression::Assignment(target, value) => {
                    self.translate_assignment(target, value)
                }
                language::OperatorExpression::Borrow(_) => Err(minirust_error(
                    "MiniRust runner does not yet support borrow expressions as statements",
                )),
                language::OperatorExpression::Dereference(_) => Err(minirust_error(
                    "MiniRust runner does not yet support dereference expressions as statements",
                )),
                language::OperatorExpression::Add(..) => Err(minirust_error(
                    "MiniRust runner does not yet support `+` as a statement",
                )),
            },
            language::ExpressionKind::Tuple(language::TupleExpression::Unit) => Ok(()),
            other => Err(minirust_error(format!(
                "MiniRust runner does not yet support expression statement `{other:?}`"
            ))),
        }
    }

    fn translate_tail_expression(
        &mut self,
        expression: &language::Expression,
    ) -> Result<(), CompilationError> {
        match &expression.kind {
            language::ExpressionKind::Grouped(_) => Err(minirust_error(
                "MiniRust runner expects grouped expressions to be desugared",
            )),
            language::ExpressionKind::Tuple(language::TupleExpression::Unit) => Ok(()),
            language::ExpressionKind::Call(call) => self.translate_function_call(call),
            other => Err(minirust_error(format!(
                "MiniRust runner only supports `()` or `print(...)` tail expressions, got {other:?}"
            ))),
        }
    }

    fn translate_let(&mut self, name: &str, ty: &language::Type) -> Result<(), CompilationError> {
        if self.local_names.contains_key(name) {
            return Err(minirust_error(format!("duplicate local `{name}`")));
        }
        let local = mini::LocalName(Name::from_internal(self.next_local));
        self.next_local += 1;
        let mini_ty = translate_type(ty)?;
        self.locals.insert(local, mini_ty);
        self.local_names.insert(name.to_owned(), local);
        self.source_local_types.insert(name.to_owned(), ty.clone());
        self.current_statements
            .push(mini::Statement::StorageLive(local));
        Ok(())
    }

    fn translate_assignment(
        &mut self,
        target: &language::Expression,
        value: &language::Expression,
    ) -> Result<(), CompilationError> {
        let (destination, _) = self.translate_place(target)?;
        self.translate_assignment_to_place(destination, value)
    }

    fn translate_assignment_to(
        &mut self,
        target: &str,
        value: &language::Expression,
    ) -> Result<(), CompilationError> {
        let destination = self.local(target)?;
        self.translate_assignment_to_place(mini::PlaceExpr::Local(destination), value)
    }

    fn translate_assignment_to_place(
        &mut self,
        destination: mini::PlaceExpr,
        value: &language::Expression,
    ) -> Result<(), CompilationError> {
        let source = self.translate_value(value)?;
        self.current_statements.push(mini::Statement::Assign {
            destination,
            source,
        });
        Ok(())
    }

    fn translate_function_call(
        &mut self,
        call: &language::CallExpression,
    ) -> Result<(), CompilationError> {
        let name = Self::expression_path(&call.callee)?;
        if name != "print" {
            return Err(minirust_error(format!(
                "MiniRust runner only supports the `print` intrinsic, got `{}`",
                name
            )));
        }
        if call.args.len() != 1 {
            return Err(minirust_error(format!(
                "`print` expects one argument, got {}",
                call.args.len()
            )));
        }
        let argument = self.translate_value(&call.args[0])?;
        let next_block = self.fresh_block();
        self.finish_current_block(mini::Terminator::Intrinsic {
            intrinsic: mini::IntrinsicOp::PrintStdout,
            arguments: [argument].into_iter().collect(),
            ret: mini::PlaceExpr::Local(self.ret),
            next_block: Some(next_block),
        });
        self.current_block = next_block;
        Ok(())
    }

    fn translate_value(
        &mut self,
        expression: &language::Expression,
    ) -> Result<mini::ValueExpr, CompilationError> {
        match &expression.kind {
            language::ExpressionKind::Literal(language::LiteralExpression::Bool(value)) => Ok(
                mini::ValueExpr::Constant(mini::Constant::Bool(*value), mini::Type::Bool),
            ),
            language::ExpressionKind::Literal(language::LiteralExpression::String(value)) => {
                Ok(self.translate_string_literal(value))
            }
            language::ExpressionKind::Literal(language::LiteralExpression::Integer(value)) => {
                let ty = mini::IntType::usize_ty::<x86_64>();
                let value = Int::from(*value);
                if !ty.can_represent(value) {
                    return Err(minirust_error(format!(
                        "MiniRust runner only supports integer literals that fit in `usize`, got `{value}`"
                    )));
                }
                Ok(mini::ValueExpr::Constant(
                    mini::Constant::Int(value),
                    mini::Type::Int(ty),
                ))
            }
            language::ExpressionKind::Tuple(language::TupleExpression::Unit) => {
                Ok(mini::ValueExpr::Tuple(List::new(), mini::unit_ty()))
            }
            language::ExpressionKind::Grouped(_) => Err(minirust_error(
                "MiniRust runner expects grouped expressions to be desugared",
            )),
            language::ExpressionKind::Path(path) => {
                let name = Self::simple_path_name(path)?;
                Ok(mini::ValueExpr::Load {
                    source: GcCow::new(mini::PlaceExpr::Local(self.local(name)?)),
                })
            }
            language::ExpressionKind::Call(call) => Err(minirust_error(format!(
                "MiniRust runner only supports function calls as statements, got `{}`",
                call
            ))),
            language::ExpressionKind::Block(_) => Err(minirust_error(
                "MiniRust runner does not yet support nested block expressions",
            )),
            language::ExpressionKind::Operator(operator) => match &**operator {
                language::OperatorExpression::Borrow(borrow) => self.translate_borrow(borrow),
                language::OperatorExpression::Dereference(dereference) => {
                    let (source, _) = self.translate_dereference_place(dereference)?;
                    Ok(mini::ValueExpr::Load {
                        source: GcCow::new(source),
                    })
                }
                language::OperatorExpression::Add(..)
                | language::OperatorExpression::Assignment(..) => Err(minirust_error(format!(
                    "MiniRust runner does not yet support operator expression `{operator}` as a value"
                ))),
            },
        }
    }

    fn translate_borrow(
        &mut self,
        borrow: &language::BorrowExpression,
    ) -> Result<mini::ValueExpr, CompilationError> {
        let (value, _) = self.translate_borrow_with_pointee(borrow)?;
        Ok(value)
    }

    fn translate_borrow_with_pointee(
        &mut self,
        borrow: &language::BorrowExpression,
    ) -> Result<(mini::ValueExpr, mini::Type), CompilationError> {
        let (target, target_ty) = self.translate_place(&borrow.expression)?;
        let ptr_ty = ref_ptr_type(borrow.mutability, target_ty)?;
        Ok((
            mini::ValueExpr::AddrOf {
                target: GcCow::new(target),
                ptr_ty,
            },
            target_ty,
        ))
    }

    fn translate_place(
        &mut self,
        expression: &language::Expression,
    ) -> Result<(mini::PlaceExpr, mini::Type), CompilationError> {
        match &expression.kind {
            language::ExpressionKind::Grouped(_) => Err(minirust_error(
                "MiniRust runner expects grouped expressions to be desugared",
            )),
            language::ExpressionKind::Path(path) => {
                let name = Self::simple_path_name(path)?;
                let local = self.local(name)?;
                Ok((mini::PlaceExpr::Local(local), self.local_type(local)?))
            }
            language::ExpressionKind::Operator(operator) => match &**operator {
                language::OperatorExpression::Dereference(dereference) => {
                    self.translate_dereference_place(dereference)
                }
                other => Err(minirust_error(format!(
                    "MiniRust runner expected a place expression, got `{other}`"
                ))),
            },
            other => Err(minirust_error(format!(
                "MiniRust runner expected a place expression, got `{other:?}`"
            ))),
        }
    }

    fn translate_dereference_place(
        &mut self,
        dereference: &language::DereferenceExpression,
    ) -> Result<(mini::PlaceExpr, mini::Type), CompilationError> {
        let (operand, pointee_ty) = self.translate_pointer_value(&dereference.expression)?;
        Ok((
            mini::PlaceExpr::Deref {
                operand: GcCow::new(operand),
                ty: pointee_ty,
            },
            pointee_ty,
        ))
    }

    fn translate_pointer_value(
        &mut self,
        expression: &language::Expression,
    ) -> Result<(mini::ValueExpr, mini::Type), CompilationError> {
        match &expression.kind {
            language::ExpressionKind::Grouped(_) => Err(minirust_error(
                "MiniRust runner expects grouped expressions to be desugared",
            )),
            language::ExpressionKind::Path(path) => {
                let name = Self::simple_path_name(path)?;
                let language::Type::Ref(_, _, pointee_ty) = self.source_local_type(name)? else {
                    return Err(minirust_error(format!(
                        "MiniRust runner can only dereference references, got `{name}`"
                    )));
                };
                Ok((
                    mini::ValueExpr::Load {
                        source: GcCow::new(mini::PlaceExpr::Local(self.local(name)?)),
                    },
                    translate_type(pointee_ty)?,
                ))
            }
            language::ExpressionKind::Operator(operator) => match &**operator {
                language::OperatorExpression::Borrow(borrow) => {
                    self.translate_borrow_with_pointee(borrow)
                }
                other => Err(minirust_error(format!(
                    "MiniRust runner expected a reference value, got `{other}`"
                ))),
            },
            other => Err(minirust_error(format!(
                "MiniRust runner expected a reference value, got `{other:?}`"
            ))),
        }
    }

    fn expression_path(expression: &language::Expression) -> Result<&str, CompilationError> {
        match &expression.kind {
            language::ExpressionKind::Grouped(_) => Err(minirust_error(
                "MiniRust runner expects grouped expressions to be desugared",
            )),
            language::ExpressionKind::Path(path) => Self::simple_path_name(path),
            other => Err(minirust_error(format!(
                "MiniRust runner expected a path expression, got `{other:?}`"
            ))),
        }
    }

    fn simple_path_name(path: &language::PathExpression) -> Result<&str, CompilationError> {
        Ok(path)
    }

    fn pattern_name(pattern: &language::Pattern) -> Result<&str, CompilationError> {
        match pattern {
            language::Pattern::Identifier(name) => Ok(name),
            language::Pattern::Wildcard => {
                Err(minirust_error("MiniRust runner needs named `let` bindings"))
            }
        }
    }

    fn translate_string_literal(&mut self, value: &str) -> mini::ValueExpr {
        // TODO: do as a desugaring instead.
        let global_name = mini::GlobalName(Name::from_internal(self.next_global));
        self.next_global += 1;
        self.globals.insert(
            global_name,
            mini::Global {
                bytes: value.as_bytes().iter().copied().map(Some).collect(),
                relocations: List::new(),
                align: Align::ONE,
            },
        );

        let thin_pointer = mini::ValueExpr::Constant(
            mini::Constant::GlobalPointer(mini::Relocation {
                name: global_name,
                offset: Size::ZERO,
            }),
            mini::Type::Ptr(memory::PtrType::Raw {
                meta_kind: memory::PointerMetaKind::None,
            }),
        );
        let length = mini::ValueExpr::Constant(
            mini::Constant::Int(Int::from(value.len())),
            mini::Type::Int(mini::IntType::usize_ty::<x86_64>()),
        );
        mini::ValueExpr::BinOp {
            operator: mini::BinOp::ConstructWidePointer(str_ref_ptr_type()),
            left: GcCow::new(thin_pointer),
            right: GcCow::new(length),
        }
    }

    fn fresh_block(&mut self) -> mini::BbName {
        let block = mini::BbName(Name::from_internal(self.next_block));
        self.next_block += 1;
        block
    }

    fn finish_current_block(&mut self, terminator: mini::Terminator) {
        self.blocks.insert(
            self.current_block,
            mini::BasicBlock {
                statements: std::mem::take(&mut self.current_statements)
                    .into_iter()
                    .collect(),
                terminator,
                kind: mini::BbKind::Regular,
            },
        );
    }

    fn local(&self, name: &str) -> Result<mini::LocalName, CompilationError> {
        self.local_names
            .get(name)
            .copied()
            .ok_or_else(|| minirust_error(format!("unknown local `{name}`")))
    }

    fn local_type(&self, local: mini::LocalName) -> Result<mini::Type, CompilationError> {
        self.locals
            .get(local)
            .ok_or_else(|| minirust_error(format!("unknown MiniRust local `{local:?}`")))
    }

    fn source_local_type(&self, name: &str) -> Result<&language::Type, CompilationError> {
        self.source_local_types
            .get(name)
            .ok_or_else(|| minirust_error(format!("unknown local `{name}`")))
    }
}

fn find_main(program: &language::Program) -> Result<&language::Function, CompilationError> {
    program
        .items
        .iter()
        .filter_map(|item| match &item.kind {
            language::ItemKind::Function(function) => Some(function),
        })
        .find(|function| function.name == "main")
        .ok_or_else(|| minirust_error("MiniRust runner needs a `main` function"))
}

fn translate_type(ty: &language::Type) -> Result<mini::Type, CompilationError> {
    match ty {
        language::Type::Unit => Ok(mini::unit_ty()),
        language::Type::Bool => Ok(mini::Type::Bool),
        language::Type::Ref(_, mutability, inner) if matches!(**inner, language::Type::Str) => {
            match mutability {
                language::Mutability::Immutable => Ok(mini::Type::Ptr(str_ref_ptr_type())),
                language::Mutability::Mutable => Err(minirust_error(
                    "MiniRust runner only supports shared `&str`",
                )),
            }
        }
        language::Type::Ref(_, mutability, inner) => {
            let pointee_ty = translate_type(inner)?;
            Ok(mini::Type::Ptr(ref_ptr_type(*mutability, pointee_ty)?))
        }
        language::Type::Str => Err(minirust_error(
            "MiniRust runner only supports `str` behind a reference",
        )),
        language::Type::TraitSelf => Err(minirust_error(format!(
            "MiniRust runner does not yet support type `{ty}`"
        ))),
    }
}

fn ref_ptr_type(
    mutability: language::Mutability,
    pointee_ty: mini::Type,
) -> Result<memory::PtrType, CompilationError> {
    Ok(memory::PtrType::Ref {
        mutbl: translate_mutability(mutability),
        pointee: sized_pointee_info(pointee_ty)?,
    })
}

fn sized_pointee_info(ty: mini::Type) -> Result<memory::PointeeInfo, CompilationError> {
    let layout = ty.layout::<x86_64>();
    let memory::LayoutStrategy::Sized(..) = layout else {
        return Err(minirust_error(format!(
            "MiniRust runner only supports references to sized types, got `{ty:?}`"
        )));
    };
    Ok(memory::PointeeInfo {
        layout,
        inhabited: true,
        unsafe_cells: memory::UnsafeCellStrategy::Sized { cells: List::new() },
        freeze: true,
        unpin: true,
    })
}

fn translate_mutability(mutability: language::Mutability) -> MiniMutability {
    match mutability {
        language::Mutability::Mutable => MiniMutability::Mutable,
        language::Mutability::Immutable => MiniMutability::Immutable,
    }
}

fn str_ref_ptr_type() -> memory::PtrType {
    memory::PtrType::Ref {
        mutbl: MiniMutability::Immutable,
        pointee: memory::PointeeInfo {
            layout: memory::LayoutStrategy::Slice(Size::from_bytes_const(1), Align::ONE),
            inhabited: true,
            unsafe_cells: memory::UnsafeCellStrategy::Slice {
                element_cells: List::new(),
            },
            freeze: true,
            unpin: true,
        },
    }
}

fn minirust_runtime_error(error: TerminationInfo) -> CompilationError {
    minirust_error(format!("MiniRust execution failed: {error:?}"))
}

fn minirust_error(message: impl Into<String>) -> CompilationError {
    CompilationError::MiniRust(message.into())
}
