use itertools::Itertools;
use std::ops::Deref;

use rustc_ast::LitKind;
use rustc_hir::{self as hir, def::CtorKind};
use rustc_middle::{
    mir::{AssignOp, BinOp, BorrowKind, FakeBorrowKind, UnOp},
    thir::{self, BlockSafety, PatKind, Thir},
    ty::{TyKind, VariantDef},
};

use super::CratePrinter;
use crate::desugar::Body;

pub(crate) struct ThirPrinter<'a, 'tcx> {
    crate_printer: &'a CratePrinter<'tcx>,
    body: &'a Body<'tcx>,
}

#[derive(Clone)]
pub(crate) struct PrintedBody {
    pub params: Vec<String>,
    pub body: String,
}

impl<'tcx> Deref for ThirPrinter<'_, 'tcx> {
    type Target = CratePrinter<'tcx>;
    fn deref(&self) -> &Self::Target {
        self.crate_printer
    }
}

impl<'a, 'tcx> ThirPrinter<'a, 'tcx> {
    pub(crate) fn new(crate_printer: &'a CratePrinter<'tcx>, body: &'a Body<'tcx>) -> Self {
        Self {
            crate_printer,
            body,
        }
    }

    fn thir(&self) -> &'a Thir<'tcx> {
        &self.body.thir
    }

    /// Print the whole body tracked by this printer.
    pub(crate) fn into_printed_body(mut self) -> Option<PrintedBody> {
        let params = self
            .body
            .thir
            .params
            .iter()
            .map(|param| {
                param
                    .pat
                    .as_deref()
                    .map(|p| self.pat(p))
                    .unwrap_or("_".into())
            })
            .collect::<Vec<_>>();
        let body = self.expr_in_block(self.body.root_expr);
        Some(PrintedBody { params, body })
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
