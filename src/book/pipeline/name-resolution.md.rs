//@ # Name resolution
//@
//@ TODO: Tighten this text up.
//@
//@ TODO: name-resolution-macro-expansion.md promises to give all variables unique names.
//@ At present locals are left unchanged.
//@
//@
//@ ## Checks before name resolution
//@
//@ Programs with the following problems are rejected:
//@ - duplicate crate-level names
//@ - duplicate item names in a single block
//@ - patterns which repeat a name
//@ - sets of function parameters which repeat a name
//@
//@ These conditions are intended to make the name resolution rules below well defined.
//@
//@
//@ ## Resolution
//@
//@ The following rules are used to resolve path expressions to entities.
//@
//@ A crate-relative path expression with a single segment after `crate::` is resolved to the
//@ crate-level item which that segment names. If there is no such item the expression fails to
//@ resolve.
//@
//@ Crate-relative path expressions with multiple segments after `crate::` fail to resolve.
//@ TODO: this is because we don't have modules yet.
//@
//@ "Visibility" of items and locals works as follows:
//@ - An item defined at crate level is visible throughout the program.
//@ - An item defined in a block is visible throughout that block (even before its definition),
//@   (including in nested blocks and nested function bodies).
//@ - For purposes of name resolution, each function has an additional block (a "parameter block")
//@   surrounding its body; the locals introduced by the patterns in the function's parameters are
//@   visible in that block.
//@ - The locals introduced by a `let` statement's pattern are visible in the statements following
//@   the `let` statement in the same block, (including in nested blocks and nested function bodies),
//@   and in that block's final operand.
//@   (They're not visible in the let statement's initialiser or `else` block.)
//@
//@ TODO: somewhere needs to talk about what kinds of pattern introduce names; note that a single
//@ `let` statement or parameter may have a pattern binding more than one name. Later there will be
//@ single-identifier patterns which aren't bindings.
//@
//@ Single-segment path expressions are resolved as follows:
//@ - Each item or local which is visible to the path expression and whose name corresponds to the
//@   segment is a "candidate".
//@ - If there are no candidates the path expression fails to resolve.
//@ - If no blocks have candidates, the crate-level candidate is chosen.
//@ - Otherwise a candidate is chosen from the innermost block which has any candidates:
//@   - If that block is a parameter block, the parameter binding is chosen.
//@   - If any of the candidates are locals, the one introduced by the latest statement is chosen.
//@   - Otherwise the block's candidate item is chosen.
//@
//@ ## Rejected programs
//@
//@ Programs with the following problems are rejected:
//@ - path expressions which fail to resolve
//@ - path expressions in nested functions which resolve to locals outside the innermost function
//@   they're in
//@
//@ ## Path expression normalisation
//@
//@ Single-segment path expressions which resolve to crate-level items are rewritten to be
//@ crate-relative.
//@
//@ ## After these steps
//@ - Every path expression resolves.
//@ - No two crate-level items have the same name.
//@ - Every path resolving to a crate-level item is crate-relative.
//@ - No path expression in a nested function resolves to a local of an enclosing function.
//@
//@ ## Discussion
//@
//@ Treating the function parameters as living in a block outside the body means that an item in a
//@ function's body shadows that function's parameters.
//@
//@ Note that the resolution rules don't pay attention to function boundaries, and path expressions
//@ in nested functions can resolve to locals from an enclosing function (although programs in which
//@ this happens are rejected). It follows that items in function bodies can't be freely re-ordered
//@ with statements.
//@
//@ > The rest of this section is a work-in-progress experiment about making the book executable.
//@ >
//@ > For the sake of the runners, it pretends there's always a crate-level function named `print`.

use std::collections::BTreeSet; //#

use derive_generic_visitor::*; //#

use crate::CompilationError; //#
use crate::interactive_example; //#
use crate::language::*; //#

//@ Try moving `let greet = true;` up to just after `helper()`.
interactive_example! {
    desugar_names,
    fn greet() {}
    fn main() {
        greet();
        helper();
        fn helper() {
            greet();
            other();
            deep();
            fn deep() {}
        }
        fn other() {}
        {
            fn helper() {}
            helper();
        }
        let greet = true;
        print(greet);
    }
}

/// Returns the names bound by a pattern.
///
/// Reports CompilationError::Desugaring if the pattern tries to bind duplicate names.
fn bound_names(pattern: &Pattern) -> Result<Vec<Identifier>, CompilationError> {
    Ok(match pattern {
        Pattern::Identifier(identifier) => vec![identifier.clone()],
        Pattern::Wildcard => Vec::new(),
    })
}

/// Returns the names bound by patterns in the function's parameters.
///
/// Reports CompilationError::Desugaring if the parameters repeat a name.
fn parameter_names(function: &Function) -> Result<Vec<Identifier>, CompilationError> {
    let mut names = Vec::new();
    let mut seen = BTreeSet::new();
    for parameter in &function.parameters {
        if let FunctionParamKind::Regular {
            pattern: Some(pattern),
            ..
        } = &parameter.kind
        {
            for name in bound_names(pattern)? {
                if !seen.insert(name.clone()) {
                    return Err(CompilationError::Desugaring(format!(
                        "Function parameters bind `{name}` more than once"
                    )));
                }
                names.push(name);
            }
        }
    }
    Ok(names)
}

/// What a name means within a single block scope.
enum Binding {
    Local,
    Item,
}

/// The names defined in a single block expression (or set of function parameters).
#[derive(Default)]
struct BlockScope {
    /// Names of the locals visible in the scope at the current point of the walk
    /// (or names of the function parameters).
    locals: BTreeSet<Identifier>,
    /// Names of the items defined in this scope.
    items: BTreeSet<Identifier>,
    /// How many functions the block is inside.
    function_depth: usize,
}

impl BlockScope {
    fn add_item(&mut self, name: Identifier) -> Result<(), CompilationError> {
        if !self.items.insert(name.clone()) {
            return Err(CompilationError::Desugaring(format!(
                "Duplicate block-level definition of `{name}`"
            )));
        }
        Ok(())
    }

    fn add_local(&mut self, source_name: Identifier) {
        self.locals.insert(source_name);
    }

    fn resolve(&self, name: &Identifier) -> Option<Binding> {
        if self.locals.contains(name) {
            return Some(Binding::Local);
        }
        if self.items.contains(name) {
            return Some(Binding::Item);
        }
        None
    }
}

/// Information about how a name resolves from a particular position in the walk.
enum Resolution {
    /// A local from the walk's current function.
    Local,
    /// A local from a function enclosing the walk's current function.
    CapturedLocal,
    /// An item defined in a block.
    BlockItem,
    /// An item defined at crate level.
    CrateItem,
}

/// Tracks which names are visible during an AST walk.
#[derive(Default)]
struct Scopes {
    crate_level_items: BTreeSet<Identifier>,
    block_scopes: Vec<BlockScope>,
    /// How many functions the walk is inside.
    function_depth: usize,
}

impl Scopes {
    /// Records a crate-level name.
    fn add_at_crate_level(&mut self, name: Identifier) -> Result<(), CompilationError> {
        if !self.crate_level_items.insert(name.clone()) {
            return Err(CompilationError::Desugaring(format!(
                "Duplicate crate-level definition of `{name}`"
            )));
        }
        Ok(())
    }

    /// Checks whether a name is defined as a crate-level item.
    fn exists_at_crate_level(&self, name: &Identifier) -> bool {
        self.crate_level_items.contains(name)
    }

    /// Enters a function, given the names bound by its parameters.
    fn enter_function(&mut self, parameter_names: Vec<Identifier>) {
        self.function_depth += 1;
        // We treat function parameters as belonging to a block scope of their own.
        self.enter_block();
        for identifier in parameter_names {
            self.add_local(identifier);
        }
    }

    /// Undoes the most recent enter_function().
    fn leave_function(&mut self) {
        self.leave_block();
        self.function_depth -= 1;
    }

    /// Enters a new block scope.
    fn enter_block(&mut self) {
        self.block_scopes.push(BlockScope {
            function_depth: self.function_depth,
            ..Default::default()
        })
    }

    /// Leaves the current block scope.
    fn leave_block(&mut self) {
        self.block_scopes
            .pop()
            .expect("there should be at least one block scope");
    }

    /// Records an entry for an item defined in the current block scope.
    fn add_item(&mut self, name: Identifier) -> Result<(), CompilationError> {
        let block_scope = self
            .block_scopes
            .last_mut()
            .expect("there should be a current block scope");
        block_scope.add_item(name)
    }

    /// Records an entry for a local bound at this point in the current block scope.
    fn add_local(&mut self, name: Identifier) {
        let block_scope = self
            .block_scopes
            .last_mut()
            .expect("there should be a current block scope");
        block_scope.add_local(name)
    }

    /// Resolves a name against the block scopes and crate-level items.
    fn resolve(&self, name: &Identifier) -> Option<Resolution> {
        self.block_scopes
            .iter()
            .rev()
            .find_map(|block_scope| {
                Some(match block_scope.resolve(name)? {
                    Binding::Local if block_scope.function_depth != self.function_depth => {
                        Resolution::CapturedLocal
                    }
                    Binding::Local => Resolution::Local,
                    Binding::Item => Resolution::BlockItem,
                })
            })
            .or_else(|| {
                self.exists_at_crate_level(name)
                    .then_some(Resolution::CrateItem)
            })
    }
}

pub fn desugar_names(program: &mut Program) -> Result<(), CompilationError> {
    struct Walker {
        /// Information about what's currently in scope.
        scopes: Scopes,
    }

    impl Visitor for Walker {
        type Break = CompilationError;
    }

    impl VisitAstMut for Walker {
        /// Keeps track of which function the walk is inside.
        fn visit_function(&mut self, function: &mut Function) -> ControlFlow<Self::Break> {
            self.scopes
                .enter_function(parameter_names(function).map_or_else(Break, Continue)?);
            self.visit_inner(function)?;
            self.scopes.leave_function();
            Continue(())
        }

        /// For each block expression:
        /// - keeps track of which block the walk is inside
        /// - before continuing the walk inside the block, records which nested items are in the
        ///   block.
        fn visit_block_expression(
            &mut self,
            block: &mut BlockExpression,
        ) -> ControlFlow<Self::Break> {
            self.scopes.enter_block();
            for statement in &mut block.statements {
                if let Statement::Item(nested_item) = statement {
                    let ItemKind::Function(function) = &nested_item.kind;
                    if let Err(e) = self.scopes.add_item(function.name.clone()) {
                        return Break(e);
                    }
                }
            }
            self.visit_inner(block)?;
            self.scopes.leave_block();
            Continue(())
        }

        /// Keeps track of local names bound by statements.
        fn visit_statement(&mut self, statement: &mut Statement) -> ControlFlow<Self::Break> {
            // Visits the initialiser and any 'else' block, without binding the new names there.
            self.visit_inner(statement)?;
            if let Statement::Let { pattern, .. } = statement {
                for identifier in bound_names(pattern).map_or_else(Break, Continue)? {
                    self.scopes.add_local(identifier);
                }
            }
            Continue(())
        }

        /// Checks that names resolve and rewrites path expressions as necessary:
        /// - rewrites single-segment path expressions which resolve to crate-level items
        ///   as crate-relative path expressions
        /// - rejects path expressions which resolve to a local from an enclosing function
        /// - rejects path expressions which don't resolve.
        fn visit_path_expression(
            &mut self,
            path_expr: &mut PathExpression,
        ) -> ControlFlow<Self::Break> {
            match path_expr {
                PathExpression::SingleSegment(name) => match self.scopes.resolve(name) {
                    Some(Resolution::Local | Resolution::BlockItem) => {}
                    Some(Resolution::CrateItem) => {
                        *path_expr = PathExpression::CrateRelative(vec![name.clone()]);
                    }
                    Some(Resolution::CapturedLocal) => {
                        return Break(CompilationError::Desugaring(format!(
                            "Can't use local `{name}` of an enclosing function from a nested function",
                        )));
                    }
                    None => {
                        return Break(CompilationError::Desugaring(format!(
                            "Can't resolve name `{name}`",
                        )));
                    }
                },
                PathExpression::CrateRelative(segments) => match segments.as_slice() {
                    [name] => {
                        if !self.scopes.exists_at_crate_level(name) {
                            return Break(CompilationError::Desugaring(format!(
                                "Can't resolve `crate::{name}`"
                            )));
                        }
                    }
                    _ => {
                        return Break(CompilationError::Desugaring(
                            "Can't resolve multi-segment path expression".to_owned(),
                        ));
                    }
                },
            }
            self.visit_inner(path_expr)
        }
    }

    // Record the crate-level items.
    let mut scopes = Scopes::default();
    // We pretend there is always a crate-level function named `print`.
    scopes.add_at_crate_level("print".into())?;
    for crate_item in &program.items {
        let ItemKind::Function(function) = &crate_item.kind;
        scopes.add_at_crate_level(function.name.clone())?;
    }

    // Walk all bodies performing the desugaring.
    let mut walker = Walker { scopes };
    walker.visit_program(program).continue_ok()
}
