use std::collections::HashMap;

use rustc_ast::Mutability;
use rustc_hir::{self as hir, def_id::LocalDefId};
use rustc_middle::{
    middle::region::{self, ScopeData},
    thir::{self, Expr, ExprId, ExprKind, Pat, PatKind, Stmt, StmtKind},
    ty::TyCtxt,
};
use rustc_span::Symbol;

pub struct Body<'tcx> {
    pub def_id: LocalDefId,
    pub thir: thir::Thir<'tcx>,
    pub root_expr: ExprId,
    synthetic_local_names: HashMap<hir::HirId, String>,
    next_local_id: hir::ItemLocalId,
}

impl<'tcx> Body<'tcx> {
    pub fn new(
        tcx: TyCtxt<'tcx>,
        def_id: LocalDefId,
        body: thir::Thir<'tcx>,
        root_expr: ExprId,
    ) -> Self {
        let owner_id = hir::OwnerId { def_id };
        let max_id = tcx.hir_owner_nodes(owner_id).nodes.len();
        Self {
            def_id,
            thir: body,
            root_expr,
            synthetic_local_names: Default::default(),
            next_local_id: hir::ItemLocalId::from_usize(max_id),
        }
    }

    pub fn owner(&self) -> hir::OwnerId {
        hir::OwnerId {
            def_id: self.def_id,
        }
    }

    pub fn synthetic_local_name(&self, id: hir::HirId) -> Option<&str> {
        self.synthetic_local_names.get(&id).map(|x| x.as_str())
    }

    fn new_hir_id(&mut self) -> hir::HirId {
        let local_id = self.next_local_id;
        self.next_local_id = hir::ItemLocalId::from_u32(self.next_local_id.as_u32() + 1);
        hir::HirId {
            owner: self.owner(),
            local_id,
        }
    }
}

pub fn desugar_thir<'tcx>(_tcx: TyCtxt<'tcx>, body: &mut Body<'tcx>) {
    if false {
        ValueToPlaceDesugarer::new(body).run();
    }
}

// Entirely silly PoC desugaring pass.
struct ValueToPlaceDesugarer<'tcx, 'a> {
    body: &'a mut Body<'tcx>,
}

impl<'tcx, 'a> ValueToPlaceDesugarer<'tcx, 'a> {
    fn new(body: &'a mut Body<'tcx>) -> Self {
        Self { body }
    }

    fn run(&mut self) {
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

    fn introduce_temp(
        &mut self,
        expr_id: thir::ExprId,
        borrow_kind: rustc_middle::mir::BorrowKind,
        arg: thir::ExprId,
    ) {
        let hir_id = self.body.new_hir_id();
        let tmp_name = format!("tmp{}", hir_id.local_id.as_u32());
        self.body
            .synthetic_local_names
            .insert(hir_id, tmp_name.clone());

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
                name: Symbol::intern(&tmp_name),
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
