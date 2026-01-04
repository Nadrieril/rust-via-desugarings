# Borrow Checking?

None of the desugarings so far mention borrow-checking. And the reason why is that unlike
type-checking, borrow-checking really is just a "check". In particular, the result of
borrow-checking must not influence runtime behavior.

So there does not need to be a desugaring related to borrow-checking.

---

## Discussion

The question remains of when to run borrow-checking. Ideally we'd run it at the end of all the
desugarings; as presented though we lose some information while desugaring, in particular around
matches, so borrow-checking here would accept unsound code (see e.g. [Bound
Checks](bound-checks.md)),
and reject code that is accepted today (see note about slice patterns below).

If we wanted to have accurate borrow-checking, we'd need:
- Add fake borrows/fake reads of the places involved in a match;
- Add fake borrows/fake reads around bounds checks to reject `x[0][{x = &[]; 0}]` (see [Bound Checks](bound-checks.md));
- Add some false edges I don't recall where (I know MIR has some for loops and match guards but both
  of these are irrelevant for us);
- Support tracking some constant indexing, for the purpose of borrow-checking slice patterns.
