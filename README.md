# tiny-ott

A small dependent type checker implementing Observational Type Theory in the Pujet-Tabareau style. Around 1700 lines of Rust. The point of the project is the equality fragment: `Eq A x y` reduces by recursion on `A`, so function extensionality, decidable equality on `Nat` and `Bool`, and proof irrelevance for equality are all definitional rather than postulated.

## What's inside

- Bidirectional elaborator for a Martin-Lof core: dependent functions, pairs, `Nat`, `Bool`, `Unit`, `Empty`, a single universe `Type` (type-in-type).
- Normalization by evaluation in the Coquand style. Values carry closures with three special shapes used by OTT.
- OTT equality machinery. `Eq A x y` reduces structurally:
  - `Eq Nat` and `Eq Bool` reduce to `Unit` or `Empty` on closed canonical inputs, so decidability holds definitionally.
  - `Eq ((x : A) -> B x) f g` reduces to the pointwise `(x : A) -> Eq (B x) (f x) (g x)`, so function extensionality is the identity.
  - `Eq (A * B) (a, b) (a', b')` reduces to `Eq A a a' * Eq B b b'`.
  - `Eq Type A B` reduces to `Unit` when `A` and `B` are convertible.
  - `Eq (Eq A x y) p q` reduces to `Unit` (proof irrelevance for equality).
- `coe : (A B : Type) -> Eq Type A B -> A -> B` reduces to the identity when `A` and `B` are convertible.
- `refl` is elaborated structurally against the expected type and disappears into the canonical inhabitant of whatever the reduction lands on.
- Snapshot tests via insta over 13 worked example files under `tests/cases/`.

## Build and test

```bash
cargo build
cargo test
```

To accept fresh snapshots after a deliberate change:

```bash
INSTA_UPDATE=always cargo test --test snapshots
```

## Run the demo

```bash
cargo run --example demo
```

## Examples

The examples below all typecheck under the OTT reduction rules and would not under intensional MLTT without `funext` as an axiom.

Function extensionality is the identity function:

```
def funext :
    (A : Type) -> (B : A -> Type) -> (f g : (x : A) -> B x) ->
    ((x : A) -> Eq (B x) (f x) (g x)) -> Eq ((x : A) -> B x) f g
  := \A B f g h => h
```

Reflexivity at concrete naturals reduces to `tt`:

```
def refl-two : Eq Nat 2 2 := refl
```

Disequal naturals give `Empty`, so the absurdity is direct:

```
def zero-neq-one : Eq Nat zero (suc zero) -> Empty := \p => p
```

Proof irrelevance for equality types:

```
def eq-irrelevance :
    (A : Type) -> (x y : A) -> (p q : Eq A x y) -> Eq (Eq A x y) p q
  := \A x y p q => tt
```

Coercion across a definitionally equal type collapses to the identity:

```
def coe-id : (A : Type) -> A -> A := \A x => coe A A tt x
```

Pair equality decomposes into component equalities, in both directions:

```
def pair-eq-inv :
    (A : Type) -> (B : Type) -> (a1 a2 : A) -> (b1 b2 : B) ->
    Eq (A * B) (a1, b1) (a2, b2) ->
    Eq A a1 a2 * Eq B b1 b2
  := \A B a1 a2 b1 b2 p => p
```

See `tests/cases/` for the full set, including `natrec`, `boolrec`, eta on `->`, nested types, and `let`-bindings.

## Reading order

1. `src/syntax.rs` for the surface (`Raw`) and core (`Tm`) AST.
2. `src/value.rs` for the `Val` domain and the three `Closure` shapes:
   - `Body` is the usual environment + term.
   - `EqPi` is the closure underneath `Eq` at a Pi type, which performs the pointwise reduction on application.
   - `Const` ignores its argument; used to lift a non-dependent equality under a Sigma codomain.
3. `src/eval.rs` for NbE plus the OTT reductions in `eq_val` and `coe_val`.
4. `src/elab.rs` for the bidirectional checker. The `check_refl` function is where `refl` is elaborated structurally against the expected type.
5. `src/parser.rs` for the hand-written lexer and recursive descent parser.
6. `src/driver.rs` for the top-level pipeline.

## References

- Pujet and Tabareau, "Observational Equality: Now for Good," POPL 2022.
- Pujet and Tabareau, "Impredicative Observational Equality," POPL 2023.
- Kovacs, the `elaboration-zoo` repository, for the NbE and bidirectional elaboration architecture.

## Limitations

Type-in-type universe (no universe levels). No metavariables or implicit argument insertion: every binder is explicit. No inductive types beyond `Nat` and `Bool`. The equality fragment is the only thing this project tries to do well.

## License

MIT
