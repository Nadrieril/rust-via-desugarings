use rustc_middle::{thir::Thir, ty::TyCtxt};

pub fn desugar_thir<'tcx>(_tcx: TyCtxt<'tcx>, _body: &mut Thir<'tcx>) {}
