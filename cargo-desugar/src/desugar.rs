use rustc_hir::def_id::LocalDefId;
use rustc_middle::{thir::Thir, ty::TyCtxt};

pub fn desugar_thir<'tcx>(_tcx: TyCtxt<'tcx>, _ldid: LocalDefId, _body: &mut Thir<'tcx>) {}
