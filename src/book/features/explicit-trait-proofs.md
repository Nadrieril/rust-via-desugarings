# Explicit Trait Proofs

The final language should be fully explicit, with nothing to "infer"; this includes trait facts
about which method to call or what the value of each associated type is.
I propose here a whole bunch of made-up syntax that makes it possible to explicitly name and pass
around trait facts.

1. Named impls;

    First, we need to be able to name a specific trait impl without going through trait solving:
    ```rust,example
    trait Trait {
        type Assoc;
    }

    // This gives a name to the impl.
    impl<T> "the_impl" Trait for T {
        type Assoc = Box<T>;
    }

    // This refers to the impl directly without going through trait solving.
    fn foo<T>(x: the_impl<T>::Assoc) { ... }
    ```

2. Named `where` predicates;

    We need to be able to refer to local clauses:
    ```rust,example
    fn clone_my_thing<T>(x: &T) -> (T, T)
    where
        // This gives a name to the `T: Clone` clause
        t_clone: (T: Clone),
    {
        // so that we can refer to it directly
        (t_clone::clone(x), t_clone::clone(x))
    }
    ```

3. Explicit proof passing

    When mentioning an item, we need to be able to tell it which trait facts to use for each
    predicate it has:
    ```rust,example
    // The impl found in the standard library.
    impl<T> "impl_clone_for_box" Clone for Box<T>
    where
        T: Clone,
    { ... }

    impl "impl_clone_for_u32" Clone for u32 { ... }

    fn clone_my_box(x: &Box<u32>) -> Box<u32> {
        // The square brackets after the generics means "pass these proofs as
        // proof of the corresponding trait clauses". Here the impl takes one
        // proof, of `u32: Clone`.
        (impl_clone_for_box::<u32>[impl_clone_for_u32])::clone(x)
    }
    ```

    (This is clearly ambiguous with indexing syntax. This is not a serious syntax proposal, I just need
    something to illustrate.)

3. Equality predicates;

    After trait clauses, the second most important kind of trait fact is associated type equality
    predicates, as in `T: Trait<Assoc = u32>`. To express them, we add standalone `T1 = T2` predicates:

    ```rust,example
    fn foo<T>()
    where
        T: Trait,
        <T as Trait>::Assoc = u32,
    { ... }
    // or, after trait solving:
    fn foo<T>()
    where
        t_trait: (T: Trait),
        // This refers to `t_trait` directly
        t_trait_eq: (t_trait::Assoc = u32),
    { ... }
    ```

    To refer to `foo` explicitly, you now need proofs for both facts: `foo<T>[trait_proof,
    eq_proof]`.

    The main way to prove an equality predicate is when it is trivial[^3]; the proof of `T = T` is
    spelled `refl<T>`:

    [^3]: By this I mean "only requires normalizing each side, with no extra trait solving or
    anything".

    ```rust,example
    impl "impl_unit" Trait for () {
        type Assoc = u32;
    }

    // Now we can call `foo` like:
    foo::<()>[impl_unit, refl<u32>]()
    ```

    The other ways are:
    - symmetry: given `eq: (T = U)`, `eq::symmetry` is a proof of `U = T`;
    - transitivity: given `eq1: (T = U)` and `eq2: (U = V)`, `eq1::transitivity<V>[eq2]` is a proof
      of `T = V`;
    - application: given `eq: (T = U)`, `eq::apply<Foo<T>, Foo<U>>` is a proof of `Foo<T>
      = Foo<U>`[^1].

    Finally, given `eq: (T = U)` and `x: T`, `eq::transmute(x)` is a term of type `U`. This is safe
    because the rest of the system ensures we can only get `T = U` if the two types really will be
    identical:
    ```rust,example
    fn foo<T>(x: t_trait::Assoc) -> u32
    where
        t_trait: (T: Trait),
        t_trait_eq: (t_trait::Assoc = u32),
    {
        t_trait_eq::transmute(x)
    }
    ```

    Note that this isn't a function call: you can use it on a place, e.g. `&mut
    t_trait_eq::transmute(x)` would mutably borrow `x` but knowing it actually (also) has type
    `u32`.

    [^1]: I couldn't find a good syntax that fit within Rust for what I really want: application to
    a type-level lambda. So instead this is a bit dodgy[^2] but hopefully good enough for simple examples.

    [^2]: The dodgy part is that this requires a little bit of inference to check that it's valid, since
    we must figure out which subterms to apply the equality to. Not the worst but not very principled.

2. Outlives predicates;

    The final kind of predicate is outlives predicates: `T: 'a` and `'a: 'b`.
    I haven't thought a lot about them yet; they at least have something like `refl` and
    `transitivity`, like type equalities.
    Borrow-checking will also need to be able to construct outlives proofs
    based on facts of the current function; I don't know what shape this might take.

2. Impl aliases;

    The final element is that we need to be able to construct [self-referential trait
    facts](https://nadrieril.github.io/blog/2026/05/14/when-can-traits-depend-on-themselves.html).

    To this purpose, I propose "impl aliases". These are impl items that don't participate in trait
    solving at all. They're only there to be referred to. The crucial part is that, like normal
    impls, they can refer to themselves. They otherwise work like a type alias, i.e. equivalent
    to their contents.

    ```rust,example
    // The impl found in the standard library.
    impl<T> "impl_clone_for_box" Clone for Box<T>
    where
        t_clone: (T: Clone),
    { ... }

    // A perfect derive example (in a world where this is legal).
    impl<T> "coind_impl_clone_for_list" Clone for List<T>
    where
        t_clone: (T: Clone),
        box_list_clone: (Box<List<T>>: Clone),
    { ... }

    // This would be added by the trait solver when needed. Something like this is
    // needed to be able to use `coind_impl_clone_for_list` in any way.
    // Notice the self-reference
    impl<T> "impl_clone_for_list" Clone for List<T>
    where
        t_clone: (T: Clone),
    = coind_impl_clone_for_list<T>[t_clone, impl_clone_for_box<T>[impl_clone_for_list<T>[t_clone]]]

    fn clone_list<T>(l: &List<T>) -> List<T>
    where
        t_clone: (T: Clone)
    {
        // The impl alias is used here.
        impl_clone_for_list<T>[t_clone]::clone(l)
    }
    ```
