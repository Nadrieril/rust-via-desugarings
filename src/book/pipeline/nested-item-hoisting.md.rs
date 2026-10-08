//@ # Nested-item hoisting
//@
//@ In this step, nested items are moved to crate level, and path expressions are rewritten as
//@ necessary.
//@
//@ (At present, functions are the only kind of item.)
//@
//@ ## Preconditions
//@ - Every path expression resolves.
//@ - Every nested function has a name that isn't used for any other item or local.
//@ - Every path expression resolving to a crate-level item is crate-relative.
//@ - Every single-segment path expression contains either
//@   - the name of a local introduced in the same function; or
//@   - the unambiguous name of a nested function.
//@ - No path expression in a nested function resolves to a local of an enclosing function.
//@
//@ (These are promised by [Name Resolution](name-resolution.md.rs).)
//@
//@ ## Hoisting
use crate::Rejection; //#
use crate::interactive_example; //#
use crate::language::*; //#
use std::collections::BTreeSet; //#
pub fn nested_item_hoisting(program: &mut Program) -> Result<(), Rejection> {
    //@ Each nested function is moved to crate level.
    //@
    //@ > Each nested function is placed immediately after the crate-level item that contained it.
    //@ > Functions hoisted from the same crate-level item are placed in the order of their `fn`
    //@ > lines in the source.
    //@
    //@ > The executable representation does this in two passes. The first pass copies each nested
    //@ > function to its new position (the statements are visited in pre-order, which gives the
    //@ > output order described above). These copies still contain any functions that were nested
    //@ > inside them.
    let mut hoisted_names = BTreeSet::new();
    let mut new_items = Vec::new();
    for item in &program.items {
        new_items.push(item.clone());
        item.visit_all_infallible(|statement: &Statement| {
            if let Statement::Item(nested_item) = statement {
                match &nested_item.kind {
                    ItemKind::Function(function) => {
                        hoisted_names.insert(function.name.clone());
                    }
                }
                new_items.push(nested_item.clone());
            }
        });
    }
    program.items = new_items;

    //@ > The second pass removes the nested items from every block, including the blocks in the
    //@ > copies.
    program.visit_all_mut_infallible(|block: &mut BlockExpression| {
        block.statements.retain(|statement| match statement {
            Statement::Item(_) => false,
            _ => true,
        });
    });

    //@ Single-segment path expressions that name a hoisted function are rewritten to
    //@ crate-relative form.
    program.visit_all_mut_infallible(|path_expr: &mut PathExpression| {
        if let PathExpression::SingleSegment(identifier) = path_expr
            && hoisted_names.contains(identifier)
        {
            *path_expr = PathExpression::CrateRelative(vec![identifier.clone()]);
        }
    });

    Ok(())
}

//@ ## After this step
//@ - No functions are nested inside other items.
//@ - Path expressions that resolve to items are in crate-relative form.
//@ - Path expressions that resolve to locals are in single-segment form.
//@ - All crate-relative path expressions name items.
//@ - No two items in the program have the same name.
//@
//@ TODO: We probably need to drop any `pub` on nested functions.
//@
//@ ## Discussion
//@
//@ Justification that this step preserves the program's meaning:
//@
//@ Rewriting all single-segment path expressions that contain the name of a nested function is the
//@ same as rewriting all single-segment path expressions that resolved to a nested function,
//@ because all path expressions resolved and nested functions have names that aren't used for any
//@ other item or local.
//@
//@ A crate-relative path expression doesn't change its meaning either because the function it's in
//@ is hoisted or because new crate-level items (with names distinct from existing crate-level
//@ items) are added.
//@
//@ A path expression that resolved to a nested function still does so because we rewrote it to
//@ find the function in its new location.
//@
//@ That leaves single-segment path expressions that resolve to a local that belongs to the
//@ function the expression is in. Hoisting the function, removing nested items, and adding new
//@ crate-level items don't affect their resolution.

interactive_example! {
    nested_item_hoisting,
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
