use std::collections::HashMap;

use rustc_hir::{self as hir, def_id::LocalDefId};
use rustc_middle::{
    thir::{self, ExprId},
    ty::TyCtxt,
};
use rustc_span::Symbol;

mod temporaries;

pub struct Body<'tcx> {
    pub def_id: LocalDefId,
    pub thir: thir::Thir<'tcx>,
    pub root_expr: ExprId,
    pub(crate) synthetic_local_names: HashMap<hir::HirId, Symbol>,
    pub(crate) next_local_id: hir::ItemLocalId,
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

    pub fn synthetic_local_name(&self, id: hir::HirId) -> Option<Symbol> {
        self.synthetic_local_names.get(&id).copied()
    }

    pub fn insert_synthetic_local_name(&mut self, id: hir::HirId, name: Symbol) {
        self.synthetic_local_names.insert(id, name);
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
        temporaries::ValueToPlaceDesugarer::new(body).run();
    }
}
