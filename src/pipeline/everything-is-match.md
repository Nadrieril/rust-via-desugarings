# Desugaring Patterns to Matches

This steps transforms all the expressions that involve patterns into `match` expressions. Now that
we've desugared temporaries, this is rather straightforward.

- Function parameters `fn f($pat: T) { ... }` desugar to `fn f(tmp: T) { let $pat = tmp; ... }`;
- `if let $pat = $expr { A }` first desugars to `if let $pat = $expr { A } else {}`;
- `if let $binding = $expr { A } else { B }` desugars to `let $binding = $expr; { A }`;
- `if let $pat = $expr { A } else { B }` desugars to `match $expr { $pat => { A }, _ => { B } }`;
- `let $pat = $expr;` first desugars to `let $pat = $expr else { unsafe { core::hint::unreachable_unchecked() } };`;
- `let $pat = $expr else { B };` turns into `let x1; let x2; ..` followed by `match $expr { $pat =>
  { x1 = ..; x2 = ..; ... }, _ => B, }` where each of the `xi` is one of the variables bound in
  `$pat`;
- Destructuring assignments `$pat = $expr;` are desugared just like `let $pat = expr;` except
  we don't declare the `let xi;` variables.

After this step, the only expressions involving patterns are `match` expressions.
