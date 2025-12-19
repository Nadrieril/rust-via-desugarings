# Borrow Checking?

None of the desugarings so far mention borrow-checking. And the reason why is that unlike
type-checking, borrow-checking really is just a "check". In particular, the result of
borrow-checking must not influence runtime behavior.

So there does not need to be a desugaring related to borrow-checking.
