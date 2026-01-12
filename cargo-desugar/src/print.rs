use itertools::Itertools;
use std::{
    cell::RefCell,
    collections::HashMap,
    fmt::{self, Write as _},
    ops::Deref,
    rc::Rc,
};

use rustc_ast::LitKind;
use rustc_hir::{
    self as hir, ItemId, ItemKind,
    def::{CtorKind, DefKind, Namespace},
    def_id::{CrateNum, DefId, LOCAL_CRATE, LocalDefId},
    definitions::{DefPathData, DefPathDataName, DisambiguatedDefPathData},
    intravisit::{self, Visitor},
    limit::Limit,
};
use rustc_hir_pretty::{AnnNode, Nested, PpAnn, State};
use rustc_middle::{
    mir::{AssignOp, BinOp, BorrowKind, FakeBorrowKind, UnOp},
    thir::{self, BlockSafety, PatKind, Thir},
    ty::{
        self, AssocContainer, GenericArg, GenericArgKind, Ty, TyCtxt, TyKind, TypeFoldable,
        TypeFolder, TypeSuperFoldable, VariantDef,
        print::{FmtPrinter, PrettyPrinter, Print, PrintError, PrintTraitRefExt, Printer},
    },
};
use rustc_span::{
    FileName,
    symbol::{Ident, Symbol, kw},
};
use std::marker::PhantomData;

use crate::desugar::{Body, desugar_thir};

/// Print the whole crate using the builtin HIR pretty-printer, but with bodies
/// replaced by our THIR-based rendering.
pub fn print_crate<'tcx>(tcx: TyCtxt<'tcx>) -> String {
    let mut root_mod = tcx.hir_root_module().clone();
    // Filter out some items.
    root_mod.item_ids = tcx
        .arena
        .alloc_from_iter(root_mod.item_ids.iter().copied().filter(|item_id| {
        let item = tcx.hir_item(*item_id);
        let attrs = tcx.hir_attrs(item.hir_id());
        let item_kind = &item.kind;
        // Remove the automatic `extern crate std` because it emits weird attrs.
        let is_extern_crate_std =
            matches!(item_kind, hir::ItemKind::ExternCrate(_, ident) if ident.as_str() == "std");
        // Remove the automatic prelude import because the attribute is unstable
        let is_prelude_import = attrs
            .iter()
            .any(|attr| attr.name().is_some_and(|s| s.as_str() == "prelude_import"));
        !(is_extern_crate_std || is_prelude_import)
    }));

    let ann = CratePrinter::new(tcx);
    let mut output = rustc_hir_pretty::print_crate(
        tcx.sess.source_map(),
        &root_mod,
        FileName::Custom("desugar".into()),
        String::new(),
        &|id| tcx.hir_attrs(id),
        &ann,
    );

    // Standard lib macros expand to unstable things.
    output.insert_str(
        0,
        "#![feature(
            allocator_api,
            fmt_arguments_from_str,
            fmt_internals,
            libstd_sys_internals,
            panic_internals,
            print_internals,
            rt,
            try_trait_v2,
        )]\n
        #![allow(
            unused_braces,
            unused_parens,
            internal_features,
        )]\n
        ",
    );

    match syn::parse_file(&output) {
        Ok(file) => prettyplease::unparse(&file),
        Err(_) => output,
    }
}

#[derive(Clone)]
struct PrintedBody {
    params: Vec<String>,
    body: String,
}

struct CratePrinter<'tcx> {
    tcx: TyCtxt<'tcx>,
    synthetic_local_names: Rc<RefCell<HashMap<hir::HirId, Symbol>>>,
    printed_bodies: RefCell<HashMap<LocalDefId, PrintedBody>>,
}

impl<'tcx> CratePrinter<'tcx> {
    fn new(tcx: TyCtxt<'tcx>) -> Self {
        Self {
            tcx,
            synthetic_local_names: Default::default(),
            printed_bodies: Default::default(),
        }
    }

    fn nested_fallback(&self, state: &mut State<'_>, nested: Nested) {
        let fallback: &dyn rustc_hir::intravisit::HirTyCtxt<'_> = &self.tcx;
        fallback.nested(state, nested);
    }

    fn print_visibility(&self, state: &mut State<'_>, def_id: LocalDefId) {
        let vis = self.tcx.visibility(def_id).map_id(|id| id.expect_local());
        let parent = self.tcx.parent_module_from_def_id(def_id).to_local_def_id();
        match vis {
            ty::Visibility::Public => state.word_nbsp("pub"),
            ty::Visibility::Restricted(module) => {
                if module != parent {
                    state.word_nbsp(vis.to_string(def_id, self.tcx))
                }
            }
        }
    }

    fn printed_body(&self, def_id: LocalDefId) -> Option<PrintedBody> {
        if let Some(body) = self.printed_bodies.borrow().get(&def_id) {
            return Some(body.clone());
        }

        let Ok((thir, root)) = self.tcx.thir_body(def_id) else {
            return None;
        };
        let mut body = Body::new(
            self.tcx,
            def_id,
            thir.steal(),
            root,
            self.synthetic_local_names.clone(),
        );
        desugar_thir(self.tcx, &mut body);
        let mut printer = ThirPrinter::new(self, &body);
        let params = body
            .thir
            .params
            .iter()
            .map(|param| {
                param
                    .pat
                    .as_deref()
                    .map(|p| printer.pat(p))
                    .unwrap_or("_".into())
            })
            .collect::<Vec<_>>();
        let body = printer.expr_in_block(body.root_expr);
        let printed = PrintedBody { params, body };
        self.printed_bodies
            .borrow_mut()
            .insert(def_id, printed.clone());
        Some(printed)
    }

    fn print_body(&self, def_id: LocalDefId) -> Option<(String, String)> {
        self.printed_body(def_id)
            .map(|body| (body.params.iter().format(", ").to_string(), body.body))
    }

    fn body_param_pat(&self, body_id: hir::BodyId, index: usize) -> Option<String> {
        let def_id = self.tcx.hir_body_owner_def_id(body_id);
        self.printed_body(def_id)
            .and_then(|body| body.params.get(index).cloned())
    }

    fn local_name(&self, id: thir::LocalVarId) -> String {
        let hir_id = id.0;
        let name = if let Some(name) = self.synthetic_local_names.borrow().get(&hir_id).copied() {
            name
        } else {
            self.tcx.hir_name(hir_id)
        };
        // Disambiguate names by their hir id, to avoid hygiene issues.
        if name == kw::SelfLower {
            name.to_string()
        } else {
            format!("{name}_{}", hir_id.local_id.as_u32())
        }
    }

    fn ty(&self, ty: Ty<'tcx>) -> String {
        let mut printer = TypePrinter::new(self);
        let _ = ty.print(&mut printer);
        printer.finish()
    }

    fn generic_args(&self, args: &[GenericArg<'tcx>]) -> String {
        if args.is_empty() {
            String::new()
        } else {
            let args = args.iter().map(|arg| self.generic_arg(*arg)).format(", ");
            format!("::<{args}>")
        }
    }

    fn generic_arg(&self, arg: GenericArg<'tcx>) -> String {
        match arg.kind() {
            GenericArgKind::Type(ty) => self.ty(ty),
            GenericArgKind::Lifetime(reg) => {
                let mut printer = TypePrinter::new(self);
                let _ = reg.print(&mut printer);
                printer.finish()
            }
            GenericArgKind::Const(ct) => {
                let mut printer = TypePrinter::new(self);
                let _ = ct.print(&mut printer);
                printer.finish()
            }
        }
    }

    fn path(&self, def_id: DefId) -> String {
        if let Some(local_def_id) = def_id.as_local()
            && self.local_parent_is_body(local_def_id)
        {
            self.tcx.item_name(local_def_id).to_string()
        } else {
            let mut printer = TypePrinter::new(self);
            match printer.try_print_visible_def_path(def_id) {
                Ok(true) => printer.finish(),
                _ => self.tcx.value_path_str_with_args(def_id, &[]),
            }
        }
    }

    fn path_with_args(&self, def_id: DefId, args: &'tcx [GenericArg<'tcx>]) -> String {
        if let Some(path) = self.associated_item_path(def_id, args) {
            return path;
        }
        format!("{}{}", self.path(def_id), self.generic_args(args))
    }

    fn local_parent_is_body(&self, def_id: LocalDefId) -> bool {
        let mut current = Some(def_id);
        while let Some(id) = current {
            if let Some(parent) = self.tcx.opt_local_parent(id) {
                if matches!(
                    self.tcx.def_kind(parent),
                    DefKind::Fn
                        | DefKind::AssocFn
                        | DefKind::Closure
                        | DefKind::InlineConst
                        | DefKind::Const
                        | DefKind::Static { .. }
                ) {
                    return true;
                }
                current = Some(parent);
            } else {
                break;
            }
        }
        false
    }

    fn associated_item_path(
        &self,
        def_id: DefId,
        args: &'tcx [GenericArg<'tcx>],
    ) -> Option<String> {
        let assoc = self.tcx.opt_associated_item(def_id)?;
        let container_def_id = self.tcx.parent(assoc.def_id);
        let generics = self.tcx.generics_of(def_id);
        let (container_args, assoc_args) = args.split_at(generics.parent_count);
        let assoc_args = self.generic_args(assoc_args);
        let container_path = match assoc.container {
            AssocContainer::Trait => {
                let [self_ty, trait_args @ ..] = container_args else {
                    unreachable!()
                };
                let self_ty = self_ty.as_type().unwrap();
                let self_ty = self.ty(self_ty);
                let trait_pred = self.path_with_args(container_def_id, trait_args);
                format!("<{self_ty} as {trait_pred}>",)
            }
            AssocContainer::InherentImpl => {
                let args_ref = self.tcx.mk_args_from_iter(container_args.iter().copied());
                let self_ty = self
                    .tcx
                    .type_of(container_def_id)
                    .instantiate(self.tcx, args_ref);
                match self_ty.kind() {
                    TyKind::Adt(adt_def, args) => self.path_with_args(adt_def.did(), args),
                    TyKind::Foreign(def_id) => self.path(*def_id),
                    _ => format!("<{}>", self.ty(self_ty)),
                }
            }
            AssocContainer::TraitImpl(_) => self.path_with_args(container_def_id, container_args),
        };

        let assoc_name = assoc.name();
        Some(format!("{container_path}::{assoc_name}{assoc_args}"))
    }
}

impl<'tcx> PpAnn for CratePrinter<'tcx> {
    fn pre(&self, state: &mut rustc_hir_pretty::State<'_>, node: AnnNode<'_>) {
        if let AnnNode::Item(item) = node
            && matches!(item.kind, ItemKind::Fn { .. })
        {
            self.print_visibility(state, item.owner_id.def_id)
        }
    }

    fn nested(&self, state: &mut rustc_hir_pretty::State<'_>, nested: Nested) {
        match nested {
            Nested::BodyParamPat(body_id, index)
                if let Some(param) = self.body_param_pat(body_id, index) =>
            {
                state.word(param);
            }
            Nested::Body(body_id)
                if let def_id = self.tcx.hir_body_owner_def_id(body_id)
                    && let Some((_, text)) = self.print_body(def_id) =>
            {
                // TODO: move this to a `print_body_with_inner_items` method
                let body = self.tcx.hir_body(body_id);
                let nested_items = {
                    struct NestedItemCollector<'tcx> {
                        found: Vec<Nested>,
                        _marker: PhantomData<&'tcx ()>,
                    }

                    impl<'tcx> NestedItemCollector<'tcx> {
                        fn new() -> Self {
                            Self {
                                found: Vec::new(),
                                _marker: PhantomData,
                            }
                        }
                    }

                    impl<'tcx> Visitor<'tcx> for NestedItemCollector<'tcx> {
                        type NestedFilter = intravisit::nested_filter::None;
                        fn visit_nested_item(&mut self, id: ItemId) {
                            self.found.push(Nested::Item(id));
                        }
                        fn visit_nested_trait_item(
                            &mut self,
                            id: rustc_hir::TraitItemId,
                        ) -> Self::Result {
                            self.found.push(Nested::TraitItem(id));
                        }
                        fn visit_nested_impl_item(
                            &mut self,
                            id: rustc_hir::ImplItemId,
                        ) -> Self::Result {
                            self.found.push(Nested::ImplItem(id));
                        }
                        fn visit_nested_foreign_item(
                            &mut self,
                            id: rustc_hir::ForeignItemId,
                        ) -> Self::Result {
                            self.found.push(Nested::ForeignItem(id));
                        }
                        fn visit_nested_body(&mut self, id: rustc_hir::BodyId) -> Self::Result {
                            self.found.push(Nested::Body(id));
                        }
                    }
                    let mut collector = NestedItemCollector::new();
                    intravisit::walk_body(&mut collector, body);
                    collector.found
                };
                if nested_items.is_empty() {
                    state.word(text);
                } else {
                    state.word("{");
                    for item in nested_items {
                        self.nested(state, item);
                    }
                    state.word(text);
                    state.word("}");
                }
            }
            _ => self.nested_fallback(state, nested),
        }
    }
}

struct TypePrinter<'a, 'tcx> {
    inner: FmtPrinter<'a, 'tcx>,
    crate_printer: &'a CratePrinter<'tcx>,
    tcx: TyCtxt<'tcx>,
    empty_path: bool,
    in_value: bool,
    printed_type_count: usize,
    type_length_limit: Limit,
}

impl<'tcx> Deref for TypePrinter<'_, 'tcx> {
    type Target = CratePrinter<'tcx>;
    fn deref(&self) -> &Self::Target {
        self.crate_printer
    }
}

impl<'a, 'tcx> TypePrinter<'a, 'tcx> {
    fn new(crate_printer: &'a CratePrinter<'tcx>) -> Self {
        let tcx = crate_printer.tcx;
        Self {
            inner: FmtPrinter::new(tcx, Namespace::TypeNS),
            tcx,
            crate_printer,
            empty_path: false,
            in_value: false,
            printed_type_count: 0,
            type_length_limit: tcx.type_length_limit(),
        }
    }

    fn finish(self) -> String {
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
        self.inner.print_type(ty)
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
        self.printed_type_count = 0;
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
        !self
            .type_length_limit
            .value_within_limit(self.printed_type_count)
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
}

struct ThirPrinter<'a, 'tcx> {
    crate_printer: &'a CratePrinter<'tcx>,
    body: &'a Body<'tcx>,
}

impl<'tcx> Deref for ThirPrinter<'_, 'tcx> {
    type Target = CratePrinter<'tcx>;
    fn deref(&self) -> &Self::Target {
        self.crate_printer
    }
}

impl<'a, 'tcx> ThirPrinter<'a, 'tcx> {
    fn new(crate_printer: &'a CratePrinter<'tcx>, body: &'a Body<'tcx>) -> Self {
        Self {
            crate_printer,
            body,
        }
    }

    fn thir(&self) -> &'a Thir<'tcx> {
        &self.body.thir
    }

    fn expr(&mut self, id: thir::ExprId) -> String {
        let expr = &self.thir().exprs[id];
        match &expr.kind {
            thir::ExprKind::Scope { value, .. } => self.expr(*value),
            thir::ExprKind::Box { value } => format!("Box::new({})", self.expr(*value)),
            thir::ExprKind::If {
                cond,
                then,
                else_opt,
                ..
            } => {
                let mut s = format!("if {} {}", self.expr(*cond), self.expr_in_block(*then));
                if let Some(e) = else_opt {
                    s.push_str(" else ");
                    s.push_str(&self.expr_in_block(*e));
                }
                s
            }
            thir::ExprKind::Call { fun, args, .. } => {
                let fun = self.expr(*fun);
                let args = args.iter().map(|arg| self.expr(*arg)).format(", ");
                format!("{}({})", fun, args)
            }
            thir::ExprKind::ByUse { expr, .. } => {
                format!("{}.use", self.expr(*expr))
            }
            thir::ExprKind::Deref { arg } => format!("*{}", self.expr(*arg)),
            thir::ExprKind::Binary { op, lhs, rhs } => {
                let op_str = self.bin_op(*op);
                format!("{} {} {}", self.expr(*lhs), op_str, self.expr(*rhs))
            }
            thir::ExprKind::LogicalOp { op, lhs, rhs } => {
                let op_str = match op {
                    thir::LogicalOp::And => "&&",
                    thir::LogicalOp::Or => "||",
                };
                format!("{} {} {}", self.expr(*lhs), op_str, self.expr(*rhs))
            }
            thir::ExprKind::Unary { op, arg } => {
                let op_str = self.un_op(*op);
                format!("{}{}", op_str, self.expr(*arg))
            }
            thir::ExprKind::Cast { source } => {
                format!("({} as {})", self.expr(*source), self.ty(expr.ty))
            }
            thir::ExprKind::Use { source }
            | thir::ExprKind::NeverToAny { source }
            | thir::ExprKind::PlaceUnwrapUnsafeBinder { source }
            | thir::ExprKind::ValueUnwrapUnsafeBinder { source }
            | thir::ExprKind::WrapUnsafeBinder { source } => self.expr(*source),
            thir::ExprKind::PointerCoercion { source, .. } => {
                format!("({} as {})", self.expr(*source), self.ty(expr.ty))
            }
            thir::ExprKind::Loop { body } => format!("loop {}", self.expr_in_block(*body)),
            thir::ExprKind::LoopMatch { match_data, .. } => self.expr(match_data.scrutinee),
            thir::ExprKind::Let { expr, pat } => {
                format!("let {} = {}", self.pat(pat), self.expr(*expr))
            }
            thir::ExprKind::Match {
                scrutinee, arms, ..
            } => {
                let mut s = format!("match ({}) {{\n", self.expr(*scrutinee));
                for arm_id in arms.iter() {
                    s.push_str(&self.arm(*arm_id));
                }
                s.push('}');
                s
            }
            thir::ExprKind::Block { block } => self.block(*block),
            thir::ExprKind::Assign { lhs, rhs } => {
                format!("{} = {}", self.expr(*lhs), self.expr(*rhs))
            }
            thir::ExprKind::AssignOp { op, lhs, rhs } => {
                let op_str = self.assign_op(*op);
                format!("{} {}= {}", self.expr(*lhs), op_str, self.expr(*rhs))
            }
            thir::ExprKind::Field { lhs, name, .. } => {
                format!("{}.{}", self.expr(*lhs), name.as_usize())
            }
            thir::ExprKind::Index { lhs, index } => {
                format!("{}[{}]", self.expr(*lhs), self.expr(*index))
            }
            thir::ExprKind::VarRef { id } => self.local_name(*id),
            thir::ExprKind::UpvarRef { var_hir_id, .. } => self.local_name(*var_hir_id),
            thir::ExprKind::Borrow { borrow_kind, arg } => {
                let prefix = match borrow_kind {
                    BorrowKind::Shared
                    | BorrowKind::Fake(FakeBorrowKind::Shallow | FakeBorrowKind::Deep) => "&",
                    BorrowKind::Mut { .. } => "&mut ",
                };
                format!("{}{}", prefix, self.expr(*arg))
            }
            thir::ExprKind::RawBorrow { mutability, arg } => {
                let prefix = match mutability {
                    hir::Mutability::Mut => "raw mut",
                    hir::Mutability::Not => "raw const",
                };
                format!("&{} {}", prefix, self.expr(*arg))
            }
            thir::ExprKind::Break { value, .. } => match value {
                Some(v) => format!("break {}", self.expr(*v)),
                None => "break".to_string(),
            },
            thir::ExprKind::Continue { .. } => "continue".to_string(),
            thir::ExprKind::ConstContinue { value, .. } => {
                format!("const_continue {}", self.expr(*value))
            }
            thir::ExprKind::Return { value } => match value {
                Some(v) => format!("return {}", self.expr(*v)),
                None => "return".to_string(),
            },
            thir::ExprKind::Become { value } => format!("become {}", self.expr(*value)),
            thir::ExprKind::ConstBlock { .. } => "const { /* ... */ }".to_string(),
            thir::ExprKind::Repeat { value, count } => {
                format!("[{}; {}]", self.expr(*value), count)
            }
            thir::ExprKind::Array { fields } => {
                let items = fields.iter().map(|f| self.expr(*f)).format(", ");
                format!("[{}]", items)
            }
            thir::ExprKind::Tuple { fields } => {
                let items = fields
                    .iter()
                    .map(|f| self.expr(*f))
                    .map(|x| format!("{x},"))
                    .format("");
                format!("({})", items)
            }
            thir::ExprKind::Adt(adt) => {
                let adt_name = self.path_with_args(adt.adt_def.did(), adt.args);
                let variant = adt.adt_def.variant(adt.variant_index);
                let variant_name = if adt.adt_def.is_enum() {
                    format!("::{}", variant.name)
                } else {
                    format!("")
                };
                let fields = if adt.fields.is_empty() {
                    format!("{{}}")
                } else {
                    let base = if let thir::AdtExprBase::Base(base) = &adt.base {
                        Some(format!("..{}", self.expr(base.base)))
                    } else {
                        None
                    };
                    let parts = adt
                        .fields
                        .iter()
                        .map(|field_expr| {
                            let field = &variant.fields[field_expr.name];
                            format!("{}: {}", field.name, self.expr(field_expr.expr))
                        })
                        .chain(base)
                        .format(", ");
                    format!("{{ {} }}", parts)
                };
                format!("{adt_name}{variant_name}{fields}")
            }
            thir::ExprKind::PlaceTypeAscription { source, .. }
            | thir::ExprKind::ValueTypeAscription { source, .. } => self.expr(*source),
            thir::ExprKind::Closure(closure) => {
                match self.crate_printer.print_body(closure.closure_id) {
                    Some((params, body)) => format!("|{params}| {body}"),
                    None => format!("todo!(\"missing body for closure\")"),
                }
            }
            thir::ExprKind::Literal { lit, neg } => {
                let lit_str = self.literal(lit);
                if *neg { format!("-{lit_str}") } else { lit_str }
            }
            thir::ExprKind::NonHirLiteral { lit, .. } => format!("{lit}"),
            thir::ExprKind::ZstLiteral { .. } => match expr.ty.kind() {
                &TyKind::FnDef(def_id, args) => {
                    if let Some(path) = self.associated_item_path(def_id, args.as_slice()) {
                        path
                    } else {
                        self.path_with_args(def_id, args)
                    }
                }
                _ => "()".to_string(),
            },
            thir::ExprKind::NamedConst { def_id, args, .. } => self.path_with_args(*def_id, *args),
            thir::ExprKind::ConstParam { param, .. } => param.name.to_string(),
            thir::ExprKind::StaticRef { def_id, .. } => {
                let borrow = match expr.ty.kind() {
                    TyKind::Ref(_, _, mutability) => format!("{}", mutability.ref_prefix_str()),
                    TyKind::RawPtr(_, mutability) => format!("&raw {}", mutability.ptr_str()),
                    _ => unreachable!(),
                };
                let path = self.path(*def_id);
                format!("{borrow} {path}")
            }
            thir::ExprKind::InlineAsm(_) => "asm!(...)".to_string(),
            thir::ExprKind::ThreadLocalRef(def_id) => {
                format!("thread_local!({})", self.path(*def_id))
            }
            thir::ExprKind::Yield { value } => format!("yield {}", self.expr(*value)),
        }
    }

    fn expr_in_block(&mut self, id: thir::ExprId) -> String {
        match self.thir().exprs[id].kind {
            thir::ExprKind::Block { block } => self.block(block),
            thir::ExprKind::Scope { value, .. } => self.expr_in_block(value),
            thir::ExprKind::Use { source }
            | thir::ExprKind::NeverToAny { source }
            | thir::ExprKind::PlaceUnwrapUnsafeBinder { source }
            | thir::ExprKind::ValueUnwrapUnsafeBinder { source }
            | thir::ExprKind::WrapUnsafeBinder { source }
            | thir::ExprKind::PlaceTypeAscription { source, .. }
            | thir::ExprKind::ValueTypeAscription { source, .. } => self.expr_in_block(source),
            _ => format!("{{ {} }}", self.expr(id)),
        }
    }

    fn block(&mut self, id: thir::BlockId) -> String {
        let block = &self.thir().blocks[id];
        let mut s = String::new();
        if !matches!(block.safety_mode, BlockSafety::Safe) {
            s.push_str("unsafe ");
        }
        s.push_str("{\n");
        for stmt_id in block.stmts.iter() {
            s.push_str(&self.stmt(*stmt_id));
        }
        if let Some(expr) = block.expr {
            s.push_str(&self.expr(expr));
            s.push_str("\n");
        }
        s.push_str("}");
        s
    }

    fn stmt(&mut self, id: thir::StmtId) -> String {
        let stmt = &self.thir().stmts[id];
        match &stmt.kind {
            thir::StmtKind::Expr { expr, .. } => {
                format!("{};\n", self.expr(*expr))
            }
            thir::StmtKind::Let {
                pattern,
                initializer,
                else_block,
                ..
            } => {
                let mut s = format!("let {}", self.pat(pattern));
                if let Some(init) = initializer {
                    s.push_str(&format!(" = {}", self.expr(*init)));
                }
                if let Some(else_blk) = else_block {
                    s.push_str(" else ");
                    s.push_str(&self.block(*else_blk));
                }
                s.push_str(";\n");
                s
            }
        }
    }

    fn arm(&mut self, id: thir::ArmId) -> String {
        let arm = &self.thir().arms[id];
        let mut s = format!("{} ", self.pat(&arm.pattern));
        if let Some(guard) = arm.guard {
            s.push_str(&format!("if {} ", self.expr(guard)));
        }
        s.push_str("=> ");
        s.push_str(&self.expr(arm.body));
        s.push_str(",\n");
        s
    }

    fn pat(&self, pat: &thir::Pat<'tcx>) -> String {
        match &pat.kind {
            PatKind::Missing | PatKind::Wild => "_".to_string(),
            PatKind::AscribeUserType { subpattern, .. } => self.pat(subpattern),
            PatKind::Binding {
                var,
                mode,
                subpattern,
                ..
            } => {
                let mut s = String::new();
                s.push_str(mode.prefix_str());
                s.push_str(&self.local_name(*var));
                if let Some(sub) = subpattern {
                    s.push_str(" @ ");
                    s.push_str(&self.pat(sub));
                }
                s
            }
            PatKind::Variant {
                adt_def,
                args,
                variant_index,
                subpatterns,
                ..
            } => {
                let variant = &adt_def.variant(*variant_index);
                let path = self.path_with_args(adt_def.did(), args);
                let path = format!("{}::{}", path, variant.name);
                self.adt_pat(path, variant, subpatterns)
            }
            PatKind::Leaf { subpatterns } => match pat.ty.kind() {
                TyKind::Tuple(_) => self.tuple_pat(None, subpatterns),
                TyKind::Adt(adt_def, args) => {
                    assert!(!adt_def.is_enum());
                    let variant = adt_def.non_enum_variant();
                    let path = self.path_with_args(adt_def.did(), args);
                    self.adt_pat(path, variant, subpatterns)
                }
                _ => "_".to_owned(),
            },
            PatKind::Deref { subpattern } => format!("&{}", self.pat(subpattern)),
            PatKind::DerefPattern { subpattern, borrow } => {
                let prefix = match borrow {
                    hir::ByRef::Yes(_, hir::Mutability::Mut) => "&mut ",
                    hir::ByRef::Yes(..) => "&",
                    hir::ByRef::No => "",
                };
                format!("{prefix}{}", self.pat(subpattern))
            }
            PatKind::Constant { value } => format!("{}", value),
            PatKind::ExpandedConstant { subpattern, .. } => self.pat(subpattern),
            PatKind::Range(range) => format!("{:?}..{:?}", range.lo, range.hi),
            PatKind::Slice {
                prefix,
                slice,
                suffix,
            }
            | PatKind::Array {
                prefix,
                slice,
                suffix,
            } => {
                let mut parts = Vec::new();
                parts.extend(prefix.iter().map(|p| self.pat(p)));
                if let Some(mid) = slice {
                    parts.push(format!("..{}", self.pat(mid)));
                }
                parts.extend(suffix.iter().map(|p| self.pat(p)));
                format!("[{}]", parts.join(", "))
            }
            PatKind::Or { pats } => pats.iter().map(|p| self.pat(p)).format(" | ").to_string(),
            PatKind::Never => "!".to_string(),
            PatKind::Error(_) => "<pat error>".to_string(),
        }
    }

    fn tuple_pat(&self, path: Option<String>, subpatterns: &[thir::FieldPat<'tcx>]) -> String {
        let mut fields = subpatterns.to_vec();
        fields.sort_by_key(|fp| fp.field.as_usize());
        let inner = fields
            .iter()
            .map(|ipat| self.pat(&ipat.pattern))
            .map(|p| format!("{p},"))
            .format("");
        match path {
            Some(p) => format!("{p}({inner})"),
            None => format!("({inner})"),
        }
    }

    fn adt_pat(
        &self,
        path: String,
        variant: &VariantDef,
        subpatterns: &[thir::FieldPat<'tcx>],
    ) -> String {
        match variant.ctor_kind() {
            Some(CtorKind::Const) => {
                assert_eq!(subpatterns.len(), 0);
                path
            }
            Some(CtorKind::Fn) | None => {
                let ellipsis = if subpatterns.len() != variant.fields.len() {
                    Some("..".to_string())
                } else {
                    None
                };
                let fields = subpatterns
                    .iter()
                    .map(|ipat| {
                        let field_def = &variant.fields[ipat.field];
                        let pat = self.pat(&ipat.pattern);
                        format!("{}: {}", field_def.name, pat)
                    })
                    .chain(ellipsis)
                    .format(", ");
                format!("{path} {{ {fields} }}")
            }
        }
    }

    fn literal(&self, lit: &hir::Lit) -> String {
        match lit.node {
            LitKind::Str(sym, _) => format!("{:?}", sym.as_str()),
            LitKind::ByteStr(ref bytes, _) => format!("&{:?}", bytes),
            LitKind::CStr(_, _) => "c\"...\"".to_string(),
            LitKind::Byte(b) => format!("{b}u8"),
            LitKind::Char(c) => format!("{:?}", c),
            LitKind::Int(n, _) => format!("{n}"),
            LitKind::Float(sym, _) => sym.to_string(),
            LitKind::Bool(b) => b.to_string(),
            LitKind::Err(_) => "<lit error>".to_string(),
        }
    }

    fn bin_op(&self, op: BinOp) -> &'static str {
        match op {
            BinOp::Add => "+",
            BinOp::Sub => "-",
            BinOp::Mul => "*",
            BinOp::Div => "/",
            BinOp::Rem => "%",
            BinOp::BitXor => "^",
            BinOp::BitAnd => "&",
            BinOp::BitOr => "|",
            BinOp::Shl => "<<",
            BinOp::Shr => ">>",
            BinOp::Eq => "==",
            BinOp::Lt => "<",
            BinOp::Le => "<=",
            BinOp::Ne => "!=",
            BinOp::Ge => ">=",
            BinOp::Gt => ">",
            BinOp::Offset => ".offset",
            BinOp::AddUnchecked => "+ (unchecked)",
            BinOp::AddWithOverflow => "+ (overflowing)",
            BinOp::SubUnchecked => "- (unchecked)",
            BinOp::SubWithOverflow => "- (overflowing)",
            BinOp::MulUnchecked => "* (unchecked)",
            BinOp::MulWithOverflow => "* (overflowing)",
            BinOp::ShlUnchecked => "<< (unchecked)",
            BinOp::ShrUnchecked => ">> (unchecked)",
            BinOp::Cmp => "<=>",
        }
    }

    fn assign_op(&self, op: AssignOp) -> &'static str {
        match op {
            AssignOp::AddAssign => "+",
            AssignOp::SubAssign => "-",
            AssignOp::MulAssign => "*",
            AssignOp::DivAssign => "/",
            AssignOp::RemAssign => "%",
            AssignOp::BitXorAssign => "^",
            AssignOp::BitAndAssign => "&",
            AssignOp::BitOrAssign => "|",
            AssignOp::ShlAssign => "<<",
            AssignOp::ShrAssign => ">>",
        }
    }

    fn un_op(&self, op: UnOp) -> &'static str {
        match op {
            UnOp::Not => "!",
            UnOp::Neg => "-",
            UnOp::PtrMetadata => "ptr_metadata ",
        }
    }
}
