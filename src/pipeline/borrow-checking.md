# Borrow Checking?

None of the desugarings so far mention borrow-checking. And the reason why is that unlike
type-checking, borrow-checking really is just a "check". In particular, the result of
borrow-checking must not influence runtime behavior.

So there does not need to be a desugaring related to borrow-checking.

Ideally we'd run borrow-checking at the end of all the desugarings; as presented though we lose
a bit of information while desugaring, in particular around matches, so borrow-checking here would
accept more code than we'd like. I leave this to later work.
