import marimo

__generated_with = "0.23.9"
app = marimo.App(width="medium")


@app.cell
def _():
    import io
    import subprocess

    import altair as alt
    import marimo as mo
    import polars as pl

    # Altair refuses >5000 rows by default; the exploded return map exceeds that,
    # so lift the guard. (We keep specs small by sampling — see the df_scatter cell.)
    _ = alt.data_transformers.disable_max_rows()
    return alt, io, mo, pl, subprocess


@app.cell(hide_code=True)
def _(mo):
    mo.md(r"""
    ## Simulating Timer, Sizer and Adder models
    Selecting a model below will run a simulated cell growth and division experiment. Growth is exponential and division time is determined by the respective model (ie time, size or added volume).
    """)
    return


@app.cell
def _(alt, io, pl, subprocess):
    # Find the release binary by walking up from the cwd, so it works no matter
    # which directory marimo was launched from.
    from pathlib import Path

    def _find_bin() -> str:
        for base in [Path.cwd(), *Path.cwd().parents]:
            cand = base / "sim" / "target" / "release" / "adder-model"
            if cand.exists():
                return str(cand)
        raise FileNotFoundError(
            "adder-model release binary not found — run `cargo build --release` in sim/"
        )

    BIN = _find_bin()

    # Stable colour per model + the empty-frame schema, so plots look consistent
    # no matter which models are selected (and so an empty plot still draws axes).
    MODEL_COLORS = alt.Scale(
        domain=["timer", "sizer", "adder"],
        range=["#4c78a8", "#e45756", "#54a24b"],
    )
    EVENT_SCHEMA = {
        "birth_volume": pl.Float64,
        "division_volume": pl.Float64,
        "added_volume": pl.Float64,
        "generation_time": pl.Float64,
        "generation": pl.Int64,
        "daughter_volumes": pl.List(pl.Float64),
        "time": pl.Float64,
        "model": pl.String,
    }

    def run_sim(model, n_max=3000, seed=42, split_noise=0.05, threshold_cv=0.1, alpha=0.0):
        """Run one model, parse stdout JSON into a Polars frame, tag the model.

        `alpha` only matters for model="adder-alpha"; the other models ignore it.
        Passed as `--alpha=<v>` (equals form) so negative values aren't mistaken
        for flags.
        """
        out = subprocess.run(
            [
                BIN,
                "--model", model,
                "--n-max", str(n_max),
                "--seed", str(seed),
                "--split-noise", str(split_noise),
                "--threshold-noise-cv", str(threshold_cv),
                f"--alpha={alpha}",
            ],
            capture_output=True,
            check=True,
        )
        return pl.read_json(io.BytesIO(out.stdout)).with_columns(model=pl.lit(model))

    return EVENT_SCHEMA, MODEL_COLORS, run_sim


@app.cell
def _(mo):
    # Controls. Model selector starts EMPTY → empty graph until the reader picks.
    models = mo.ui.multiselect(
        options=["timer", "sizer", "adder"],
        value=[],
        label="model(s) to run",
    )
    n_max = mo.ui.slider(500, 8000, value=3000, step=500, label="population (n_max)")
    split_noise = mo.ui.slider(0.0, 0.2, value=0.05, step=0.01, label="split noise σ")
    threshold_cv = mo.ui.slider(0.0, 0.3, value=0.1, step=0.01, label="threshold CV")
    seed = mo.ui.slider(1, 100, value=42, step=1, label="seed")

    mo.vstack([models, mo.md("---"), n_max, split_noise, threshold_cv, seed])
    return models, n_max, seed, split_noise, threshold_cv


@app.cell
def _(
    EVENT_SCHEMA,
    models,
    n_max,
    pl,
    run_sim,
    seed,
    split_noise,
    threshold_cv,
):
    # Run only the selected models; empty (typed) frame when nothing is chosen.
    if models.value:
        df = pl.concat(
            run_sim(
                m,
                n_max=n_max.value,
                seed=seed.value,
                split_noise=split_noise.value,
                threshold_cv=threshold_cv.value,
            )
            for m in models.value
        )
    else:
        df = pl.DataFrame(schema=EVENT_SCHEMA)
    return (df,)


@app.cell
def _(df):
    # Cap points per model for the SCATTER panels (A/B/C). Embedding every event
    # in each chart spec blows past marimo's output-size limit; a random sample of
    # ~1000/model shows the slope identically. Panel D uses the FULL data via Polars
    # aggregation, so its distribution stays exact.
    SCATTER_CAP = 1000
    df_scatter = (
        df.sample(fraction=1.0, shuffle=True, seed=0).group_by("model").head(SCATTER_CAP)
        if len(df)
        else df
    )
    return (df_scatter,)


@app.cell
def _(MODEL_COLORS, alt, df_scatter):
    # Panel A — the control law: added volume vs birth volume. slope = α.
    # Faint points + a per-model fitted line so the slope reads through the
    # overlap. The only panel that keeps the colour legend.
    base_a = alt.Chart(df_scatter.select("birth_volume", "added_volume", "model")).encode(
        x=alt.X("birth_volume:Q", scale=alt.Scale(domain=[0, 2.5]), title="birth volume  V_b"),
        y=alt.Y("added_volume:Q", scale=alt.Scale(domain=[0, 2.5]), title="added volume  ΔV"),
        color=alt.Color("model:N", scale=MODEL_COLORS, title="model"),
    )
    fit_a = base_a.transform_regression(
        "birth_volume", "added_volume", groupby=["model"]
    ).mark_line(size=3)
    panel_a = (base_a.mark_point(opacity=0.15, size=10) + fit_a).properties(
        width=300, height=240, title="A · ΔV vs V_b  (slope = α)"
    )
    return (panel_a,)


@app.cell
def _(MODEL_COLORS, alt, df_scatter, pl):
    # Panel B — return map: each daughter's birth volume vs the mother's.
    # slope = (1+α)/2 → 0 sizer · ½ adder · 1 timer (the homeostasis eigenvalue).
    # Fitted lines make the three slopes readable despite the overlapping clouds.
    returns = (
        df_scatter.select("birth_volume", "daughter_volumes", "model")
        .explode("daughter_volumes")
        .rename({"birth_volume": "mother_vb", "daughter_volumes": "daughter_vb"})
    )
    diagonal = (
        alt.Chart(pl.DataFrame({"v": [0.0, 4.0]}))
        .mark_line(color="#888", strokeDash=[4, 4])
        .encode(x="v:Q", y="v:Q")
    )
    base_b = alt.Chart(returns).encode(
        x=alt.X("mother_vb:Q", scale=alt.Scale(domain=[0, 2.5]), title="mother  V_b"),
        y=alt.Y("daughter_vb:Q", scale=alt.Scale(domain=[0, 2.5]), title="daughter  V_b"),
        color=alt.Color("model:N", scale=MODEL_COLORS, legend=None),
    )
    fit_b = base_b.transform_regression(
        "mother_vb", "daughter_vb", groupby=["model"]
    ).mark_line(size=3)
    panel_b = (diagonal + base_b.mark_point(opacity=0.15, size=10) + fit_b).properties(
        width=300, height=240, title=""
    )
    return (panel_b,)


@app.cell
def _(MODEL_COLORS, alt, df_scatter):
    # Panel C — generation time vs birth volume.
    # flat = timer (T = ln2/r); negative = sizer (steep) / adder (mild).
    # Linear fit is a trend indicator (the true T–V_b law is logarithmic).
    base_c = alt.Chart(df_scatter.select("birth_volume", "generation_time", "model")).encode(
        x=alt.X("birth_volume:Q", scale=alt.Scale(domain=[0, 2.5]), title="birth volume  V_b"),
        y=alt.Y("generation_time:Q", scale=alt.Scale(domain=[0, 2.5]), title="generation time  T"),
        color=alt.Color("model:N", scale=MODEL_COLORS, legend=None),
    )
    fit_c = base_c.transform_regression(
        "birth_volume", "generation_time", groupby=["model"]
    ).mark_line(size=3)
    panel_c = (base_c.mark_point(opacity=0.15, size=10) + fit_c).properties(
        width=300, height=240, title="C · T vs V_b"
    )
    return (panel_c,)


@app.cell
def _(MODEL_COLORS, alt, df, pl):
    # Panel D — steady-state birth-volume distribution (warm-up generations dropped).
    # Pre-binned in Polars → true counts and a tiny spec; overlaid via stack=None.
    BIN_W = 0.1
    hist = (
        df.filter(pl.col("generation") >= 4)
        .with_columns(((pl.col("birth_volume") / BIN_W).floor() * BIN_W).alias("vb"))
        .group_by("model", "vb")
        .len()
        .rename({"len": "count"})
        .with_columns((pl.col("vb") + BIN_W).alias("vb_end"))
    )
    panel_d = (
        alt.Chart(hist)
        .mark_bar(opacity=0.45)
        .encode(
            x=alt.X("vb:Q", bin="binned", scale=alt.Scale(domain=[0, 2.5]), title="birth volume  V_b"),
            x2="vb_end:Q",
            y=alt.Y("count:Q", stack=None, title="count"),
            color=alt.Color("model:N", scale=MODEL_COLORS, legend=None),
        )
        .properties(width=300, height=240, title="D · birth-volume distribution (steady state)")
    )
    return (panel_d,)


@app.cell
def _(panel_a, panel_b, panel_c, panel_d):
    # Compose the four panels into a 2×2. configure_* must sit on the OUTERMOST
    # (composed) chart, so the whole grid gets one consistent style.
    grid = (
        ((panel_a | panel_b) & (panel_c | panel_d))
        .configure_axis(labelFontSize=11, titleFontSize=12, gridOpacity=0.3)
        .configure_legend(labelFontSize=11, titleFontSize=12)
        .configure_view(stroke="transparent")
    )
    grid
    return


@app.cell
def _(mo):
    mo.md("""
    ---
    ## The α model — one dial from sizer to timer

    The three strategies are really *one family*. A cell divides once it has
    added **ΔV = α·V_b + V_c**, and the single parameter **α** interpolates
    between them: **−1 = sizer**, **0 = adder**, **+1 = timer**. Drag α and
    watch all four panels swing continuously between the regimes.
    """)
    return


@app.cell
def _(mo):
    alpha = mo.ui.slider(
        -1.0, 1.0, value=0.0, step=0.1,
        label="α   (−1 sizer · 0 adder · +1 timer)", show_value=True,
    )
    alpha
    return (alpha,)


@app.cell
def _(alpha, mo):
    av = alpha.value
    if av <= -0.5:
        regime = "sizer-like"
    elif av < 0:
        regime = "between sizer and adder"
    elif av == 0:
        regime = "a pure adder"
    elif av < 1:
        regime = "between adder and timer"
    else:
        regime = "a pure timer"
    mo.md(
        f"**α = {av:+.2f}** → expect ΔV-vs-V_b slope ≈ **{av:+.2f}**, "
        f"return-map slope ≈ **{(1 + av) / 2:.2f}** — *{regime}*."
    )
    return


@app.cell
def _(alpha, n_max, run_sim, seed, split_noise, threshold_cv):
    # One adder-alpha run at the chosen α (reuses the noise/seed/n_max sliders).
    df_alpha = run_sim(
        "adder-alpha",
        n_max=n_max.value,
        seed=seed.value,
        split_noise=split_noise.value,
        threshold_cv=threshold_cv.value,
        alpha=alpha.value,
    )
    df_alpha_s = df_alpha.sample(fraction=1.0, shuffle=True, seed=0).head(1000)
    return df_alpha, df_alpha_s


@app.cell
def _(alt, df_alpha, df_alpha_s, pl):
    # Same four panels as the comparison, but for the single adder-alpha run.
    # Wrapped in a _-prefixed helper so its locals don't collide with the
    # comparison cells' variable names (marimo's single-definition rule).
    def _alpha_figure():
        clr, fit = "#4c78a8", "#e45756"

        base_a = alt.Chart(df_alpha_s).encode(
            x=alt.X("birth_volume:Q", scale=alt.Scale(domain=[0, 2.5]), title="birth volume  V_b"),
            y=alt.Y("added_volume:Q", scale=alt.Scale(domain=[0, 2.5]), title="added volume  ΔV"),
        )
        pa = (
            base_a.mark_point(opacity=0.2, size=10, color=clr)
            + base_a.transform_regression("birth_volume", "added_volume").mark_line(size=3, color=fit)
        ).properties(width=300, height=240, title="A · ΔV vs V_b  (slope = α)")

        ret = (
            df_alpha_s.select("birth_volume", "daughter_volumes")
            .explode("daughter_volumes")
            .rename({"birth_volume": "mother_vb", "daughter_volumes": "daughter_vb"})
        )
        diag = (
            alt.Chart(pl.DataFrame({"v": [0.0, 4.0]}))
            .mark_line(color="#888", strokeDash=[4, 4])
            .encode(x="v:Q", y="v:Q")
        )
        base_b = alt.Chart(ret).encode(
            x=alt.X("mother_vb:Q", scale=alt.Scale(domain=[0, 2.5]), title="mother  V_b"),
            y=alt.Y("daughter_vb:Q", scale=alt.Scale(domain=[0, 2.5]), title="daughter  V_b"),
        )
        pb = (
            diag
            + base_b.mark_point(opacity=0.15, size=10, color=clr)
            + base_b.transform_regression("mother_vb", "daughter_vb").mark_line(size=3, color=fit)
        ).properties(width=300, height=240, title="B · return map  (slope = (1+α)/2)")

        base_c = alt.Chart(df_alpha_s).encode(
            x=alt.X("birth_volume:Q", scale=alt.Scale(domain=[0, 2.5]), title="birth volume  V_b"),
            y=alt.Y("generation_time:Q", scale=alt.Scale(domain=[0, 2.5]), title="generation time  T"),
        )
        pc = (
            base_c.mark_point(opacity=0.2, size=10, color=clr)
            + base_c.transform_regression("birth_volume", "generation_time").mark_line(size=3, color=fit)
        ).properties(width=300, height=240, title="C · T vs V_b")

        bw = 0.1
        hist = (
            df_alpha.filter(pl.col("generation") >= 4)
            .with_columns(((pl.col("birth_volume") / bw).floor() * bw).alias("vb"))
            .group_by("vb")
            .len()
            .rename({"len": "count"})
            .with_columns((pl.col("vb") + bw).alias("vb_end"))
        )
        pd_ = (
            alt.Chart(hist)
            .mark_bar(opacity=0.6, color=clr)
            .encode(
                x=alt.X("vb:Q", bin="binned", scale=alt.Scale(domain=[0, 2.5]), title="birth volume  V_b"),
                x2="vb_end:Q",
                y=alt.Y("count:Q", title="count"),
            )
            .properties(width=300, height=240, title="D · birth-volume distribution")
        )

        return (
            ((pa | pb) & (pc | pd_))
            .configure_axis(labelFontSize=11, titleFontSize=12, gridOpacity=0.3)
            .configure_view(stroke="transparent")
        )

    alpha_grid = _alpha_figure()
    alpha_grid
    return


if __name__ == "__main__":
    app.run()
