# TODO: Trait Desugarings

I would like trait solving to make explicit how it figured out every trait fact holds.
In that way, the final language would have "nothing left to infer".
See [the proposed syntax for that](../features/explicit-trait-proofs.md).

TODO: Add desugaring steps that make trait facts explicit:
- Desugar argument `impl Trait`;
- Make implied and implicit bounds explicit: `T: Sized` and `T: 'a` stuffs;
- Desugar `T: Trait<Assoc: OtherTrait<Assoc = u32>>` type stuff;
- Give a name to all input predicates;
- Somehow desugar return-position `impl Trait`?
- Pass proofs everywhere. In theory `(A, B)` becomes `(A, B)[impl_Sized_for_A]` etc, every `copy!`
  and use of a type equality needs a justification, etc.
