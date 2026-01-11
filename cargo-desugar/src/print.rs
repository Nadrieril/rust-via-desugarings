use itertools::Itertools;
use std::fmt::{self, Display, Write as _};

use rustc_ast::LitKind;
use rustc_hir::{
    self as hir, ItemId,
    def::Namespace,
    def_id::{CrateNum, DefId, LOCAL_CRATE, LocalDefId},
    definitions::{DefPathData, DefPathDataName, DisambiguatedDefPathData},
    intravisit::{self, Visitor},
    limit::Limit,
};
use rustc_hir_pretty::{Nested, PpAnn};
use rustc_middle::{
    mir::{AssignOp, BinOp, BorrowKind, FakeBorrowKind, UnOp},
    thir::{self, BlockSafety, Pat, PatKind, Thir},
    ty::{
        self, AssocContainer, GenericArg, GenericArgKind, Ty, TyCtxt, TyKind, TypeFoldable,
        TypeFolder, TypeSuperFoldable,
        print::{FmtPrinter, PrettyPrinter, Print, PrintError, PrintTraitRefExt, Printer},
    },
};
use rustc_span::{
    FileName,
    symbol::{Ident, kw},
};
use std::marker::PhantomData;

use crate::desugar::{Body, desugar_thir};

/// Print the whole crate using the builtin HIR pretty-printer, but with bodies
/// replaced by our THIR-based rendering.
pub fn print_crate<'tcx>(tcx: TyCtxt<'tcx>) -> String {
    let ann = DesugaredBodyPrettyPrinter { tcx };
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
        "#![feature(fmt_arguments_from_str, print_internals, try_trait_v2)]\n",
    );
    output
}

struct DesugaredBodyPrettyPrinter<'tcx> {
    tcx: TyCtxt<'tcx>,
}

impl<'tcx> DesugaredBodyPrettyPrinter<'tcx> {
    fn print_body(&self, def_id: LocalDefId) -> Option<(String, String)> {
        let Ok((thir, root)) = self.tcx.thir_body(def_id) else {
            return None;
        };
        let mut body = Body::new(self.tcx, def_id, thir.steal(), root);
        desugar_thir(self.tcx, &mut body);
        let mut printer = ThirPrinter::new(self.tcx, &body);
        let expr = printer.expr_in_block(body.root_expr);
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
            .format(", ")
            .to_string();
        Some((params, expr))
    }
    fn nested_fallback(&self, state: &mut rustc_hir_pretty::State<'_>, nested: Nested) {
        let fallback: &dyn rustc_hir::intravisit::HirTyCtxt<'_> = &self.tcx;
        fallback.nested(state, nested);
    }
}

impl<'tcx> PpAnn for DesugaredBodyPrettyPrinter<'tcx> {
    fn nested(&self, state: &mut rustc_hir_pretty::State<'_>, nested: Nested) {
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
            fn visit_nested_trait_item(&mut self, id: rustc_hir::TraitItemId) -> Self::Result {
                self.found.push(Nested::TraitItem(id));
            }
            fn visit_nested_impl_item(&mut self, id: rustc_hir::ImplItemId) -> Self::Result {
                self.found.push(Nested::ImplItem(id));
            }
            fn visit_nested_foreign_item(&mut self, id: rustc_hir::ForeignItemId) -> Self::Result {
                self.found.push(Nested::ForeignItem(id));
            }
            fn visit_nested_body(&mut self, id: rustc_hir::BodyId) -> Self::Result {
                self.found.push(Nested::Body(id));
            }
        }

        match nested {
            Nested::Body(body_id)
                if let def_id = self.tcx.hir_body_owner_def_id(body_id)
                    && let Some((_, text)) = self.print_body(def_id) =>
            {
                let body = self.tcx.hir_body(body_id);
                let nested_items = {
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

struct TypePrinter<'tcx> {
    inner: FmtPrinter<'tcx, 'tcx>,
    tcx: TyCtxt<'tcx>,
    empty_path: bool,
    in_value: bool,
    printed_type_count: usize,
    type_length_limit: Limit,
}

impl<'tcx> TypePrinter<'tcx> {
    fn new(tcx: TyCtxt<'tcx>) -> Self {
        Self {
            inner: FmtPrinter::new(tcx, Namespace::TypeNS),
            tcx,
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

impl fmt::Write for TypePrinter<'_> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.inner.write_str(s)
    }
}

impl<'tcx> Printer<'tcx> for TypePrinter<'tcx> {
    fn tcx<'a>(&'a self) -> TyCtxt<'tcx> {
        self.tcx
    }

    fn print_def_path(
        &mut self,
        def_id: DefId,
        args: &'tcx [GenericArg<'tcx>],
    ) -> Result<(), PrintError> {
        if args.is_empty() {
            if self.inner.try_print_trimmed_def_path(def_id)? {
                self.empty_path = false;
                return Ok(());
            }

            if self.inner.try_print_visible_def_path(def_id)? {
                self.empty_path = false;
                return Ok(());
            }
        }

        self.default_print_def_path(def_id, args)
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

impl<'tcx> PrettyPrinter<'tcx> for TypePrinter<'tcx> {
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

struct ThirPrinter<'tcx, 'a> {
    tcx: TyCtxt<'tcx>,
    body: &'a Body<'tcx>,
}

impl<'tcx, 'a> ThirPrinter<'tcx, 'a> {
    fn new(tcx: TyCtxt<'tcx>, body: &'a Body<'tcx>) -> Self {
        Self { tcx, body }
    }

    fn thir(&self) -> &'a Thir<'tcx> {
        &self.body.thir
    }

    fn print_ty(&self, ty: Ty<'tcx>) -> String {
        let mut printer = TypePrinter::new(self.tcx);
        let _ = ty.print(&mut printer);
        printer.finish()
    }

    fn format_generic_args(&self, args: &[GenericArg<'tcx>]) -> impl Display {
        if args.is_empty() {
            String::new()
        } else {
            let args = args.iter().map(|arg| self.generic_arg(*arg)).format(", ");
            format!("::<{args}>")
        }
    }

    fn generic_arg(&self, arg: GenericArg<'tcx>) -> String {
        match arg.kind() {
            GenericArgKind::Type(ty) => self.print_ty(ty),
            GenericArgKind::Lifetime(reg) => {
                let mut printer = TypePrinter::new(self.tcx);
                let _ = reg.print(&mut printer);
                printer.finish()
            }
            GenericArgKind::Const(ct) => {
                let mut printer = TypePrinter::new(self.tcx);
                let _ = ct.print(&mut printer);
                printer.finish()
            }
        }
    }

    fn path_with_args(&self, def_id: DefId, args: &'tcx [GenericArg<'tcx>]) -> String {
        let mut printer = TypePrinter::new(self.tcx);
        match printer.try_print_visible_def_path(def_id) {
            Ok(true) => format!("{}{}", printer.finish(), self.format_generic_args(args)),
            _ => self.tcx.value_path_str_with_args(def_id, args),
        }
    }

    fn expr(&mut self, id: thir::ExprId) -> String {
        let expr = &self.thir().exprs[id];
        match &expr.kind {
            thir::ExprKind::Scope { value, .. } => self.expr(*value),
            thir::ExprKind::Box { value } => format!("box {}", self.expr(*value)),
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
            thir::ExprKind::Cast { source } => format!("({} as _)", self.expr(*source)),
            thir::ExprKind::Use { source }
            | thir::ExprKind::NeverToAny { source }
            | thir::ExprKind::PlaceUnwrapUnsafeBinder { source }
            | thir::ExprKind::ValueUnwrapUnsafeBinder { source }
            | thir::ExprKind::WrapUnsafeBinder { source } => self.expr(*source),
            thir::ExprKind::PointerCoercion { source, .. } => {
                format!("({} as _)", self.expr(*source))
            }
            thir::ExprKind::Loop { body } => format!("loop {}", self.expr_in_block(*body)),
            thir::ExprKind::LoopMatch { match_data, .. } => self.expr(match_data.scrutinee),
            thir::ExprKind::Let { expr, pat } => {
                format!("let {} = {}", self.pat(pat), self.expr(*expr))
            }
            thir::ExprKind::Match {
                scrutinee, arms, ..
            } => {
                let mut s = format!("match {} {{\n", self.expr(*scrutinee));
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
                let items = fields.iter().map(|f| self.expr(*f)).format(", ");
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
                let dbpp = DesugaredBodyPrettyPrinter { tcx: self.tcx };
                match dbpp.print_body(closure.closure_id) {
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
                    if let Some(assoc) = self.tcx.opt_associated_item(def_id) {
                        let container_def_id = self.tcx.parent(assoc.def_id);
                        let method_generics = self.tcx.generics_of(def_id);
                        let (container_args, method_args) =
                            args.split_at(method_generics.parent_count);
                        let method_name = assoc.name();
                        let method_args = self.format_generic_args(method_args);
                        let container = match assoc.container {
                            AssocContainer::Trait => {
                                let [self_ty, trait_args @ ..] = container_args else {
                                    unreachable!()
                                };
                                let self_ty = self.print_ty(self_ty.as_type().unwrap());
                                let trait_pred = self.path_with_args(container_def_id, trait_args);
                                format!("<{self_ty} as {trait_pred}>",)
                            }
                            AssocContainer::InherentImpl | AssocContainer::TraitImpl(_) => {
                                self.path_with_args(container_def_id, container_args)
                            }
                        };
                        format!("{container}::{method_name}{method_args}")
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

    fn pat(&self, pat: &Pat<'tcx>) -> String {
        match &pat.kind {
            PatKind::Missing => "_".to_string(),
            PatKind::Wild => "_".to_string(),
            PatKind::AscribeUserType { subpattern, .. } => self.pat(subpattern),
            PatKind::Binding {
                name, mode, subpattern, ..
            } => {
                let mut s = String::new();
                s.push_str(mode.prefix_str());
                s.push_str(&name.to_string());
                if let Some(sub) = subpattern {
                    s.push_str(" @ ");
                    s.push_str(&self.pat(sub));
                }
                s
            }
            PatKind::Variant {
                adt_def,
                variant_index,
                subpatterns,
                ..
            } => {
                let variant = &adt_def.variant(*variant_index);
                let mut parts = Vec::new();
                for fp in subpatterns {
                    parts.push(format!(
                        "{}: {}",
                        fp.field.as_usize(),
                        self.pat(&fp.pattern)
                    ));
                }
                format!(
                    "{}::{} {{ {} }}",
                    self.path(adt_def.did()),
                    variant.name,
                    parts.join(", ")
                )
            }
            PatKind::Leaf { subpatterns } => {
                // TODO: tuple pats, `..` pats
                let mut parts = Vec::new();
                for fp in subpatterns {
                    parts.push(format!(
                        "{}: {}",
                        fp.field.as_usize(),
                        self.pat(&fp.pattern)
                    ));
                }
                format!("{{ {} }}", parts.join(", "))
            }
            PatKind::Deref { subpattern } | PatKind::DerefPattern { subpattern, .. } => {
                format!("*{}", self.pat(subpattern))
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

    fn local_name(&self, id: thir::LocalVarId) -> String {
        if let Some(name) = self.body.synthetic_local_name(id.0) {
            name.to_string()
        } else {
            self.tcx.hir_name(id.0).to_string()
        }
    }

    fn path(&self, def_id: DefId) -> String {
        self.tcx.value_path_str_with_args(def_id, &[])
    }
}
