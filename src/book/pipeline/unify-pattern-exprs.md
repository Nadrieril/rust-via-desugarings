# Desugaring Pattern Expressions

This step transforms all the expressions that involve patterns into either `match` or `if let` expressions.

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
  {
    let $pat = $expr else { $else };
    $body
  }
  // becomes
  if let $pat = $expr {
      $body
  } else {
      $else
  }
  ```
  Where we added a block to make `let else` the first statement of its block.

- Destructuring assignment
  ```rust
  $pat = $expr;
  // becomes
  if let $pat_ = $expr {
    x1 = x1_;
    ..
    xn = x1_;
  } else {
    unsafe { core::hint::unreachable_unchecked() }
  }
  ```
  where each of the `xi` is one of the variables bound in `$pat`, and `$pat_` is the outcome of
  changing `$pat` to use the variables `xi_` instead;

- Matches

  ```rust
  match $expr {
      $pat if $guard => $arm,
      ..
  }
  // stays unchanged
  ```

After this step, the only expressions involving patterns are `match` and `if let else` expressions.
