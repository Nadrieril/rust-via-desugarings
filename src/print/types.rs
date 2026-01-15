//! Pretty-printer for types, heavily copied from rustc's `FmtPrinter` but modified to print
//! correct paths for nested items and probably more things in the future.
use std::fmt::{self, Write as _};

use rustc_hir::{
    def::Namespace,
    def_id::{CrateNum, DefId, LOCAL_CRATE},
    definitions::{DefPathData, DefPathDataName, DisambiguatedDefPathData},
};
use rustc_middle::ty::{
    self, GenericArg, Ty, TyCtxt, TyKind, TypeFoldable, TypeFolder, TypeSuperFoldable,
    print::{
        FmtPrinter, PrettyPrinter, Print, PrintError, PrintTraitRefExt, Printer,
        with_reduced_queries,
    },
};
use rustc_span::symbol::{Ident, kw};

use super::CratePrinter;

pub(super) struct TypePrinter<'a, 'tcx> {
    inner: FmtPrinter<'a, 'tcx>,
    crate_printer: &'a CratePrinter<'tcx>,
    tcx: TyCtxt<'tcx>,
    empty_path: bool,
    in_value: bool,
}

impl<'tcx> std::ops::Deref for TypePrinter<'_, 'tcx> {
    type Target = CratePrinter<'tcx>;
    fn deref(&self) -> &Self::Target {
        self.crate_printer
    }
}

impl<'a, 'tcx> TypePrinter<'a, 'tcx> {
    pub(super) fn new(crate_printer: &'a CratePrinter<'tcx>) -> Self {
        let tcx = crate_printer.tcx;
        Self {
            inner: FmtPrinter::new(tcx, Namespace::TypeNS),
            tcx,
            crate_printer,
            empty_path: false,
            in_value: false,
        }
    }

    pub(super) fn finish(self) -> String {
        self.inner.into_buffer()
    }
}

impl fmt::Write for TypePrinter<'_, '_> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.inner.write_str(s)
    }
}

impl<'tcx> Printer<'tcx> for TypePrinter<'_, 'tcx> {
    fn tcx<'a>(&'a self) -> TyCtxt<'tcx> {
        self.tcx
    }

    fn print_def_path(
        &mut self,
        def_id: DefId,
        args: &'tcx [GenericArg<'tcx>],
    ) -> Result<(), PrintError> {
        self.write_str(&self.crate_printer.path_with_args(def_id, args))
    }

    fn print_region(&mut self, region: ty::Region<'tcx>) -> Result<(), PrintError> {
        self.inner.print_region(region)
    }

    fn print_type(&mut self, ty: Ty<'tcx>) -> Result<(), PrintError> {
        let placeholder = Ty::new_param(self.tcx, 0, kw::Underscore);
        struct ScrubUnnameable<'tcx> {
            tcx: TyCtxt<'tcx>,
            placeholder: Ty<'tcx>,
        }

        impl<'tcx> TypeFolder<TyCtxt<'tcx>> for ScrubUnnameable<'tcx> {
            fn cx(&self) -> TyCtxt<'tcx> {
                self.tcx
            }

            fn fold_ty(&mut self, ty: Ty<'tcx>) -> Ty<'tcx> {
                match ty.kind() {
                    TyKind::Closure(..)
                    | TyKind::CoroutineClosure(..)
                    | TyKind::Coroutine(..)
                    | TyKind::CoroutineWitness(..) => self.placeholder,
                    _ => ty.super_fold_with(self),
                }
            }
        }

        let mut scrubber = ScrubUnnameable {
            tcx: self.tcx,
            placeholder,
        };
        let ty = ty.fold_with(&mut scrubber);

        match ty.kind() {
            TyKind::Adt(def, args) => self.print_def_path(def.did(), args),
            TyKind::Foreign(def_id) => self.print_def_path(*def_id, &[]),
            TyKind::RawPtr(inner, mutbl) => {
                write!(self, "*{} ", mutbl.ptr_str())?;
                inner.print(self)
            }
            TyKind::Ref(region, inner, mutbl) => {
                write!(self, "&")?;
                if self.should_print_optional_region(*region) {
                    region.print(self)?;
                    write!(self, " ")?;
                }
                ty::TypeAndMut {
                    ty: *inner,
                    mutbl: *mutbl,
                }
                .print(self)
            }
            TyKind::Tuple(tys) => {
                write!(self, "(")?;
                for (index, ty) in tys.iter().enumerate() {
                    if index > 0 {
                        write!(self, ", ")?;
                    }
                    ty.print(self)?;
                }
                if tys.len() == 1 {
                    write!(self, ",")?;
                }
                write!(self, ")")?;
                Ok(())
            }
            TyKind::Array(inner, len) => {
                write!(self, "[")?;
                inner.print(self)?;
                write!(self, "; ")?;
                len.print(self)?;
                write!(self, "]")?;
                Ok(())
            }
            TyKind::Slice(inner) => {
                write!(self, "[")?;
                inner.print(self)?;
                write!(self, "]")?;
                Ok(())
            }
            TyKind::FnPtr(sig, header) => sig.with(*header).print(self),
            TyKind::FnDef(def_id, args) => {
                if with_reduced_queries() {
                    self.print_def_path(*def_id, args)
                } else {
                    write!(self, "_")?;
                    Ok(())
                }
            }
            TyKind::Dynamic(data, region) => {
                let print_r = self.should_print_optional_region(*region);
                write!(self, "(")?;
                write!(self, "dyn ")?;
                data.print(self)?;
                if print_r {
                    write!(self, " + ")?;
                    region.print(self)?;
                }
                write!(self, ")")?;
                Ok(())
            }
            TyKind::Alias(_, data) => data.print(self),
            _ => self.inner.print_type(ty),
        }
    }

    fn print_dyn_existential(
        &mut self,
        predicates: &'tcx ty::List<ty::PolyExistentialPredicate<'tcx>>,
    ) -> Result<(), PrintError> {
        self.inner.pretty_print_dyn_existential(predicates)
    }

    fn print_const(&mut self, ct: ty::Const<'tcx>) -> Result<(), PrintError> {
        self.inner.pretty_print_const(ct, false)
    }

    fn print_crate_name(&mut self, cnum: CrateNum) -> Result<(), PrintError> {
        self.empty_path = true;
        if cnum == LOCAL_CRATE && !rustc_middle::ty::print::with_resolve_crate_name() {
            if self.tcx.sess.at_least_rust_2018() && rustc_middle::ty::print::with_crate_prefix() {
                write!(self, "{}", kw::Crate)?;
                self.empty_path = false;
            }
        } else {
            write!(self, "{}", self.tcx.crate_name(cnum))?;
            self.empty_path = false;
        }
        Ok(())
    }

    fn print_path_with_simple(
        &mut self,
        print_prefix: impl FnOnce(&mut Self) -> Result<(), PrintError>,
        disambiguated_data: &DisambiguatedDefPathData,
    ) -> Result<(), PrintError> {
        print_prefix(self)?;
        if let DefPathData::ForeignMod | DefPathData::Ctor = disambiguated_data.data {
            return Ok(());
        }

        if !self.empty_path {
            write!(self, "::")?;
        }

        if let DefPathDataName::Named(name) = disambiguated_data.data.name() {
            if Ident::with_dummy_span(name).is_raw_guess() {
                write!(self, "r#")?;
            }
        }

        let verbose = self.tcx.sess.verbose_internals();
        write!(self, "{}", disambiguated_data.as_sym(verbose))?;

        self.empty_path = false;
        Ok(())
    }

    fn print_path_with_impl(
        &mut self,
        print_prefix: impl FnOnce(&mut Self) -> Result<(), PrintError>,
        self_ty: Ty<'tcx>,
        trait_ref: Option<ty::TraitRef<'tcx>>,
    ) -> Result<(), PrintError> {
        print_prefix(self)?;

        self.generic_delimiters(|p| {
            write!(p, "impl ")?;
            if let Some(trait_ref) = trait_ref {
                trait_ref.print_only_trait_path().print(p)?;
                write!(p, " for ")?;
            }
            self_ty.print(p)?;

            Ok(())
        })?;
        self.empty_path = false;
        Ok(())
    }

    fn print_path_with_generic_args(
        &mut self,
        print_prefix: impl FnOnce(&mut Self) -> Result<(), PrintError>,
        args: &[GenericArg<'tcx>],
    ) -> Result<(), PrintError> {
        print_prefix(self)?;

        if !args.is_empty() {
            if self.in_value {
                write!(self, "::")?;
            }
            self.generic_delimiters(|p| p.comma_sep(args.iter().copied()))?;
        }
        self.empty_path = false;
        Ok(())
    }

    fn print_path_with_qualified(
        &mut self,
        self_ty: Ty<'tcx>,
        trait_ref: Option<ty::TraitRef<'tcx>>,
    ) -> Result<(), PrintError> {
        if trait_ref.is_none() {
            match self_ty.kind() {
                TyKind::Adt(..)
                | TyKind::Foreign(_)
                | TyKind::Bool
                | TyKind::Char
                | TyKind::Str
                | TyKind::Int(_)
                | TyKind::Uint(_)
                | TyKind::Float(_) => {
                    return self_ty.print(self);
                }
                _ => {}
            }
        }

        self.generic_delimiters(|p| {
            self_ty.print(p)?;
            if let Some(trait_ref) = trait_ref {
                write!(p, " as ")?;
                trait_ref.print_only_trait_path().print(p)?;
            }
            Ok(())
        })?;
        self.empty_path = false;
        Ok(())
    }
}

impl<'tcx> PrettyPrinter<'tcx> for TypePrinter<'_, 'tcx> {
    fn ty_infer_name(&self, id: ty::TyVid) -> Option<rustc_span::Symbol> {
        self.inner.ty_infer_name(id)
    }

    fn reset_type_limit(&mut self) {
        self.inner.reset_type_limit();
    }

    fn const_infer_name(&self, id: ty::ConstVid) -> Option<rustc_span::Symbol> {
        self.inner.const_infer_name(id)
    }

    fn generic_delimiters(
        &mut self,
        f: impl FnOnce(&mut Self) -> Result<(), PrintError>,
    ) -> Result<(), PrintError> {
        write!(self, "<")?;

        let was_in_value = std::mem::replace(&mut self.in_value, false);
        f(self)?;
        self.in_value = was_in_value;

        write!(self, ">")?;
        Ok(())
    }

    fn should_truncate(&mut self) -> bool {
        false
    }

    fn should_print_optional_region(&self, region: ty::Region<'tcx>) -> bool {
        self.inner.should_print_optional_region(region)
    }

    fn pretty_print_const_pointer<Prov: rustc_middle::mir::interpret::Provenance>(
        &mut self,
        _p: rustc_middle::mir::interpret::Pointer<Prov>,
        ty: Ty<'tcx>,
    ) -> Result<(), PrintError> {
        let print = |this: &mut Self| {
            write!(this, "&_")?;
            Ok(())
        };
        self.typed_value(print, |this| this.print_type(ty), ": ")
    }

    fn pretty_print_in_binder<T>(&mut self, value: &ty::Binder<'tcx, T>) -> Result<(), PrintError>
    where
        T: Print<'tcx, Self> + TypeFoldable<TyCtxt<'tcx>>,
    {
        // TODO: print bound variables correctly
        value.as_ref().skip_binder().print(self)
    }
}
