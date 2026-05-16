# tiny-ott

My first basic attempt at an efficient Rust implementation of Observational Type Theory, based on trying to understand Pujet and Tabareau, [Observational Equality: Now for Good](https://dl.acm.org/doi/pdf/10.1145/3498693).

## Build

```bash
cargo build
```

## REPL

```bash
cargo run -- repl
```

```text
tiny-ott REPL. Type :? for help, :q to quit.
> def id : (A : Type) -> A -> A := \A x => x
def id
  : (A : Type) -> A -> A
  := \A => \x => x
> :t id Nat
id Nat : Nat -> Nat
> id Nat (suc zero)
suc zero
  : Nat
> :l examples/prelude.ott
> :t add 2 3
add 2 3 : Nat
> add 2 3
suc (suc (suc (suc (suc zero))))
  : Nat
> :q
```

| command     | effect                                 |
| ----------- | -------------------------------------- |
| `:t <expr>` | infer the type of `<expr>`             |
| `:l <file>` | load definitions from a file           |
| `:?`        | show help                              |
| `:q`        | quit                                   |
| `<decl>`    | run a `def` / `eval` / `check` decl    |
| `<expr>`    | evaluate to normal form and print type |

```bash
cargo run -- tests/cases/01_identity.ott
cargo run -- tests/cases/02_refl_nat.ott
cargo run -- tests/cases/03_refl_bool.ott
cargo run -- tests/cases/04_funext.ott
cargo run -- tests/cases/05_pair_eq.ott
cargo run -- tests/cases/06_coe_identity.ott
cargo run -- tests/cases/07_disequal_nat.ott
cargo run -- tests/cases/08_natrec.ott
cargo run -- tests/cases/09_boolrec.ott
cargo run -- tests/cases/10_proof_irrelevance.ott
cargo run -- tests/cases/11_eta_pi.ott
cargo run -- tests/cases/12_eq_compute.ott
cargo run -- tests/cases/13_let.ott
cargo run -- tests/cases/14_arith_compute.ott
cargo run -- tests/cases/15_bool_ops.ott
cargo run -- tests/cases/16_compose.ott
cargo run -- tests/cases/17_negation.ott
cargo run -- tests/cases/18_polymorphism.ott
cargo run -- tests/cases/19_factorial.ott
cargo run -- tests/cases/20_sigma_projections.ott
cargo run -- tests/cases/21_dep_sigma.ott
cargo run -- tests/cases/22_check_decl.ott
cargo run -- tests/cases/23_err_type_mismatch.ott
cargo run -- tests/cases/24_err_unbound.ott
cargo run -- tests/cases/25_err_refl_disequal.ott
```

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
