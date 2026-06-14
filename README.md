# cell-growth-sim

A simulation of single-cell growth and division under different **size-control
models**, with a Rust simulation engine and a Python (marimo) analysis layer.

The biological question: how do cells decide *when* to divide? Three canonical
strategies are implemented, plus a generalisation:

| Model        | Division rule                                  | Intuition                          |
|--------------|------------------------------------------------|------------------------------------|
| **Timer**    | divide after a fixed time `τ`                  | "wait a set interval"              |
| **Sizer**    | divide on reaching an absolute volume          | "grow to a target size"            |
| **Adder**    | divide after adding a fixed volume increment   | "add a constant amount, then split"|
| **AdderAlpha** | `Δv ≥ α·v_birth + v_c` — interpolates the above | α = 0 → adder, α = −1 → sizer    |

Cells grow exponentially (`V(t) = V_b · e^(r·t)`), divide with stochastic
partitioning (split fraction ~ `Normal(0.5, split_noise)`), and inherit their
mother's control model. Run-to-run reproducibility is guaranteed by a fixed RNG
seed.

## Repository layout

This is a **monorepo** — one git repository, two cooperating projects that
communicate by file handoff:

```
cell-growth-sim/
├── sim/            Rust simulation engine (Cargo project)
│   ├── Cargo.toml
│   └── src/        cell.rs · config.rs · simulation.rs · main.rs
├── analysis/       Python analysis & visualisation (uv project, marimo)
│   ├── pyproject.toml
│   └── notebooks/
├── data/           simulation output consumed by the analysis layer
│   └── events.json
├── LICENSE
└── README.md
```

Data flow:

```
  sim/  ──writes──▶  data/events.json  ──reads──▶  analysis/
 (Rust)                                            (marimo / Polars)
```

The two halves are intentionally decoupled: the simulation knows nothing about
the notebooks, and the notebooks just read the emitted JSON. Each division event
records `birth_volume`, `division_volume`, `added_volume`, `generation_time`,
`generation`, `daughter_volumes`, and `time`.

## Running the simulation (Rust)

Requires a recent Rust toolchain (edition 2024).

```bash
cd sim
cargo run            # runs the sim, writes ../data/events.json
cargo test           # unit tests for the cell model and config
```

Parameters (growth rate, noise, seed, …) live in `sim/src/config.rs`. Per-cell
division thresholds are *derived* from the fundamental parameters rather than set
directly — configure `r` and `V₀`, and the timer period / sizer target / adder
increment follow.

## Analysis (Python)

Requires [uv](https://docs.astral.sh/uv/) and Python 3.14+.

```bash
cd analysis
uv sync              # create the environment
uv run marimo edit   # open the notebook
```

The analysis layer uses **Polars** for data wrangling and **Altair** for plots,
reading `../data/events.json`.

## Status

Active learning project (Rust). The `AdderAlpha` model is implemented in the
cell logic but not yet wired into `Config`/`main` — see the `todo!` in
`sim/src/cell.rs`.
