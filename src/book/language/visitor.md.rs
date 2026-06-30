//@ # Visiting
//@
//@ > This section is a work-in-progress experiment about making the book executable.
//@
//@ These traits provide an overrideable visitor over the language AST.
use std::{any::Any, marker::PhantomData}; //#

use crate::language::*; //#
use derive_generic_visitor::*; //#

#[rustfmt::skip]
#[visitable_group(
    visitor(drive(&VisitAst)),
    visitor(drive_mut(&mut VisitAstMut)),
    skip(
        (), String, bool, char,
        u8, u16, u32, u64, u128, usize,
        i8, i16, i32, i64, i128, isize,
    ),
    drive(
        for<T: AstVisitable> Box<T>,
        for<T: AstVisitable> Option<T>,
        for<T: AstVisitable> Vec<T>,
        for<A: AstVisitable, B: AstVisitable> (A, B),
        for<A: AstVisitable, B: AstVisitable, C: AstVisitable> (A, B, C),
        for<A: AstVisitable, B: AstVisitable> Result<A, B>,
    ),
    override(
        BlockExpression,
        CallExpression,
        ExternAbi,
        Expression,
        ExpressionKind,
        Function,
        FunctionBody,
        FunctionParam,
        FunctionParamKind,
        FunctionParamType,
        FunctionQualifiers,
        GenericParams,
        InnerAttribute,
        Item,
        ItemKind,
        ItemSafety,
        Lifetime,
        LiteralExpression,
        Mutability,
        OperatorExpression,
        OuterAttribute,
        Pattern,
        Program,
        Statement,
        TupleExpression,
        Type,
        Visibility,
        WhereClauses,
    )
)]
pub trait AstVisitable: Any {
    /// The name of the type, used for debug logging.
    fn name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }

    /// Visit all occurrences of that type inside `self`, in pre-order traversal.
    fn visit_all_infallible<T: AstVisitable>(&self, mut f: impl FnMut(&T)) {
        let _ = self.visit_all::<T, ()>(|x| Ok(f(x)));
    }

    /// Visit all occurrences of that type inside `self`, in pre-order traversal.
    fn visit_all_mut_infallible<T: AstVisitable>(&mut self, mut f: impl FnMut(&mut T)) {
        let _ = self.visit_all_mut::<T, ()>(|x| Ok(f(x)));
    }

    /// Visit all occurrences of that type inside `self`, in pre-order traversal.
    fn visit_all<T: AstVisitable, E>(&self, f: impl FnMut(&T) -> Result<(), E>) -> Result<(), E> {
        match self.drive(&mut DynVisitor::new_shared(f)) {
            Continue(()) => Ok(()),
            Break(e) => Err(e),
        }
    }

    /// Visit all occurrences of that type inside `self`, in pre-order traversal.
    fn visit_all_mut<T: AstVisitable, E>(&mut self, f: impl FnMut(&mut T) -> Result<(), E>) -> Result<(), E> {
        match self.drive_mut(&mut DynVisitor::new_mut(f)) {
            Continue(()) => Ok(()),
            Break(e) => Err(e),
        }
    }
}

/// Ast visitor that uses dynamic dispatch to call the provided function on the visited values of
/// the right type.
pub struct DynVisitor<F, T: Any, E> {
    enter: F,
    phantom: PhantomData<(T, E)>,
}

impl<F, T: Any, E> Visitor for DynVisitor<F, T, E> {
    type Break = E;
}

impl<F, T: Any, E> DynVisitor<F, T, E>
where
    F: FnMut(&T) -> Result<(), E>,
{
    pub fn new_shared(enter: F) -> Self {
        DynVisitor {
            enter,
            phantom: PhantomData,
        }
    }
}
impl<F, T: Any, E> DynVisitor<F, T, E>
where
    F: FnMut(&mut T) -> Result<(), E>,
{
    pub fn new_mut(enter: F) -> Self {
        DynVisitor {
            enter,
            phantom: PhantomData,
        }
    }
}

impl<F, T: Any, E> VisitAst for DynVisitor<F, T, E>
where
    F: FnMut(&T) -> Result<(), E>,
{
    fn visit<U: AstVisitable>(&mut self, x: &U) -> ControlFlow<Self::Break> {
        if let Some(x) = (x as &dyn Any).downcast_ref::<T>() {
            match (self.enter)(x) {
                Ok(()) => {}
                Err(e) => return Break(e),
            }
        }
        x.drive(self)?;
        Continue(())
    }
}

impl<F, T: Any, E> VisitAstMut for DynVisitor<F, T, E>
where
    F: FnMut(&mut T) -> Result<(), E>,
{
    fn visit<U: AstVisitable>(&mut self, x: &mut U) -> ControlFlow<Self::Break> {
        if let Some(x) = (x as &mut dyn Any).downcast_mut::<T>() {
            match (self.enter)(x) {
                Ok(()) => {}
                Err(e) => return Break(e),
            }
        }
        x.drive_mut(self)?;
        Continue(())
    }
}
