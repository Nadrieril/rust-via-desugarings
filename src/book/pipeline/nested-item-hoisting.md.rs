//@ # Nested-item hoisting
//@
//@ Every nested function is moved to crate level.
//@
//@ Single-segment path expressions which named these functions are rewritten to
//@ crate-relative form.
//@
//@ ## Preconditions
//@ - Every nested function has a name that isn't used for any other item or local.
//@ - No path expression in a nested function resolves to a local of an enclosing function.
//@ - Every path resolving to a crate-level item is crate-relative.
//@
//@ (These are promised by [Name Resolution](name-resolution.md.rs).)
//@
//@ ## After this step:
//@ - No functions are nested inside other items.
//@ - Path expressions which resolve to items are in crate-relative form.
//@ - Path expressions which resolve to locals are single identifiers.
//@ - All crate-relative path expressions directly name items.
//@ - Every item in the program has a distinct name.
//@
//@ TODO: We probably need to drop any `pub` on nested functions.
//@
//@ ## Discussion
//@
//@ The preconditions are used as follows:
//@
//@ > Every nested function has a name that isn't used for any other item or local.
//@
//@ This lets us assume every path expression naming the function resolves to that function.
//@
//@ > No path expression in a nested function resolves to a local of an enclosing function.
//@
//@ Otherwise hoisting a function could cause a program that shouldn't compile to compile.
//@
//@ > Every path resolving to a crate-level item is crate-relative.
//@
//@ This lets us promise that afterwards all path expressions which resolve to items are in
//@ crate-relative form.
//@
//@ Justification that this step preserves the program's meaning:
//@
//@ Path expressions in the hoisted function's body preserve their resolution because (the
//@ preconditions tell us) each of them is a crate-relative path, an unambiguous name, or one of the
//@ function's own locals. None of those change their resolution when the function is hoisted.
//@
//@ Path expressions elsewhere which resolve to the function still do so because we rewrote them to
//@ find it in its new location.
//@
//@ Path expressions elsewhere which didn't resolve to the function don't change their resolution
//@ because none of them named the function, because its name isn't used for any other item or
//@ local.
//@
//@ > The rest of this section is a work-in-progress experiment about making the book executable.
//@ >
//@ > For convenience, the hoisted functions are placed in the order of their `fn` lines in the
//@ > source.

use std::collections::BTreeSet; //#

use crate::CompilationError; //#

use crate::interactive_example; //#
use crate::language::*; //#

interactive_example! {
    hoist_nested_functions,
    fn greet() {}
    fn main() {
        crate::greet();
        main__helper();
        fn main__helper() {
            crate::greet();
            main__other();
            main__helper__deep();
            fn main__helper__deep() {}
        }
        fn main__other() {}
        {
            fn main__helper_0() {}
            main__helper_0();
        }
        let greet = true;
        crate::print(greet);
    }
}

pub fn hoist_nested_functions(program: &mut Program) -> Result<(), CompilationError> {
    // Copy each nested function to crate level, after the crate-level item which contains it.
    // The traversal is pre-order, so the copies are in the source order of their `fn` lines.
    let mut hoisted = BTreeSet::new();
    for item in std::mem::take(&mut program.items) {
        let mut nested = Vec::new();
        item.visit_all_infallible(|statement: &Statement| {
            if let Statement::Item(
                nested_item @ Item {
                    kind: ItemKind::Function(function),
                    ..
                },
            ) = statement
            {
                hoisted.insert(function.name.clone());
                nested.push(nested_item.clone());
            }
        });
        program.items.push(item);
        program.items.extend(nested);
    }

    // Remove the nested functions from their blocks (including the blocks in the copies)
    program.visit_all_mut_infallible(|block: &mut BlockExpression| {
        block.statements.retain(|statement| {
            !matches!(
                statement,
                Statement::Item(Item {
                    kind: ItemKind::Function(_),
                    ..
                })
            )
        });
    });

    // Rewrite path expressions which resolved to the hoisted functions
    program.visit_all_mut_infallible(|path_expr: &mut PathExpression| {
        if let PathExpression::SingleSegment(identifier) = path_expr
            && hoisted.contains(identifier)
        {
            *path_expr = PathExpression::CrateRelative(vec![std::mem::take(identifier)]);
        }
    });

    Ok(())
}
