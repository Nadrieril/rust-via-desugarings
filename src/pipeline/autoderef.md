# Autoderef

"Autoderef" refers the fact that postfix operations can work through
references by automatically inserting derefs.
Some such derefs were introduced in the previous section already.

In this step we desugar the remaining case: field access. In the expression `x.field`, if `x`
does not have a field `field` then we desugar it to `(*x).field` and try again, until it is no
longer legal to dereference the place.
