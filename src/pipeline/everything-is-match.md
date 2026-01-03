# Desugaring Pattern Expressions

This steps transforms all the expressions that involve patterns into `match` or `if let` expressions.

Patterns can show up in the following locations. In what follows, `$pat` is a pattern that's not
a simple binding.

- Function parameters

  ```rust
  fn f($pat: T) { $body }
  // becomes
  fn f(tmp: T) { let $pat = tmp; $body }
  ```

- If let
  ```rust
  if let $pat = $expr { $then }
  // becomes
  if let $pat = $expr { $then } else {}
  ```

- If let else

  ```rust
  if let $pat = $expr { $then } else { $else }
  // stays unchanged
  ```

- let

  ```rust
  let $pat = $expr;
  // becomes
  let $pat = $expr else { unsafe { core::hint::unreachable_unchecked() } };
  ```

- let else

  ```rust
  let $pat = $expr else { $else };
  // becomes
  let x1;
  ..
  let xn;
  if let $pat = $expr {
    x1 = ..;
    ..
    xn = ..;
  } else {
    $else
  }
  ```
  where each of the `xi` is one of the variables bound in `$pat`;

- Destructuring assignment
  ```rust
  $pat = $expr;
  // becomes
  if let $pat = $expr {
    x1 = ..;
    ..
    xn = ..;
  } else {
    $else
  }
  ```
  where each of the `xi` is one of the variables bound in `$pat`;

- Matches

  ```rust
  match $expr {
      $pat if $guard => $arm,
      ..
  }
  // stays unchanged
  ```

After this step, the only expressions involving patterns are `match` and `if let else` expressions.
