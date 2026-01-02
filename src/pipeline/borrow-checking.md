# Borrow Checking?

None of the desugarings so far mention borrow-checking. And the reason why is that unlike
type-checking, borrow-checking really is just a "check". In particular, the result of
borrow-checking must not influence runtime behavior.

So there does not need to be a desugaring related to borrow-checking.

---

## Discussion

The question remains of when to run borrow-checking. Ideally we'd run it at the end of all the
desugarings; as presented though we lose some information while desugaring, in particular around
matches, so borrow-checking here would accept more code than we'd like.

If we wanted to have accurate borrow-checking, we'd need:
- Fake borrows/fake reads of the places involved in a match;
- Some shenanigans around bounds checks to reject `x[0][{x = &[]; 0}]`;
- Some false edges I don't recall where (I know MIR has some for loops and match guards but both of
  these are irrelevant for us);
- Information about the liveness of places on unwind probably;
- A bunch more things I forgot about.
