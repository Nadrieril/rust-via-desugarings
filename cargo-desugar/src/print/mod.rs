use itertools::Itertools;
use std::{cell::RefCell, collections::HashMap};

use rustc_hir::{
    self as hir, ItemId, ItemKind,
    def::DefKind,
    def_id::{DefId, LocalDefId},
    intravisit::{self, Visitor},
};
use rustc_hir_pretty::{AnnNode, Nested, PpAnn, State};
use rustc_middle::ty::{
    self, AssocContainer, GenericArg, GenericArgKind, Ty, TyCtxt, TyKind,
    print::{PrettyPrinter, Print},
};
use rustc_span::FileName;
use std::marker::PhantomData;

use crate::desugar::{Body, desugar_thir};

mod thir;
use thir::{PrintedBody, ThirPrinter};
mod types;
use types::TypePrinter;

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

pub(crate) struct CratePrinter<'tcx> {
    tcx: TyCtxt<'tcx>,
    printed_bodies: RefCell<HashMap<LocalDefId, PrintedBody>>,
}

impl<'tcx> CratePrinter<'tcx> {
    fn new(tcx: TyCtxt<'tcx>) -> Self {
        Self {
            tcx,
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
        let mut body = Body::new(self.tcx, def_id, thir.steal(), root);
        desugar_thir(self.tcx, &mut body);
        let printed = ThirPrinter::new(self, &body).into_printed_body()?;
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
        let generics = self.tcx.generics_of(def_id);
        let own_args = generics.own_args_no_defaults(self.tcx, args);
        let own_args_vec = own_args.iter().copied().collect::<Vec<_>>();
        format!("{}{}", self.path(def_id), self.generic_args(&own_args_vec))
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
                let self_ty = container_args[0].as_type().unwrap();
                let self_ty = self.ty(self_ty);
                let trait_pred = self.path_with_args(container_def_id, container_args);
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
