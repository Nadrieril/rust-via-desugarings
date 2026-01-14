//@ # Temporaries and Lifetime Extension
//@
//@ A "value-to-place coercion" occurs when a value expression is used in a context where a place is
//@ needed, e.g. because it is borrowed, matched on, or has a field accessed.
//@ See [this blog post](https://nadrieril.github.io/blog/2025/12/06/on-places-and-their-magic.html)
//@ for more details about place/value expressions and place/value contexts.
//@
//@ Whenever that happens, the value will get stored in a temporary variable. In this step, we make
//@ these temporaries explicit.
//@
//@ The rules that determine the scope of these temporaries are complex; they're described in [the
//@ Reference](https://doc.rust-lang.org/reference/destructors.html#temporary-scopes).
//@ You may also enjoy [this blog post](https://blog.m-ou.se/super-let/) with a more explanatory style.
//@
//@ In this step, for each expression `$expr` to be coerced, we first add a `let tmp;` statement,
//@ then assign it `tmp = $expr;` (these two steps can be merged), then use `tmp` where the expression was.
//@ The placement of the `let tmp;` determines how long the value will live and its drop order.
//@ To get the right scope, extra blocks `{ .. }` may be added.
//@
//@ For example:
//@ ```rust
//@ let s = if Option::is_some(&Option::clone(&opt)) {
//@     let _x = &42;
//@     &String::new()
//@ } else {
//@     &String::new()
//@ };
//@
//@ // becomes:
//@ let tmp3;
//@ let tmp4;
//@ let s = if { let tmp1 = Option::clone(&opt); Option::is_some(&tmp1) } {
//@     let tmp2 = 42;
//@     let _x = &tmp2;
//@     tmp3 = String::new();
//@     &tmp3
//@ } else {
//@     tmp4 = String::new();
//@     &tmp4
//@ };
//@ ```
//@ Or:
//@ ```rust
//@ let opt: RwLock<Option<u32>> = ...
//@ if let Some(x) = Option::as_ref(&*Result::unwrap(RwLock::read(&opt))) {
//@     ...
//@ } else {
//@     ...
//@ }
//@
//@ // becomes (in edition 2024):
//@ if let tmp = Result::unwrap(RwLock::read(&opt)) && let Some(x) = Option::as_ref(&*tmp) {
//@     ...
//@ } else {
//@     ...
//@ }
//@ ```
//@
//@ Note how in let chains we may introduce the temporaries as part of the let chain to get the
//@ right scope. Our [Extended Let Chains](../features/extended-let-chains.md) allow forward declarations
//@ `let x;` in the middle of a let chain for that purpose.
//@
//@ Taking an example from the [edition
//@ book](https://doc.rust-lang.org/edition-guide/rust-2024/temporary-tail-expr-scope.html):
//@
//@ ```rust
//@ fn f() -> usize {
//@     let c = RefCell::new("..");
//@     c.borrow().len()
//@ }
//@
//@ // Becomes, after method resolution:
//@ fn f() -> usize {
//@     let c = RefCell::new("..");
//@     str::len(*<Ref<_> as Deref>::deref(&RefCell::borrow(&c)))
//@ }
//@
//@ // Before 2024, this becomes:
//@ fn f() -> usize {
//@     let tmp1; // Added at the start of scope so that it drops after the other locals.
//@     let tmp2;
//@     let c = RefCell::new("..");
//@     tmp1 = RefCell::borrow(&c); // error[E0597]: `c` does not live long enough
//@     tmp2 = <Ref<_> as Deref>::deref(&tmp1);
//@     str::len(*tmp2)
//@ }
//@
//@ // After 2024, this becomes:
//@ fn f() -> usize {
//@     let c = RefCell::new("..");
//@     let tmp1; // drops before other locals
//@     let tmp2;
//@     tmp1 = RefCell::borrow(&c);
//@     tmp2 = <Ref<_> as Deref>::deref(&tmp1);
//@     str::len(*tmp2)
//@ }
//@ ```
//@
//@ There is an exception to the above: temporaries can, [when
//@ sensible](https://doc.rust-lang.org/reference/destructors.html#r-destructors.scope.const-promotion),
//@ become statics instead of local variables. This is called "constant promotion":
//@ ```rust
//@ let x = &1 + 2;
//@
//@ // becomes:
//@ static TMP: u32 = 1 + 2;
//@ let x = &TMP; // this allows `x` to have type `&'static u32`
//@ ```
//@
//@ After this step, all place contexts contain place expressions.
//@
//@ ## Implementation
//@
//@ This is a PoC implementation of a pass over the AST.
//! Entirely silly PoC desugaring pass.
use rustc_ast::Mutability; //#
use rustc_hir::{self as hir}; //#
use rustc_middle::middle::region::{self, ScopeData}; //#
use rustc_middle::thir::{self, Expr, ExprKind, Pat, PatKind, Stmt, StmtKind}; //#
use rustc_span::Symbol; //#
                        //#
use super::Body; //#
                 //#
pub struct ValueToPlaceDesugarer<'tcx, 'a> {
    body: &'a mut Body<'tcx>,
}

impl<'tcx, 'a> ValueToPlaceDesugarer<'tcx, 'a> {
    pub fn new(body: &'a mut Body<'tcx>) -> Self {
        Self { body }
    }

    pub fn run(&mut self) {
        // Only iterate over the original expressions; newly created ones don't need to be
        // revisited.
        for expr_id in self.body.thir.exprs.indices() {
            if let ExprKind::Borrow { borrow_kind, arg } =
                self.body.thir.exprs[expr_id].kind.clone()
            {
                if self.should_temp(arg) {
                    self.introduce_temp(expr_id, borrow_kind, arg);
                }
            }
        }
    }

    fn should_temp(&self, arg: thir::ExprId) -> bool {
        let peeled = self.peel_value_expr(arg);
        matches!(self.body.thir.exprs[peeled].kind, ExprKind::Call { .. })
    }

    fn peel_value_expr(&self, mut id: thir::ExprId) -> thir::ExprId {
        loop {
            match self.body.thir.exprs[id].kind {
                ExprKind::Scope { value, .. }
                | ExprKind::Use { source: value }
                | ExprKind::NeverToAny { source: value }
                | ExprKind::PlaceTypeAscription { source: value, .. }
                | ExprKind::ValueTypeAscription { source: value, .. }
                | ExprKind::PlaceUnwrapUnsafeBinder { source: value }
                | ExprKind::ValueUnwrapUnsafeBinder { source: value }
                | ExprKind::WrapUnsafeBinder { source: value } => {
                    id = value;
                }
                _ => break id,
            }
        }
    }

    fn fresh_scope(&mut self) -> region::Scope {
        region::Scope {
            local_id: self.body.new_hir_id().local_id,
            data: ScopeData::Node,
        }
    }

    //@ Some markdown explanation:
    fn introduce_temp(
        &mut self,
        expr_id: thir::ExprId,
        borrow_kind: rustc_middle::mir::BorrowKind,
        arg: thir::ExprId,
    ) {
        let hir_id = self.body.new_hir_id();
        let tmp_name = Symbol::intern("tmp");
        self.body.insert_synthetic_local_name(hir_id, tmp_name);

        let var_expr_id = self.body.thir.exprs.push(Expr {
            kind: ExprKind::VarRef {
                id: thir::LocalVarId(hir_id),
            },
            ..self.body.thir.exprs[arg]
        });

        let borrow_expr = Expr {
            kind: ExprKind::Borrow {
                borrow_kind,
                arg: var_expr_id,
            },
            ..self.body.thir.exprs[expr_id]
        };
        let borrow_expr_id = self.body.thir.exprs.push(borrow_expr);

        let arg_expr = &self.body.thir.exprs[arg];
        let pat = Pat {
            ty: arg_expr.ty,
            span: arg_expr.span,
            kind: PatKind::Binding {
                name: tmp_name,
                mode: hir::BindingMode(hir::ByRef::No, Mutability::Not),
                var: thir::LocalVarId(hir_id),
                ty: arg_expr.ty,
                subpattern: None,
                is_primary: true,
                is_shorthand: false,
            },
        };
        let assign_stmt = Stmt {
            kind: StmtKind::Let {
                remainder_scope: self.fresh_scope(),
                init_scope: self.fresh_scope(),
                pattern: Box::new(pat),
                initializer: Some(arg),
                else_block: None,
                lint_level: thir::LintLevel::Inherited,
                span: self.body.thir.exprs[arg].span,
            },
        };
        let stmt_id = self.body.thir.stmts.push(assign_stmt);

        let block = thir::Block {
            targeted_by_break: false,
            region_scope: self.fresh_scope(),
            span: self.body.thir.exprs[expr_id].span,
            stmts: Box::from([stmt_id]),
            expr: Some(borrow_expr_id),
            safety_mode: thir::BlockSafety::Safe,
        };
        let block_id = self.body.thir.blocks.push(block);
        self.body.thir.exprs[expr_id].kind = ExprKind::Block { block: block_id };
    }
}
