//@ # Name resolution
//@
//@ TODO: name-resolution-macro-expansion.md promises to give all variables unique names.
//@ At present locals are left unchanged.
//@
//@ ## Checks that resolution depends on
//@
//@ These checks make the [name resolution rules](#resolution) below well defined, by ensuring that
//@ no frame has two entries for the same name.

use crate::Rejection; //#
use crate::interactive_example; //#
use crate::language::*; //#
use derive_generic_visitor::*; //#
use std::collections::{BTreeMap, BTreeSet}; //#
use std::convert::Infallible; //#

pub fn checks_that_resolution_depends_on(program: &Program) -> Result<(), Rejection> {
    //@ Programs with duplicate crate-level names are rejected:
    reject_repeats(crate_level_names(program), |name| {
        format!("Duplicate crate-level declaration of `{name}`")
    })?;

    //@ Programs with duplicate item names in a single block are rejected:
    program.visit_all(|block: &BlockExpression| {
        reject_repeats(block_item_names(block), |name| {
            format!("Duplicate block-level declaration of `{name}`")
        })
    })?;

    //@ Programs with patterns that repeat a name are rejected:
    program.visit_all(|pattern: &Pattern| {
        reject_repeats(bound_names(pattern), |name| {
            format!("Pattern binds `{name}` more than once")
        })
    })?;
    //@ > At present this can't reject anything, because a pattern binds at most one name.

    //@ Programs with functions whose parameters repeat a name are rejected:
    program.visit_all(|function: &Function| {
        reject_repeats(parameter_names(function), |name| {
            format!("Function parameters bind `{name}` more than once")
        })
    })
}

//@ ## Resolution
//@
//@ The rules below describe how to "resolve" a path expression to an
//@ [entity](../language/names.md.rs).
//@
//@ For now the forms of entity available from resolution are items and locals (i.e. parameter or
//@ variable bindings).
//@
//@ For now functions are the only kind of item.
//@ In this chapter "function" means a particular `fn` item.
//@
//@ > A `FunctionId` is how the code in this chapter identifies a particular function.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct FunctionId(usize);

#[derive(Clone)]
enum Entity {
    Function(FunctionId),
    Local {
        name: Identifier,
        /// The function the local belongs to.
        belongs_to: FunctionId,
    },
}

//@ A local "belongs to" the function whose parameters or body contain the pattern that introduces
//@ it.
//@
//@ TODO: somewhere needs to talk about what kinds of pattern introduce names. Later there will be
//@ patterns that bind more than one name. Later there will be single-identifier patterns that
//@ aren't bindings.
//@
//@ For the purposes of name resolution, there is a tree of "frames". Each frame covers part of the
//@ program, and a child frame's part is inside its parent's part. So it's meaningful to talk about
//@ the sequence of frames that cover a particular point in the program, and about the "outermost"
//@ and "innermost" frames in the sequence.
//@
//@ Each frame has a map of names (identifiers) to entities. The members of the map are the frame's
//@ "entries".
struct Frame {
    /// The frame's entries, mapping names to entities.
    entries: BTreeMap<Identifier, Entity>,
}

//@ > A `Resolver` walks a program once, keeping track of the frames that cover the walk's current
//@ > point.
//@ > It records what each path expression resolves to, and also fills in an `AstBookkeeping`
//@ > describing where the functions and path expressions are in the program
//@ > (see [Supporting code](#supporting-code)).
#[derive(Default)]
struct Resolver {
    /// The frames covering the current point in the walk, from outermost to innermost.
    frames: Vec<Frame>,
    /// The functions containing the current point in the walk, from outermost to innermost.
    containing_functions: Vec<FunctionId>,
    /// The results of resolving path expressions (built up during the walk).
    resolutions: Resolutions,
    /// Book-keeping about where functions and path expressions are in the program (built up during
    /// the walk).
    bookkeeping: AstBookkeeping,
}

#[derive(Default)]
struct Resolutions {
    /// The entity each path expression resolves to, if any, indexed by `PathExpressionId`.
    entities: Vec<Option<Entity>>,
}

impl VisitAst for Resolver {
    //@ > The resolver resolves each path expression during the walk, using the rules given
    //@ > below, and records the result.
    fn visit_path_expression(
        &mut self,
        path_expression: &PathExpression,
    ) -> ControlFlow<Infallible> {
        let entity = self.resolve(path_expression).cloned();
        self.resolutions.entities.push(entity);
        self.bookkeeping
            .record_path_expression(self.innermost_function());
        self.visit_inner(path_expression)
    }

    //@ The frame structure is as follows:
    //@
    //@ - A single "crate frame" covers the whole program.
    //@   It maps the name of each crate-level item to that item.
    fn visit_program(&mut self, program: &Program) -> ControlFlow<Infallible> {
        let mut frame = Frame::new();
        self.add_print_entry(&mut frame);
        for item in &program.items {
            self.add_item_entry(&mut frame, item);
        }
        self.frames.push(frame);
        self.visit_inner(program)?;
        self.frames.pop();
        Continue(())
    }

    //@ - A "parameter frame" for each function covers the function's signature and body.
    //@   It maps the names of the locals introduced by the patterns in the function's parameters to
    //@   those locals.
    fn visit_function(&mut self, function: &Function) -> ControlFlow<Infallible> {
        let function_id = self.declared_function_id(function);
        self.bookkeeping.record_declaration(function_id);
        let mut frame = Frame::new();
        for name in parameter_names(function) {
            let local = Entity::Local {
                name: name.clone(),
                belongs_to: function_id,
            };
            frame.add_entry(name, local);
        }
        self.frames.push(frame);
        self.containing_functions.push(function_id);
        self.visit_inner(function)?;
        self.containing_functions.pop();
        self.frames.pop();
        Continue(())
    }

    //@ - A "block frame" for each block expression covers that block.
    //@   It maps the name of each item declared directly in the block to that item. It has no
    //@   entries for locals.
    fn visit_block_expression(&mut self, block: &BlockExpression) -> ControlFlow<Infallible> {
        let mut frame = Frame::new();
        for statement in &block.statements {
            if let Statement::Item(item) = statement {
                self.add_item_entry(&mut frame, item);
            }
        }
        self.frames.push(frame);
        self.visit_statements(&block.statements, &block.tail)?;
        self.frames.pop();
        Continue(())
    }
}

impl Resolver {
    //@ - A "let frame" for each `let` statement covers the following statements in the same
    //@   block and that block's tail expression.
    //@   It maps the names of the locals introduced by the statement's pattern to those locals.
    //@   Note that it doesn't cover the `let` statement's own initializer or its optional `else`
    //@   block.
    fn visit_statements(
        &mut self,
        statements: &[Statement],
        tail: &Option<Box<Expression>>,
    ) -> ControlFlow<Infallible> {
        match statements {
            [] => self.visit(tail),
            [statement, rest @ ..] => {
                self.visit(statement)?;
                if let Statement::Let { pattern, .. } = statement {
                    let mut frame = Frame::new();
                    for name in bound_names(pattern) {
                        let local = Entity::Local {
                            name: name.clone(),
                            belongs_to: self.innermost_function(),
                        };
                        frame.add_entry(name, local);
                    }
                    self.frames.push(frame);
                    self.visit_statements(rest, tail)?;
                    self.frames.pop();
                    Continue(())
                } else {
                    self.visit_statements(rest, tail)
                }
            }
        }
    }

    //@ > In crate frames and block frames, the entry for an item depends on the item's kind.
    fn add_item_entry(&mut self, frame: &mut Frame, item: &Item) {
        match &item.kind {
            ItemKind::Function(function) => {
                let function_id = self.assign_function_id(&function.name);
                frame.add_entry(function.name.clone(), Entity::Function(function_id));
            }
        }
    }
}

impl Resolver {
    //@ The rules for resolution are as follows:
    fn resolve(&self, path_expression: &PathExpression) -> Option<&Entity> {
        match path_expression {
            //@ A single-segment path expression resolves using the innermost frame that covers the
            //@ path expression and has an entry for the name corresponding to that segment. The
            //@ expression resolves to the entity that entry maps the name to.
            //@
            //@ If there is no such frame the path expression fails to resolve.
            PathExpression::SingleSegment(name) => {
                for frame in self.frames.iter().rev() {
                    if let Some(entity) = frame.entries.get(name) {
                        return Some(entity);
                    }
                }
                None
            }
            //@ Resolving a crate-relative path expression depends on its number of segments.
            PathExpression::CrateRelative(segments) => match segments.as_slice() {
                //@ A crate-relative path expression that has a single segment after `crate::`
                //@ resolves to the crate-level item whose name corresponds to that segment.
                //@
                //@ If there is no such item the path expression fails to resolve.
                [name] => self.crate_frame().entries.get(name),
                //@ A crate-relative path expression with multiple segments after `crate::` fails to
                //@ resolve.
                _ => None,
                //@ TODO: this is because we don't have modules (or enums or associated items) yet.
            },
        }
    }
}

//@ > Each of the checks and desugarings below runs the resolver over the whole program and uses
//@ > the resolutions it recorded.

impl Resolutions {
    /// Returns the entity a path expression resolves to, if any.
    fn resolves_to(&self, path_expression_id: PathExpressionId) -> Option<Entity> {
        self.entities[path_expression_id.0].clone()
    }
}

/// Runs a `Resolver` over the whole program.
fn run_resolver(program: &Program) -> (Resolutions, AstBookkeeping) {
    let mut resolver = Resolver::default();
    let _ = resolver.visit(program);
    (resolver.resolutions, resolver.bookkeeping)
}

//@ ## Checks that depend on resolution
pub fn checks_that_depend_on_resolution(program: &Program) -> Result<(), Rejection> {
    let (resolutions, bookkeeping) = run_resolver(program);

    //@ Programs containing path expressions that fail to resolve are rejected.
    for_each_path_expression(program, |path_expression, path_expression_id| {
        if resolutions.resolves_to(path_expression_id).is_none() {
            return Err(Rejection::invalid(fails_to_resolve_message(
                path_expression,
            )));
        }
        Ok(())
    })?;

    //@ Programs containing path expressions that resolve to locals that belong to a function other
    //@ than the innermost function containing the path expression ("captured locals") are rejected.
    for_each_path_expression(program, |_, path_expression_id| {
        if let Some(Entity::Local { name, belongs_to }) =
            resolutions.resolves_to(path_expression_id)
            && belongs_to != bookkeeping.innermost_function_containing(path_expression_id)
        {
            return Err(Rejection::invalid(format!(
                "Can't use local `{name}` of an enclosing function from a nested function",
            )));
        }
        Ok(())
    })
    //@
}

//@ > We supply a suitable rejection message for path expressions that fail to resolve.
fn fails_to_resolve_message(path_expression: &PathExpression) -> String {
    match path_expression {
        PathExpression::SingleSegment(name) => format!("Can't resolve name `{name}`"),
        PathExpression::CrateRelative(segments) => match segments.as_slice() {
            [name] => format!("Can't resolve `crate::{name}`"),
            _ => "Can't resolve multi-segment path expression".to_owned(),
        },
    }
}

//@ ## Path expression normalisation
//@
//@ Single-segment path expressions that resolve to crate-level items are rewritten to be
//@ crate-relative.
pub fn path_expression_normalisation(program: &mut Program) -> Result<(), Rejection> {
    let (resolutions, bookkeeping) = run_resolver(program);
    for_each_path_expression_mut(program, |path_expression, path_expression_id| {
        if let PathExpression::SingleSegment(_) = path_expression
            && let Some(Entity::Function(function)) = resolutions.resolves_to(path_expression_id)
            && bookkeeping.is_crate_level(function)
        {
            *path_expression =
                PathExpression::CrateRelative(vec![bookkeeping.name(function).clone()]);
        }
    });
    Ok(())
}

//@ ## Nested function renaming
//@
//@ Each nested function is renamed to a fresh name, used nowhere else in the program (its
//@ "unambiguous name").
//@
//@ Path expressions that resolve to those functions are rewritten to use the unambiguous name
//@ (these are necessarily single-segment path expressions).
pub fn nested_function_renaming(program: &mut Program) -> Result<(), Rejection> {
    let (resolutions, bookkeeping) = run_resolver(program);
    let unambiguous_names = choose_unambiguous_names(program, &bookkeeping);

    //@ > The rewriting passes below use the information the resolver recorded for the unmodified
    //@ > program. Neither pass adds, removes, or reorders path expressions or function
    //@ > declarations, so the `PathExpressionId`s and `DeclarationId`s still identify the same
    //@ > nodes.

    for_each_path_expression_mut(program, |path_expression, path_expression_id| {
        if let Some(Entity::Function(function)) = resolutions.resolves_to(path_expression_id)
            && !bookkeeping.is_crate_level(function)
        {
            *path_expression = PathExpression::SingleSegment(unambiguous_names[&function].clone());
        }
    });

    for_each_declaration_mut(program, |function, declaration_id| {
        let function_id = bookkeeping.function_declared_by(declaration_id);
        function.name = unambiguous_names[&function_id].clone();
    });
    Ok(())
}

//@ > The unambiguous names have the form `<outer>__<inner>`, with a further underscore and numeric
//@ > suffix added if necessary to make an otherwise-unused name.
//@ > `<inner>` is the function's original name.
//@ > `<outer>` is the name of the function directly containing it if that is a crate-level
//@ > function, or that function's unambiguous name if it is a nested function.
fn choose_unambiguous_names(
    program: &Program,
    bookkeeping: &AstBookkeeping,
) -> BTreeMap<FunctionId, Identifier> {
    let mut used_names = UsedNames::for_program(program);
    let mut unambiguous_names = BTreeMap::new();
    for function in bookkeeping.functions_outer_first() {
        let name = bookkeeping.name(function);
        let unambiguous_name = match bookkeeping.function_directly_containing(function) {
            Some(outer) => {
                let outer_name = &unambiguous_names[&outer];
                used_names.fresh_name(&format!("{outer_name}__{name}"))
            }
            // Crate-level functions keep their existing names.
            None => name.clone(),
        };
        unambiguous_names.insert(function, unambiguous_name);
    }
    unambiguous_names
}

//@ ## Examples

//@ > The interactive example below runs the checks and the desugarings.
fn check_and_desugar_names(program: &mut Program) -> Result<(), Rejection> {
    checks_that_resolution_depends_on(program)?;
    checks_that_depend_on_resolution(program)?;
    path_expression_normalisation(program)?;
    nested_function_renaming(program)?;
    Ok(())
}

//@ Try moving `let greet = true;` up to just after `helper()`.
interactive_example! {
    check_and_desugar_names,
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

//@ ## After these steps
//@ - Every path expression resolves.
//@ - No two items in the program have the same name.
//@ - Every nested item has a name that is not used for any local.
//@ - Every path expression resolving to a crate-level item is crate-relative.
//@ - Every single-segment path expression contains either
//@   - the name of a local introduced in the same function; or
//@   - the unambiguous name of a nested function.
//@ - No path expression in a nested function resolves to a local of an enclosing function.
//@
//@ ## Discussion
//@
//@ Having a parameter frame outside the body's block frame means that an item in a function's body
//@ shadows that function's parameters.
//@
//@ Note that the resolution rules don't pay attention to function boundaries, and path expressions
//@ in nested functions can resolve to locals from an enclosing function (although programs in which
//@ this happens are rejected). It follows that items in function bodies can't be freely re-ordered
//@ with statements.
//@
//@ Justification that renaming a nested function preserves the program's meaning:
//@ - The rewritten path expressions still resolve to the same function because:
//@   - the block frame that had an entry for the old name now has an entry for the new one
//@   - no higher-priority frame has an entry for the new name, because the new name is fresh.
//@ - No other path expression's resolution changes because:
//@   - path expressions using the old name: removing an entry that wasn't the one found can't
//@     change which one is found
//@   - path expressions using the new name: there are none because the new name is fresh.
//@

//@ ## Choosing fresh names
//@
//@ > The code in this section tracks which identifiers are in use, and supplies the fresh names
//@ > used by [Nested function renaming](#nested-function-renaming).

/// Identifiers that are in use, so that fresh names can be chosen to avoid them.
#[derive(Default)]
struct UsedNames {
    names: BTreeSet<Identifier>,
}

impl UsedNames {
    /// Returns a UsedNames populated with the identifiers used in the program.
    ///
    /// It treats identifiers that appear in expressions as in use, not just identifiers that
    /// appear as the names of entities.
    fn for_program(program: &Program) -> Self {
        let mut used_names = UsedNames::default();
        // We pretend there is always a crate-level function named `print` (see
        // `crate_level_names`).
        used_names.mark_used("print".into());
        program.visit_all_infallible(|function: &Function| {
            used_names.mark_used(function.name.clone());
        });
        program.visit_all_infallible(|path_expression: &PathExpression| match path_expression {
            PathExpression::SingleSegment(identifier) => {
                used_names.mark_used(identifier.clone());
            }
            PathExpression::CrateRelative(segments) => {
                for segment in segments {
                    used_names.mark_used(segment.clone());
                }
            }
        });
        program.visit_all_infallible(|pattern: &Pattern| {
            for name in bound_names(pattern) {
                used_names.mark_used(name);
            }
        });
        used_names
    }

    /// Records an identifier as being in use.
    fn mark_used(&mut self, name: Identifier) {
        self.names.insert(name);
    }

    /// Returns a previously-unused identifier.
    ///
    /// If `base` isn't in use, returns it unchanged. Otherwise adds a numeric suffix.
    ///
    /// Records the returned identifier as being in use.
    fn fresh_name(&mut self, base: &Identifier) -> Identifier {
        std::iter::once(base.to_owned())
            .chain((0..).map(|n| format!("{base}_{n}")))
            .find(|candidate| self.names.insert(candidate.clone()))
            .expect("fresh_name should not run out of numbers")
    }
}

//@ ## Supporting code
//@
//@ > The code in this section supports the `Resolver`.
//@ >
//@ > The `AstBookkeeping`, `PathExpressionId` and `DeclarationId` provide a way to identify a
//@ > particular path expression or function declaration, and a way to ask which function a node is
//@ > in.

impl Visitor for Resolver {
    type Break = Infallible;
}

impl Resolver {
    /// Records a function in the AST bookkeeping, and returns its `FunctionId`.
    ///
    /// `name` must be the name of a function that is declared directly in an item-holding construct
    /// that the walk is currently visiting.
    fn assign_function_id(&mut self, name: &Identifier) -> FunctionId {
        let directly_containing = self.containing_functions.last().copied();
        self.bookkeeping.add_function(name, directly_containing)
    }

    /// Returns the `FunctionId` for a function that is declared at the current point in the walk.
    fn declared_function_id(&self, function: &Function) -> FunctionId {
        // Finds the innermost function-valued entry with the right name (which will be in a block
        // or crate frame).
        self.frames
            .iter()
            .rev()
            .find_map(|frame| match frame.entries.get(&function.name) {
                Some(Entity::Function(function_id)) => Some(*function_id),
                _ => None,
            })
            .expect("a function should have an entry in a frame surrounding its declaration")
    }

    /// Adds an entry for the pretend crate-level function named `print` (see `crate_level_names`).
    fn add_print_entry(&mut self, frame: &mut Frame) {
        let name: Identifier = "print".into();
        let function = self.assign_function_id(&name);
        frame.add_entry(name, Entity::Function(function));
    }

    /// Returns the program's crate frame.
    fn crate_frame(&self) -> &Frame {
        &self.frames[0]
    }

    /// Returns the innermost function that contains the current point in the walk.
    ///
    /// At present every expression and statement is in a function body.
    /// TODO: That will no longer be true when we have `const` or `static` items.
    fn innermost_function(&self) -> FunctionId {
        *self
            .containing_functions
            .last()
            .expect("there should be an enclosing function")
    }
}

/// Book-keeping information about where functions and path expressions are in the program's AST:
/// which function each path expression is in, and how functions are nested.
///
/// A `Resolver` fills this in during its walk.
#[derive(Default)]
struct AstBookkeeping {
    /// Information about each of the program's functions, indexed by `FunctionId`.
    ///
    /// Each function comes after the function(s) it is nested inside.
    functions: Vec<FunctionPlacement>,
    /// The program's functions, indexed by `DeclarationId`.
    declarations: Vec<FunctionId>,
    /// The innermost function containing each path expression, indexed by `PathExpressionId`.
    innermost_functions_containing_path_expressions: Vec<FunctionId>,
}

/// A function's name, and which function (if any) it is nested in.
struct FunctionPlacement {
    /// The function's name as written in its declaration.
    name: Identifier,
    /// The function directly containing this one, or `None` for a crate-level function.
    directly_containing: Option<FunctionId>,
}

impl AstBookkeeping {
    /// Returns the specified function's name, as written in its declaration.
    fn name(&self, function: FunctionId) -> &Identifier {
        &self.functions[function.0].name
    }

    /// Returns the function directly containing the specified function, or `None` for a crate-level
    /// function.
    fn function_directly_containing(&self, function: FunctionId) -> Option<FunctionId> {
        self.functions[function.0].directly_containing
    }

    /// Says whether the specified function is a crate-level function.
    fn is_crate_level(&self, function: FunctionId) -> bool {
        self.function_directly_containing(function).is_none()
    }

    /// Returns every function, with each one after the function(s) it is nested inside.
    fn functions_outer_first(&self) -> impl Iterator<Item = FunctionId> {
        (0..self.functions.len()).map(FunctionId)
    }

    /// Returns the innermost function that contains the specified path expression.
    fn innermost_function_containing(&self, path_expression_id: PathExpressionId) -> FunctionId {
        self.innermost_functions_containing_path_expressions[path_expression_id.0]
    }

    /// Returns the function that has the specified declaration id.
    fn function_declared_by(&self, declaration: DeclarationId) -> FunctionId {
        self.declarations[declaration.0]
    }

    /// Adds a function to the bookkeeping, returning the `FunctionId` to use to refer to it.
    fn add_function(
        &mut self,
        name: &Identifier,
        directly_containing: Option<FunctionId>,
    ) -> FunctionId {
        self.functions.push(FunctionPlacement {
            name: name.clone(),
            directly_containing,
        });
        FunctionId(self.functions.len() - 1)
    }

    /// Records the function declared by the next function declaration in pre-order.
    fn record_declaration(&mut self, function: FunctionId) {
        self.declarations.push(function);
    }

    /// Records the innermost function that contains the next path expression in pre-order.
    fn record_path_expression(&mut self, function: FunctionId) {
        self.innermost_functions_containing_path_expressions
            .push(function);
    }
}

/// Identifies a path expression, by its position in a pre-order walk of the program.
///
/// The `Resolver` visits path expressions and function declarations in the same order as
/// `visit_all`. The `PathExpressionId`s and `DeclarationId`s given out by the functions below match
/// that order.
#[derive(Clone, Copy)]
struct PathExpressionId(usize);

/// Identifies a function declaration, by its position in a pre-order walk of the program.
#[derive(Clone, Copy)]
struct DeclarationId(usize);

/// Calls `f` for each path expression in the program, also passing its `PathExpressionId`.
fn for_each_path_expression<E>(
    program: &Program,
    mut f: impl FnMut(&PathExpression, PathExpressionId) -> Result<(), E>,
) -> Result<(), E> {
    let mut next = 0;
    program.visit_all(|path_expression: &PathExpression| {
        let path_expression_id = PathExpressionId(next);
        next += 1;
        f(path_expression, path_expression_id)
    })
}

/// Variant of `for_each_path_expression` that allows `f` to modify the path expression.
///
/// `f` must not add, remove, or reorder path expressions.
fn for_each_path_expression_mut(
    program: &mut Program,
    mut f: impl FnMut(&mut PathExpression, PathExpressionId),
) {
    let mut next = 0;
    program.visit_all_mut_infallible(|path_expression: &mut PathExpression| {
        f(path_expression, PathExpressionId(next));
        next += 1;
    });
}

/// Calls `f` for each function declaration in the program, also passing its `DeclarationId`. `f`
/// can modify the declaration.
///
/// `f` must not add, remove, or reorder function declarations.
fn for_each_declaration_mut(
    program: &mut Program,
    mut f: impl FnMut(&mut Function, DeclarationId),
) {
    let mut next = 0;
    program.visit_all_mut_infallible(|declaration: &mut Function| {
        f(declaration, DeclarationId(next));
        next += 1;
    });
}

//@ ## Collecting names
//@
//@ > The code in this section lists the names that frames have entries for, and finds repeats in
//@ > those lists.
//@ >
//@ > For the sake of the runners, it pretends there's always a crate-level function named `print`.

/// Returns the names of the crate-level items, including any repeats.
fn crate_level_names(program: &Program) -> Vec<Identifier> {
    // We pretend there is always a crate-level function named `print`.
    let mut names = vec!["print".into()];
    for crate_item in &program.items {
        let ItemKind::Function(function) = &crate_item.kind;
        names.push(function.name.clone());
    }
    names
}

/// Returns the names of the items declared directly in a block, including any repeats.
fn block_item_names(block: &BlockExpression) -> Vec<Identifier> {
    let mut names = Vec::new();
    for statement in &block.statements {
        if let Statement::Item(nested_item) = statement {
            let ItemKind::Function(function) = &nested_item.kind;
            names.push(function.name.clone());
        }
    }
    names
}

/// Returns the names bound by a pattern, including any repeats.
fn bound_names(pattern: &Pattern) -> Vec<Identifier> {
    match pattern {
        Pattern::Identifier(identifier) => vec![identifier.clone()],
        Pattern::Wildcard => Vec::new(),
    }
}

/// Returns the names bound by patterns in the function's parameters, including any repeats.
fn parameter_names(function: &Function) -> Vec<Identifier> {
    let mut names = Vec::new();
    for parameter in &function.parameters {
        if let FunctionParamKind::Regular {
            pattern: Some(pattern),
            ..
        } = &parameter.kind
        {
            names.extend(bound_names(pattern));
        }
    }
    names
}

impl Frame {
    /// Returns a frame with no entries.
    fn new() -> Self {
        Frame {
            entries: BTreeMap::new(),
        }
    }

    /// Adds an entry mapping `name` to `entity`.
    ///
    /// `checks_that_resolution_depends_on` rejects programs that would give a frame two entries
    /// for the same name. If a program hasn't passed those checks, the last entry for a repeated
    /// name wins.
    fn add_entry(&mut self, name: Identifier, entity: Entity) {
        self.entries.insert(name, entity);
    }
}

/// Returns the first name in `names` that repeats an earlier one, if there is such a name.
fn find_repeat(names: impl IntoIterator<Item = Identifier>) -> Option<Identifier> {
    let mut seen = BTreeSet::new();
    names.into_iter().find(|name| !seen.insert(name.clone()))
}

/// Returns a rejection built using `message` if `names` repeats a name.
fn reject_repeats(
    names: Vec<Identifier>,
    message: impl Fn(&Identifier) -> String,
) -> Result<(), Rejection> {
    match find_repeat(names) {
        Some(name) => Err(Rejection::invalid(message(&name))),
        None => Ok(()),
    }
}
