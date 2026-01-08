use rustc_ast::LitKind;
use rustc_hir as hir;
use rustc_middle::{
    mir::{AssignOp, BinOp, BorrowKind, FakeBorrowKind, UnOp},
    thir::{self, Pat, PatKind, Thir},
    ty::{AssocContainer, TyCtxt, TyKind},
};
use std::fmt::Write;

pub fn print_thir<'tcx>(tcx: TyCtxt<'tcx>, def_id: hir::def_id::LocalDefId) -> String {
    match tcx.thir_body(def_id) {
        Ok((thir, root)) if !thir.is_stolen() => {
            let thir = thir.borrow();
            let def_path = tcx.def_path_str(def_id);
            let mut printer = ThirPrinter { tcx, thir: &thir };
            let mut output = String::new();
            writeln!(output, "fn {def_path}()").unwrap();
            writeln!(output, "{}", printer.expr(root, 0)).unwrap();
            output
        }
        _ => "error".into(),
    }
}

struct ThirPrinter<'tcx, 'a> {
    tcx: TyCtxt<'tcx>,
    thir: &'a Thir<'tcx>,
}

impl<'tcx, 'a> ThirPrinter<'tcx, 'a> {
    fn indent(&self, level: usize) -> String {
        "    ".repeat(level)
    }

    fn expr(&mut self, id: thir::ExprId, indent: usize) -> String {
        let expr = &self.thir.exprs[id];
        match &expr.kind {
            thir::ExprKind::Scope { value, .. } => self.expr(*value, indent),
            thir::ExprKind::Box { value } => format!("box {}", self.expr(*value, indent)),
            thir::ExprKind::If {
                cond,
                then,
                else_opt,
                ..
            } => {
                let mut s = format!(
                    "if {} {}",
                    self.expr(*cond, indent),
                    self.expr_in_block(*then, indent)
                );
                if let Some(e) = else_opt {
                    s.push_str(" else ");
                    s.push_str(&self.expr_in_block(*e, indent));
                }
                s
            }
            thir::ExprKind::Call { fun, args, .. } => {
                let args = args
                    .iter()
                    .map(|arg| self.expr(*arg, indent))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{}({})", self.expr(*fun, indent), args)
            }
            thir::ExprKind::ByUse { expr, .. } => {
                format!("{}.use", self.expr(*expr, indent))
            }
            thir::ExprKind::Deref { arg } => format!("*{}", self.expr(*arg, indent)),
            thir::ExprKind::Binary { op, lhs, rhs } => {
                let op_str = self.bin_op(*op);
                format!(
                    "{} {} {}",
                    self.expr(*lhs, indent),
                    op_str,
                    self.expr(*rhs, indent)
                )
            }
            thir::ExprKind::LogicalOp { op, lhs, rhs } => {
                let op_str = match op {
                    thir::LogicalOp::And => "&&",
                    thir::LogicalOp::Or => "||",
                };
                format!(
                    "{} {} {}",
                    self.expr(*lhs, indent),
                    op_str,
                    self.expr(*rhs, indent)
                )
            }
            thir::ExprKind::Unary { op, arg } => {
                let op_str = self.un_op(*op);
                format!("{}{}", op_str, self.expr(*arg, indent))
            }
            thir::ExprKind::Cast { source } => format!("({} as _)", self.expr(*source, indent)),
            thir::ExprKind::Use { source }
            | thir::ExprKind::NeverToAny { source }
            | thir::ExprKind::PlaceUnwrapUnsafeBinder { source }
            | thir::ExprKind::ValueUnwrapUnsafeBinder { source }
            | thir::ExprKind::WrapUnsafeBinder { source } => self.expr(*source, indent),
            thir::ExprKind::PointerCoercion { source, .. } => {
                format!("({} as _)", self.expr(*source, indent))
            }
            thir::ExprKind::Loop { body } => format!("loop {}", self.expr_in_block(*body, indent)),
            thir::ExprKind::LoopMatch { match_data, .. } => self.expr(match_data.scrutinee, indent),
            thir::ExprKind::Let { expr, pat } => {
                format!("let {} = {}", self.pat(pat), self.expr(*expr, indent))
            }
            thir::ExprKind::Match {
                scrutinee, arms, ..
            } => {
                let mut s = format!("match {} {{\n", self.expr(*scrutinee, indent + 1));
                for arm_id in arms.iter() {
                    s.push_str(&self.arm(*arm_id, indent + 1));
                }
                s.push_str(&format!("{}}}", self.indent(indent)));
                s
            }
            thir::ExprKind::Block { block } => self.block(*block, indent),
            thir::ExprKind::Assign { lhs, rhs } => {
                format!("{} = {}", self.expr(*lhs, indent), self.expr(*rhs, indent))
            }
            thir::ExprKind::AssignOp { op, lhs, rhs } => {
                let op_str = self.assign_op(*op);
                format!(
                    "{} {}= {}",
                    self.expr(*lhs, indent),
                    op_str,
                    self.expr(*rhs, indent)
                )
            }
            thir::ExprKind::Field { lhs, name, .. } => {
                format!("{}.{}", self.expr(*lhs, indent), name.as_usize())
            }
            thir::ExprKind::Index { lhs, index } => {
                format!("{}[{}]", self.expr(*lhs, indent), self.expr(*index, indent))
            }
            thir::ExprKind::VarRef { id } => self.local_name(*id),
            thir::ExprKind::UpvarRef { var_hir_id, .. } => self.local_name(*var_hir_id),
            thir::ExprKind::Borrow { borrow_kind, arg } => {
                let prefix = match borrow_kind {
                    BorrowKind::Shared
                    | BorrowKind::Fake(FakeBorrowKind::Shallow | FakeBorrowKind::Deep) => "&",
                    BorrowKind::Mut { .. } => "&mut ",
                };
                format!("{}{}", prefix, self.expr(*arg, indent))
            }
            thir::ExprKind::RawBorrow { mutability, arg } => {
                let prefix = match mutability {
                    hir::Mutability::Mut => "raw mut",
                    hir::Mutability::Not => "raw const",
                };
                format!("&{} {}", prefix, self.expr(*arg, indent))
            }
            thir::ExprKind::Break { value, .. } => match value {
                Some(v) => format!("break {}", self.expr(*v, indent)),
                None => "break".to_string(),
            },
            thir::ExprKind::Continue { .. } => "continue".to_string(),
            thir::ExprKind::ConstContinue { value, .. } => {
                format!("const_continue {}", self.expr(*value, indent))
            }
            thir::ExprKind::Return { value } => match value {
                Some(v) => format!("return {}", self.expr(*v, indent)),
                None => "return".to_string(),
            },
            thir::ExprKind::Become { value } => format!("become {}", self.expr(*value, indent)),
            thir::ExprKind::ConstBlock { .. } => "const { /* ... */ }".to_string(),
            thir::ExprKind::Repeat { value, count } => {
                format!("[{}; {}]", self.expr(*value, indent), count)
            }
            thir::ExprKind::Array { fields } => {
                let items = fields
                    .iter()
                    .map(|f| self.expr(*f, indent))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("[{}]", items)
            }
            thir::ExprKind::Tuple { fields } => {
                let items = fields
                    .iter()
                    .map(|f| self.expr(*f, indent))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("({})", items)
            }
            thir::ExprKind::Adt(adt) => {
                let adt_name = self.tcx.def_path_str(adt.adt_def.did());
                if adt.fields.is_empty() {
                    format!("{} {{}}", adt_name)
                } else {
                    let mut parts = Vec::new();
                    for field in adt.fields.iter() {
                        parts.push(format!(
                            "{}: {}",
                            field.name.as_usize(),
                            self.expr(field.expr, indent)
                        ));
                    }
                    if let thir::AdtExprBase::Base(base) = &adt.base {
                        parts.push(format!("..{}", self.expr(base.base, indent)));
                    }
                    format!("{} {{ {} }}", adt_name, parts.join(", "))
                }
            }
            thir::ExprKind::PlaceTypeAscription { source, .. }
            | thir::ExprKind::ValueTypeAscription { source, .. } => self.expr(*source, indent),
            thir::ExprKind::Closure(closure) => {
                let upvars = closure
                    .upvars
                    .iter()
                    .map(|u| self.expr(*u, indent))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("/* closure upvars: [{upvars}] */ || {{ ... }}")
            }
            thir::ExprKind::Literal { lit, neg } => {
                let lit_str = self.literal(lit);
                if *neg { format!("-{lit_str}") } else { lit_str }
            }
            thir::ExprKind::NonHirLiteral { lit, .. } => format!("{lit}"),
            thir::ExprKind::ZstLiteral { .. } => match expr.ty.kind() {
                TyKind::FnDef(def_id, args) => {
                    if let Some(assoc) = self.tcx.opt_associated_item(*def_id)
                        && matches!(assoc.container, AssocContainer::Trait)
                        && let Some(trait_def_id) = assoc.trait_container(self.tcx)
                    {
                        let method_generics = self.tcx.generics_of(*def_id);
                        let trait_arg_count = method_generics.parent_count;
                        let self_ty = args.type_at(0);
                        let trait_args = args
                            .iter()
                            .take(trait_arg_count)
                            .skip(1)
                            .map(|a| a.to_string())
                            .collect::<Vec<_>>()
                            .join(", ");
                        let method_args = args
                            .iter()
                            .skip(trait_arg_count)
                            .map(|a| a.to_string())
                            .collect::<Vec<_>>()
                            .join(", ");
                        let trait_args_suffix = if trait_args.is_empty() {
                            String::new()
                        } else {
                            format!("::<{trait_args}>")
                        };
                        let method_args_suffix = if method_args.is_empty() {
                            String::new()
                        } else {
                            format!("::<{method_args}>")
                        };
                        format!(
                            "<{} as {}{}>::{}{}",
                            self_ty,
                            self.tcx.def_path_str(trait_def_id),
                            trait_args_suffix,
                            assoc.name(),
                            method_args_suffix
                        )
                    } else {
                        let args = args
                            .iter()
                            .map(|a| a.to_string())
                            .collect::<Vec<_>>()
                            .join(", ");
                        if args.is_empty() {
                            self.tcx.def_path_str(*def_id)
                        } else {
                            format!("{}::<{}>", self.tcx.def_path_str(*def_id), args)
                        }
                    }
                }
                _ => "()".to_string(),
            },
            thir::ExprKind::NamedConst { def_id, .. } => self.tcx.def_path_str(*def_id),
            thir::ExprKind::ConstParam { param, .. } => param.name.to_string(),
            thir::ExprKind::StaticRef { def_id, .. } => {
                format!("&{}", self.tcx.def_path_str(*def_id))
            }
            thir::ExprKind::InlineAsm(_) => "asm!(...)".to_string(),
            thir::ExprKind::ThreadLocalRef(def_id) => {
                format!("thread_local!({})", self.tcx.def_path_str(*def_id))
            }
            thir::ExprKind::Yield { value } => format!("yield {}", self.expr(*value, indent)),
        }
    }

    fn expr_in_block(&mut self, id: thir::ExprId, indent: usize) -> String {
        match self.thir.exprs[id].kind {
            thir::ExprKind::Block { block } => self.block(block, indent),
            thir::ExprKind::Scope { value, .. } => self.expr_in_block(value, indent),
            thir::ExprKind::Use { source }
            | thir::ExprKind::NeverToAny { source }
            | thir::ExprKind::PlaceUnwrapUnsafeBinder { source }
            | thir::ExprKind::ValueUnwrapUnsafeBinder { source }
            | thir::ExprKind::WrapUnsafeBinder { source }
            | thir::ExprKind::PlaceTypeAscription { source, .. }
            | thir::ExprKind::ValueTypeAscription { source, .. } => {
                self.expr_in_block(source, indent)
            }
            _ => format!("{{ {} }}", self.expr(id, indent)),
        }
    }

    fn block(&mut self, id: thir::BlockId, indent: usize) -> String {
        let block = &self.thir.blocks[id];
        let mut s = format!("{{\n");
        for stmt_id in block.stmts.iter() {
            s.push_str(&self.stmt(*stmt_id, indent + 1));
        }
        if let Some(expr) = block.expr {
            s.push_str(&self.indent(indent + 1));
            s.push_str(&self.expr(expr, indent + 1));
            s.push_str(";\n");
        }
        s.push_str(&format!("{}}}", self.indent(indent)));
        s
    }

    fn stmt(&mut self, id: thir::StmtId, indent: usize) -> String {
        let stmt = &self.thir.stmts[id];
        match &stmt.kind {
            thir::StmtKind::Expr { expr, .. } => {
                format!("{}{};\n", self.indent(indent), self.expr(*expr, indent))
            }
            thir::StmtKind::Let {
                pattern,
                initializer,
                else_block,
                ..
            } => {
                let mut s = format!("{}let {}", self.indent(indent), self.pat(pattern));
                if let Some(init) = initializer {
                    s.push_str(&format!(" = {}", self.expr(*init, indent)));
                }
                if let Some(else_blk) = else_block {
                    s.push_str(" else ");
                    s.push_str(&self.block(*else_blk, indent));
                }
                s.push_str(";\n");
                s
            }
        }
    }

    fn arm(&mut self, id: thir::ArmId, indent: usize) -> String {
        let arm = &self.thir.arms[id];
        let mut s = format!("{}{} ", self.indent(indent), self.pat(&arm.pattern));
        if let Some(guard) = arm.guard {
            s.push_str(&format!("if {} ", self.expr(guard, indent)));
        }
        s.push_str("=> ");
        s.push_str(&self.expr(arm.body, indent));
        s.push_str(",\n");
        s
    }

    fn pat(&self, pat: &Pat<'tcx>) -> String {
        match &pat.kind {
            PatKind::Missing => "_".to_string(),
            PatKind::Wild => "_".to_string(),
            PatKind::AscribeUserType { subpattern, .. } => self.pat(subpattern),
            PatKind::Binding {
                name, subpattern, ..
            } => {
                let mut s = name.to_string();
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
                    self.tcx.def_path_str(adt_def.did()),
                    variant.name,
                    parts.join(", ")
                )
            }
            PatKind::Leaf { subpatterns } => {
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
            PatKind::Or { pats } => {
                let parts = pats
                    .iter()
                    .map(|p| self.pat(p))
                    .collect::<Vec<_>>()
                    .join(" | ");
                parts
            }
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
        self.tcx.hir_name(id.0).to_string()
    }
}
