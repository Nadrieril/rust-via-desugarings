# Autoderef

"Autoderef" refers the fact that postfix operations can work through
references by automatically inserting dereferences `*$expr`.
Some such derefs were introduced in method resolution already.

In this step we desugar the remaining cases: field access and indexing.
In the expression `$expr.field`, if the type of `$expr` does not have a field `field` then we
desugar it to `(*$expr).field` and try again, until it is no longer legal to dereference the place.
The expression `$expr[$index]` also has this behavior.
