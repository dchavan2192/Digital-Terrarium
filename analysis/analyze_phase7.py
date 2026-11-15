"""Phase 7 ON/OFF ablation analysis for Terrarium.

Answers the four "Planned experiments" questions from docs/PROGRESS.md:

  1. OEE        — does D(t) (mean_cumulative_activity) keep growing in the baseline?
  2. Niche      — do niche-constructing populations weather droughts better?
  3. Epigenetics— do populations with epigenetics recover faster after droughts?
  4. Sex        — does sexual reproduction speed adaptation vs asexual?

Data layout (written by `sim-runner sweep`):

  <root>/sex/sweep/stats_seed{N}_reproduction.sexual=1.000000_seed{N}.csv   baseline (all ON)
  <root>/sex/sweep/stats_seed{N}_reproduction.sexual=0.000000_seed{N}.csv   asexual
  <root>/niche/sweep/stats_seed{N}_niche_construction.enabled=0.000000_seed{N}.csv
  <root>/epi/sweep/stats_seed{N}_epigenetics.enabled=0.000000_seed{N}.csv

The baseline (sexual=1) runs are the shared control for all three comparisons.

Usage:
    analysis/.venv/bin/python analysis/analyze_phase7.py
    analysis/.venv/bin/python analysis/analyze_phase7.py --min-tick 100000   # partial data
    analysis/.venv/bin/python analysis/analyze_phase7.py --root output/phase7 \
        --fig-dir docs/images/phase7 --summary output/phase7/summary.json

Statistics: per-seed metric values, mean ± sd, Welch's t-test (unequal variances),
Cohen's d (ON − OFF, pooled sd), and Holm-adjusted p within each comparison family.
With n = 5 seeds per arm, only large effects (|d| ≳ 1.5) are detectable; null results
are reported as null.
"""

import argparse
import glob
import json
import math
import os
import re
import sys
import warnings

import numpy as np
import pandas as pd
import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt  # noqa: E402
import matplotlib.ticker as ticker  # noqa: E402

try:
    from scipy import stats as sps
    from scipy.optimize import curve_fit
    HAVE_SCIPY = True
except ImportError:  # pragma: no cover - venv has scipy, but stay robust
    HAVE_SCIPY = False

# ── Style (matches analysis/plot_oee.py and analysis/plot_readme.py) ─────────
plt.rcParams.update({
    "figure.facecolor": "#0d1117",
    "axes.facecolor": "#161b22",
    "axes.edgecolor": "#30363d",
    "axes.labelcolor": "#c9d1d9",
    "text.color": "#c9d1d9",
    "xtick.color": "#8b949e",
    "ytick.color": "#8b949e",
    "grid.color": "#21262d",
    "grid.alpha": 0.8,
    "font.family": "monospace",
    "font.size": 11,
    "legend.facecolor": "#161b22",
    "legend.edgecolor": "#30363d",
    "legend.labelcolor": "#c9d1d9",
})

ACCENT = "#58a6ff"
GREEN = "#3fb950"
ORANGE = "#d29922"
RED = "#f85149"
PURPLE = "#bc8cff"
PINK = "#f778ba"
CYAN = "#39d2c0"
MUTED = "#8b949e"
SEED_COLORS = [ACCENT, GREEN, ORANGE, PURPLE, PINK, CYAN, RED]

ON_COLOR = ACCENT
OFF_COLOR = ORANGE

# ── Experimental arms ────────────────────────────────────────────────────────
GROUPS = {
    "baseline": ("sex", "reproduction.sexual", 1.0),
    "asexual": ("sex", "reproduction.sexual", 0.0),
    "niche_off": ("niche", "niche_construction.enabled", 0.0),
    "epi_off": ("epi", "epigenetics.enabled", 0.0),
}

COMPARISONS = {
    "niche": ("baseline", "niche_off", "Niche construction ON vs OFF"),
    "epigenetics": ("baseline", "epi_off", "Epigenetics ON vs OFF"),
    "sex": ("baseline", "asexual", "Sexual vs asexual"),
}

KTICK = ticker.FuncFormatter(lambda x, _: f"{x / 1000:.0f}k")


def warn(msg):
    print(f"WARNING: {msg}", file=sys.stderr)


def smooth(series, window=20):
    return pd.Series(series).rolling(window, min_periods=1).mean().values


# ── Loading ──────────────────────────────────────────────────────────────────

def read_stats_csv(path):
    """Read a (possibly still-being-written) stats CSV. Returns None if unusable."""
    try:
        if os.path.getsize(path) == 0:
            return None
        df = pd.read_csv(path, on_bad_lines="skip")
    except (pd.errors.EmptyDataError, pd.errors.ParserError, OSError) as e:
        warn(f"could not read {path}: {e}")
        return None
    if "tick" not in df.columns:
        warn(f"{path}: no 'tick' column, skipping")
        return None
    df = df.apply(pd.to_numeric, errors="coerce")
    # A truncated final line (file mid-write) shows up as NaNs — drop it.
    df = df.dropna(subset=["tick", "population"]).sort_values("tick")
    df = df.drop_duplicates("tick", keep="last").reset_index(drop=True)
    # The tick-0 row is TickStats::default() (written before the first step):
    # all zeros, population 0. It is not real data — drop it.
    df = df[~((df["tick"] == 0) & (df["population"] == 0))].reset_index(drop=True)
    return df


def load_group(root, subdir, param, value):
    """Load all seeds of one experimental arm. Returns {seed: DataFrame}."""
    pattern = os.path.join(root, subdir, "sweep", f"stats_seed*_{param}={value:.6f}_seed*.csv")
    runs = {}
    for path in sorted(glob.glob(pattern)):
        m = re.search(r"_seed(\d+)\.csv$", path)
        if not m:
            continue
        seed = int(m.group(1))
        df = read_stats_csv(path)
        if df is None or len(df) < 3:
            warn(f"{os.path.relpath(path)}: empty or <3 rows (still running?), skipping")
            continue
        runs[seed] = df
    return runs


def filter_runs(groups, min_tick, align):
    """Drop runs shorter than min_tick (unless extinct) and optionally truncate
    all non-extinct runs to the common max tick so arms are compared fairly."""
    info = {}
    for g, runs in groups.items():
        for seed in list(runs):
            df = runs[seed]
            extinct = bool(df["population"].iloc[-1] == 0)
            last = int(df["tick"].iloc[-1])
            info[(g, seed)] = {"max_tick": last, "extinct": extinct}
            if not extinct and last < min_tick:
                warn(f"{g} seed {seed}: only reached tick {last} < --min-tick {min_tick}, skipping")
                info[(g, seed)]["used"] = False
                del runs[seed]
                continue
            info[(g, seed)]["used"] = True
    common = None
    if align:
        lasts = [int(df["tick"].iloc[-1]) for runs in groups.values() for df in runs.values()
                 if df["population"].iloc[-1] > 0]
        if lasts:
            common = min(lasts)
            for runs in groups.values():
                for seed, df in runs.items():
                    runs[seed] = df[df["tick"] <= common].reset_index(drop=True)
    return info, common


# ── Stats helpers ────────────────────────────────────────────────────────────

def finite(xs):
    a = np.asarray([x for x in xs if x is not None], dtype=float)
    return a[np.isfinite(a)]


def welch(a, b):
    """Welch's t-test + Cohen's d (a − b). Returns dict with NaNs when undefined."""
    a, b = finite(a), finite(b)
    out = {"t": math.nan, "p": math.nan, "d": math.nan}
    if len(a) < 2 or len(b) < 2:
        return out
    va, vb = a.var(ddof=1), b.var(ddof=1)
    pooled = math.sqrt((va + vb) / 2.0)
    diff = a.mean() - b.mean()
    if pooled > 0:
        out["d"] = diff / pooled
    se = math.sqrt(va / len(a) + vb / len(b))
    if se == 0:
        return out
    t = diff / se
    df = (va / len(a) + vb / len(b)) ** 2 / (
        (va / len(a)) ** 2 / (len(a) - 1) + (vb / len(b)) ** 2 / (len(b) - 1))
    out["t"] = t
    if HAVE_SCIPY:
        out["p"] = float(2 * sps.t.sf(abs(t), df))
    else:  # normal approximation (anti-conservative at small n)
        out["p"] = float(math.erfc(abs(t) / math.sqrt(2)))
    return out


def holm(pvals):
    """Holm-Bonferroni adjusted p-values (NaNs pass through)."""
    idx = [i for i, p in enumerate(pvals) if np.isfinite(p)]
    m = len(idx)
    adj = [math.nan] * len(pvals)
    running = 0.0
    for rank, i in enumerate(sorted(idx, key=lambda i: pvals[i])):
        running = max(running, min(1.0, (m - rank) * pvals[i]))
        adj[i] = running
    return adj


def describe(values):
    a = finite(values)
    return {
        "mean": float(a.mean()) if len(a) else math.nan,
        "sd": float(a.std(ddof=1)) if len(a) > 1 else math.nan,
        "n": int(len(a)),
    }


def late(df, frac=0.2):
    """Rows in the final `frac` of the run (by tick)."""
    t0, t1 = df["tick"].iloc[0], df["tick"].iloc[-1]
    return df[df["tick"] >= t1 - frac * (t1 - t0)]


def slope(x, y):
    x, y = np.asarray(x, float), np.asarray(y, float)
    ok = np.isfinite(x) & np.isfinite(y)
    if ok.sum() < 3:
        return math.nan, math.nan
    if HAVE_SCIPY:
        r = sps.linregress(x[ok], y[ok])
        return float(r.slope), float(r.pvalue)
    a, _ = np.polyfit(x[ok], y[ok], 1)
    return float(a), math.nan


def log_interval(df):
    return float(np.median(np.diff(df["tick"].values))) if len(df) > 1 else 1.0


def col(df, name):
    if name in df.columns:
        return df[name].values.astype(float)
    return np.full(len(df), np.nan)


# ── Q1: OEE / D(t) ───────────────────────────────────────────────────────────

def _fit_quality(y, yhat, k):
    resid = y - yhat
    ss_res = float(np.sum(resid ** 2))
    ss_tot = float(np.sum((y - y.mean()) ** 2))
    n = len(y)
    r2 = 1 - ss_res / ss_tot if ss_tot > 0 else math.nan
    aic = n * math.log(ss_res / n + 1e-300) + 2 * k
    return r2, aic


def fit_dt(df, burn_in_frac):
    """Fit log, linear and saturating models to D(t)."""
    t = df["tick"].values.astype(float)
    d = col(df, "mean_cumulative_activity")
    t_end = t[-1]
    keep = (t > max(burn_in_frac * t_end, 0)) & np.isfinite(d)
    t, d = t[keep], d[keep]
    res = {"n_points": int(len(t))}
    if len(t) < 6:
        return res

    # Logarithmic: D = a ln t + b
    a, b = np.polyfit(np.log(t), d, 1)
    r2, aic = _fit_quality(d, a * np.log(t) + b, 2)
    res["log"] = {"a": float(a), "b": float(b), "r2": r2, "aic": aic}

    # Linear: D = a t + b
    a, b = np.polyfit(t, d, 1)
    r2, aic = _fit_quality(d, a * t + b, 2)
    res["linear"] = {"a": float(a), "b": float(b), "r2": r2, "aic": aic}

    # Saturating: D = c + A (1 − exp(−t/τ))
    res["saturating"] = None
    if HAVE_SCIPY:
        def sat(tt, c, A, tau):
            return c + A * (1 - np.exp(-tt / tau))
        span = max(d.max() - d.min(), 1e-9)
        try:
            with warnings.catch_warnings():
                warnings.simplefilter("ignore")
                p, _ = curve_fit(sat, t, d, p0=[d[0], span * 1.5, t_end / 3],
                                 bounds=([-np.inf, 0, 1.0], [np.inf, np.inf, 1e3 * t_end]),
                                 maxfev=20000)
            r2, aic = _fit_quality(d, sat(t, *p), 3)
            res["saturating"] = {
                "c": float(p[0]), "A": float(p[1]), "tau": float(p[2]),
                "asymptote": float(p[0] + p[1]), "r2": r2, "aic": aic,
                # If τ ≫ run length the "saturating" fit is just a straight line.
                "tau_over_run": float(p[2] / t_end),
                "fraction_of_asymptote_reached": float((d[-1] - p[0]) / p[1]) if p[1] > 0 else math.nan,
            }
        except (RuntimeError, ValueError) as e:
            warn(f"saturating fit failed: {e}")

    models = {k: v for k, v in res.items() if k in ("log", "linear", "saturating") and v}
    res["best_by_aic"] = min(models, key=lambda k: models[k]["aic"]) if models else None

    # Is D(t) still increasing in the last 20% of the run?
    lt = late(df.loc[keep] if keep.any() else df)
    s, p = slope(lt["tick"], lt["mean_cumulative_activity"])
    s_all, _ = slope(t, d)
    res["late_slope"] = s
    res["late_slope_p"] = p
    res["overall_slope"] = s_all
    res["late_to_overall_slope_ratio"] = s / s_all if s_all and np.isfinite(s_all) else math.nan
    res["still_increasing"] = bool(np.isfinite(s) and s > 0 and (not np.isfinite(p) or p < 0.05))
    res["final_D"] = float(d[-1])
    return res


def analyze_oee(groups, args):
    out = {"per_seed": {}, "summary": {}}
    runs = groups["baseline"]
    if not runs:
        warn("OEE: no baseline runs available")
        return out
    for seed, df in sorted(runs.items()):
        out["per_seed"][seed] = fit_dt(df, args.burn_in)
    ps = list(out["per_seed"].values())

    def agg(path):
        vals = []
        for r in ps:
            v = r
            for k in path:
                v = v.get(k) if isinstance(v, dict) else None
            vals.append(v)
        return describe(vals)

    out["summary"] = {
        "log_a": agg(["log", "a"]),
        "log_r2": agg(["log", "r2"]),
        "linear_a": agg(["linear", "a"]),
        "linear_r2": agg(["linear", "r2"]),
        "sat_tau_over_run": agg(["saturating", "tau_over_run"]),
        "sat_r2": agg(["saturating", "r2"]),
        "late_slope": agg(["late_slope"]),
        "late_to_overall_slope_ratio": agg(["late_to_overall_slope_ratio"]),
        "n_still_increasing": int(sum(r.get("still_increasing", False) for r in ps)),
        "n_seeds": len(ps),
        "best_model_counts": {m: sum(r.get("best_by_aic") == m for r in ps)
                              for m in ("log", "linear", "saturating")},
    }
    return out


def plot_oee(groups, oee, fig_dir):
    runs = groups["baseline"]
    if not runs:
        return
    fig, axes = plt.subplots(2, 2, figsize=(14, 9))
    fig.suptitle("Phase 7 — Open-Ended Evolution: D(t) in baseline (all features ON)",
                 fontsize=14, fontweight="bold", color="#c9d1d9")
    ax_d, ax_log, ax_nov, ax_act = axes.flat

    for i, (seed, df) in enumerate(sorted(runs.items())):
        c = SEED_COLORS[i % len(SEED_COLORS)]
        t = df["tick"].values.astype(float)
        d = col(df, "mean_cumulative_activity")
        ax_d.plot(t, d, color=c, lw=1.2, label=f"seed {seed}")
        pos = t > 0
        ax_log.plot(t[pos], d[pos], color=c, lw=1.0, alpha=0.8)
        fit = oee["per_seed"].get(seed, {})
        if fit.get("log"):
            tt = np.linspace(t[pos].min(), t.max(), 200)
            ax_log.plot(tt, fit["log"]["a"] * np.log(tt) + fit["log"]["b"],
                        color=c, lw=1, ls="--", alpha=0.7)
        ax_nov.plot(t, smooth(col(df, "novel_traits")), color=c, lw=1.0, alpha=0.85)
        ax_act.plot(t, smooth(col(df, "active_traits")), color=c, lw=1.0, alpha=0.85)

    # Late-window shading
    t_end = max(df["tick"].iloc[-1] for df in runs.values())
    ax_d.axvspan(0.8 * t_end, t_end, color=MUTED, alpha=0.08, label="last 20%")

    s = oee["summary"]
    txt = (f"still increasing in last 20%: {s.get('n_still_increasing', 0)}/{s.get('n_seeds', 0)} seeds\n"
           f"best fit (AIC): {s.get('best_model_counts', {})}\n"
           f"late/overall slope: {s.get('late_to_overall_slope_ratio', {}).get('mean', math.nan):.2f}")
    ax_d.annotate(txt, xy=(0.02, 0.97), xycoords="axes fraction", fontsize=9, color=MUTED,
                  va="top", bbox=dict(boxstyle="round,pad=0.3", fc="#161b22", ec="#30363d"))
    ax_d.set_ylabel("D(t) — mean cumulative activity")
    ax_d.set_title("D(t) per seed")
    ax_d.legend(loc="lower right", fontsize=8)

    ax_log.set_xscale("log")
    ax_log.set_title("D(t), log-x  (dashed = a·ln t + b fit)")
    ax_log.set_ylabel("D(t)")

    ax_nov.set_title("Novel topologies per window (smoothed)")
    ax_nov.set_ylabel("novel_traits")
    ax_act.set_title("Distinct live topologies (smoothed)")
    ax_act.set_ylabel("active_traits")

    for ax in axes.flat:
        ax.grid(True, alpha=0.3)
        ax.set_xlabel("Tick")
        if ax is not ax_log:
            ax.xaxis.set_major_formatter(KTICK)
    fig.tight_layout()
    save(fig, fig_dir, "oee_dt.png")


# ── Drought episode helpers (Q2/Q3) ──────────────────────────────────────────

def drought_episodes(df):
    """Contiguous logged rows with active_droughts > 0 → list of (onset_tick, end_tick).
    end_tick is the first logged tick where active_droughts is back to 0 (None if the
    run ends in drought)."""
    t = df["tick"].values
    a = col(df, "active_droughts")
    on = np.nan_to_num(a) > 0
    eps = []
    i = 0
    while i < len(on):
        if on[i]:
            j = i
            while j < len(on) and on[j]:
                j += 1
            eps.append((int(t[i]), int(t[j]) if j < len(on) else None))
            i = j
        else:
            i += 1
    return eps


def window_mean(df, column, t0, t1):
    m = (df["tick"] >= t0) & (df["tick"] < t1)
    v = df.loc[m, column].values.astype(float) if column in df.columns else np.array([])
    return float(np.nanmean(v)) if len(v) else math.nan


def event_trace(df, column, t_event, ref, pre, post, step):
    """Series of column/ref on a relative grid [-pre, post] around t_event."""
    grid = np.arange(-pre, post + step, step)
    t = df["tick"].values.astype(float) - t_event
    y = col(df, column)
    if not np.isfinite(ref) or ref == 0 or t.min() > -pre or t.max() < post:
        return grid, None
    return grid, np.interp(grid, t, y) / ref


def drought_metrics(df, args):
    """Per-run drought-response metrics."""
    out = {}
    step = log_interval(df)
    dr = np.nan_to_num(col(df, "active_droughts")) > 0
    out["drought_time_fraction"] = float(dr.mean())
    for c in ("population", "mean_energy"):
        dm = float(np.nanmean(col(df, c)[dr])) if dr.any() else math.nan
        nm = float(np.nanmean(col(df, c)[~dr])) if (~dr).any() else math.nan
        out[f"{c}_in_drought"] = dm
        out[f"{c}_no_drought"] = nm
        out[f"{c}_drought_ratio"] = dm / nm if nm else math.nan

    eps = drought_episodes(df)
    out["n_episodes"] = len(eps)
    dips = {"population": [], "mean_energy": []}
    recov = {"population": [], "mean_energy": []}
    rec_time = []
    traces_on = {"population": [], "mean_energy": []}
    traces_off = {"population": [], "mean_energy": []}
    t_first = df["tick"].iloc[0]
    for k, (onset, end) in enumerate(eps):
        if onset - args.pre_window < t_first + args.burn_in * df["tick"].iloc[-1]:
            continue  # skip burn-in / no pre-window
        prev_end = eps[k - 1][1] if k > 0 else None
        pre_lo = onset - args.pre_window
        if prev_end is not None and prev_end > pre_lo:
            pre_lo = prev_end  # don't let a previous drought contaminate the reference
        if onset - pre_lo < step * 2:
            continue
        for c in dips:
            ref = window_mean(df, c, pre_lo, onset)
            m = (df["tick"] >= onset) & (df["tick"] <= onset + args.dip_window)
            if m.any() and np.isfinite(ref) and ref:
                dips[c].append((ref - float(df.loc[m, c].min())) / ref)
            _, tr = event_trace(df, c, onset, ref, args.pre_window, args.dip_window, step)
            if tr is not None:
                traces_on[c].append(tr)
            # Post-drought recovery
            if end is None:
                continue
            next_onset = eps[k + 1][0] if k + 1 < len(eps) else math.inf
            hi = min(end + args.recovery_window, next_onset)
            if hi - end < args.recovery_window / 2:
                continue  # next drought starts too soon to measure recovery
            post = window_mean(df, c, end, hi)
            if np.isfinite(ref) and ref:
                recov[c].append(post / ref)
            _, tr = event_trace(df, c, end, ref, args.pre_window, args.recovery_window, step)
            if tr is not None:
                traces_off[c].append(tr)
            if c == "population" and np.isfinite(ref) and ref:
                m = (df["tick"] >= end) & (df["tick"] < hi)
                sub = df.loc[m]
                hit = sub[sub["population"] >= args.recovery_level * ref]
                # Censored at the window length if never recovered.
                rec_time.append(float(hit["tick"].iloc[0] - end) if len(hit) else float(hi - end))

    def m(x):
        return float(np.mean(x)) if x else math.nan

    out["n_onsets_used"] = len(dips["population"])
    out["n_recoveries_used"] = len(recov["population"])
    out["onset_dip_population"] = m(dips["population"])
    out["onset_dip_energy"] = m(dips["mean_energy"])
    out["post_drought_population_rel"] = m(recov["population"])
    out["post_drought_energy_rel"] = m(recov["mean_energy"])
    out["recovery_time"] = m(rec_time)
    out["_traces_onset"] = {c: np.mean(v, axis=0) if v else None for c, v in traces_on.items()}
    out["_traces_end"] = {c: np.mean(v, axis=0) if v else None for c, v in traces_off.items()}
    return out


# ── General / sex-vs-asex metrics ────────────────────────────────────────────

def general_metrics(df):
    out = {}
    lt = late(df)
    t = df["tick"].values.astype(float)
    span = t[-1] - t[0]
    out["final_tick"] = float(t[-1])
    out["extinct"] = float(df["population"].iloc[-1] == 0)
    out["final_max_generation"] = float(col(df, "max_generation")[-1])
    out["final_mean_generation"] = float(col(df, "mean_generation")[-1])
    out["generations_per_100k_ticks"] = out["final_max_generation"] / span * 1e5 if span else math.nan
    tb = col(df, "total_births")
    out["birth_rate"] = (tb[-1] - tb[0]) / span if span else math.nan
    ltb = col(lt, "total_births")
    lspan = lt["tick"].iloc[-1] - lt["tick"].iloc[0]
    out["late_birth_rate"] = (ltb[-1] - ltb[0]) / lspan if lspan else math.nan
    for c in ("population", "mean_energy", "species_count", "lineage_diversity",
              "mean_connections", "active_traits", "mean_trophic_level",
              "mean_construction_level", "mean_methylation"):
        out[f"late_{c}"] = float(np.nanmean(col(lt, c)))
    out["final_D"] = float(col(df, "mean_cumulative_activity")[-1])
    # Adaptation speed: trend in mean_energy across the first half (after the founder crash)
    half = df[df["tick"] <= t[0] + span / 2]
    out["energy_slope_first_half"] = slope(half["tick"], half["mean_energy"])[0]
    # Muller's ratchet proxy: a sustained decline in mean energy (fitness proxy) over
    # the second half of the run. Sex should prevent it; asex might show it.
    second = df[df["tick"] >= t[0] + span / 2]
    out["energy_slope_second_half"] = slope(second["tick"], second["mean_energy"])[0]
    sb, ab = col(df, "sexual_births"), col(df, "asexual_births")
    tot = np.nansum(sb) + np.nansum(ab)
    out["sexual_birth_fraction"] = float(np.nansum(sb) / tot) if tot else math.nan
    return out


METRICS = {
    "niche": [
        ("population_drought_ratio", "pop in drought / pop no drought", "higher = more robust"),
        ("mean_energy_drought_ratio", "energy in drought / no drought", "higher = more robust"),
        ("onset_dip_population", "pop dip after onset (frac)", "lower = more robust"),
        ("onset_dip_energy", "energy dip after onset (frac)", "lower = more robust"),
        ("population_in_drought", "mean pop during drought", ""),
        ("late_population", "late pop (last 20%)", ""),
        ("late_mean_energy", "late mean energy", ""),
        ("late_mean_construction_level", "late construction level", "sanity: 0 when OFF"),
    ],
    "epigenetics": [
        ("post_drought_population_rel", "post-drought pop / pre", "higher = faster recovery"),
        ("post_drought_energy_rel", "post-drought energy / pre", "higher = faster recovery"),
        ("recovery_time", "ticks to regain pop level", "lower = faster (censored)"),
        ("onset_dip_population", "pop dip after onset (frac)", ""),
        ("late_population", "late pop (last 20%)", ""),
        ("late_mean_energy", "late mean energy", ""),
        ("late_mean_methylation", "late methylation", "sanity: 0 when OFF"),
    ],
    "sex": [
        ("energy_slope_first_half", "d(energy)/dt first half", "adaptation speed"),
        ("energy_slope_second_half", "d(energy)/dt second half", "ratchet proxy (<0 = decline)"),
        ("late_mean_energy", "late mean energy", ""),
        ("birth_rate", "births / tick", ""),
        ("final_max_generation", "final max generation", ""),
        ("generations_per_100k_ticks", "max gen per 100k ticks", ""),
        ("late_species_count", "late species count", ""),
        ("late_lineage_diversity", "late lineage diversity", ""),
        ("late_mean_connections", "late mean connections", ""),
        ("late_active_traits", "late active topologies", ""),
        ("final_D", "final D(t)", ""),
        ("late_population", "late pop (last 20%)", ""),
        ("sexual_birth_fraction", "sexual birth fraction", "sanity: 0 when asexual"),
    ],
}


def compare(per_run, comp, args):
    on_g, off_g, title = COMPARISONS[comp]
    res = {"title": title, "on_group": on_g, "off_group": off_g, "metrics": {}}
    rows = []
    for key, label, note in METRICS[comp]:
        on = {s: m.get(key, math.nan) for s, m in per_run[on_g].items()}
        off = {s: m.get(key, math.nan) for s, m in per_run[off_g].items()}
        w = welch(list(on.values()), list(off.values()))
        entry = {"label": label, "note": note,
                 "on": {"per_seed": on, **describe(on.values())},
                 "off": {"per_seed": off, **describe(off.values())},
                 "welch_t": w["t"], "p": w["p"], "cohens_d": w["d"]}
        res["metrics"][key] = entry
        rows.append(entry)
    # Holm correction over the scientific metrics only (sanity checks excluded).
    family = [e for e in rows if not e["note"].startswith("sanity")]
    for entry, padj in zip(family, holm([e["p"] for e in family])):
        entry["p_holm"] = padj
    for e in rows:
        e.setdefault("p_holm", math.nan)
    return res


# ── Plots for comparisons ────────────────────────────────────────────────────

def save(fig, fig_dir, name):
    os.makedirs(fig_dir, exist_ok=True)
    path = os.path.join(fig_dir, name)
    fig.savefig(path, dpi=150, bbox_inches="tight", facecolor=fig.get_facecolor())
    plt.close(fig)
    print(f"  Saved {path}")


def band(ax, x, ys, color, label):
    ys = [y for y in ys if y is not None]
    if not ys:
        return
    arr = np.vstack(ys)
    mu = np.nanmean(arr, axis=0)
    ax.plot(x, mu, color=color, lw=1.5, label=f"{label} (n={len(ys)})")
    if len(ys) > 1:
        sd = np.nanstd(arr, axis=0, ddof=1)
        ax.fill_between(x, mu - sd, mu + sd, color=color, alpha=0.2)


def strip(ax, comp_res, key, on_label, off_label):
    m = comp_res["metrics"][key]
    rng = np.random.default_rng(0)
    for i, (arm, color, lab) in enumerate(((m["on"], ON_COLOR, on_label), (m["off"], OFF_COLOR, off_label))):
        vals = finite(arm["per_seed"].values())
        ax.scatter(i + rng.uniform(-0.08, 0.08, len(vals)), vals, color=color, s=30, zorder=3)
        if np.isfinite(arm["mean"]):
            ax.hlines(arm["mean"], i - 0.25, i + 0.25, color=color, lw=2)
    ax.set_xticks([0, 1])
    ax.set_xticklabels([on_label, off_label])
    ax.set_xlim(-0.6, 1.6)
    p, d = m["p"], m["cohens_d"]
    ax.set_title(f"{m['label']}\np={p:.3g}  d={d:.2f}", fontsize=9)
    ax.grid(True, alpha=0.3, axis="y")


def plot_drought_comparison(per_run, comp_res, comp, args, fig_dir):
    on_g, off_g = comp_res["on_group"], comp_res["off_group"]
    on_lab, off_lab = {"niche": ("niche ON", "niche OFF"),
                       "epigenetics": ("epi ON", "epi OFF")}[comp]
    trace_key = "_traces_onset" if comp == "niche" else "_traces_end"
    post = args.dip_window if comp == "niche" else args.recovery_window
    keys = (["onset_dip_population", "population_drought_ratio", "mean_energy_drought_ratio"]
            if comp == "niche" else
            ["post_drought_population_rel", "post_drought_energy_rel", "recovery_time"])
    step = None
    for g in (on_g, off_g):
        for m in per_run[g].values():
            step = step or m.get("_step")
    if step is None:
        return
    grid = np.arange(-args.pre_window, post + step, step)

    fig = plt.figure(figsize=(15, 8))
    gs = fig.add_gridspec(2, 3)
    what = "drought onset" if comp == "niche" else "drought end"
    fig.suptitle(f"Phase 7 — {comp_res['title']}: response aligned to {what}",
                 fontsize=14, fontweight="bold", color="#c9d1d9")
    for r, c in enumerate(("population", "mean_energy")):
        ax = fig.add_subplot(gs[r, :2])
        for g, color, lab in ((on_g, ON_COLOR, on_lab), (off_g, OFF_COLOR, off_lab)):
            ys = [m[trace_key][c] for m in per_run[g].values()
                  if m[trace_key][c] is not None and len(m[trace_key][c]) == len(grid)]
            band(ax, grid, ys, color, lab)
        ax.axvline(0, color=RED, ls="--", lw=1, alpha=0.7)
        ax.axhline(1, color="#30363d", ls=":", lw=1)
        ax.set_ylabel(f"{c} / pre-drought mean")
        ax.grid(True, alpha=0.3)
        if ax.get_legend_handles_labels()[0]:
            ax.legend(loc="lower left", fontsize=8)
        else:
            ax.text(0.5, 0.5, "no complete drought events yet", transform=ax.transAxes,
                    ha="center", color=MUTED)
        if r == 1:
            ax.set_xlabel(f"Ticks relative to {what}")
    for i, k in enumerate(keys[:2]):
        strip(fig.add_subplot(gs[i, 2]), comp_res, k, on_lab, off_lab)
    fig.tight_layout()
    save(fig, fig_dir, f"{comp}_drought.png")


def plot_sex(groups, comp_res, fig_dir):
    on, off = groups["baseline"], groups["asexual"]
    if not on and not off:
        return
    panels = [("mean_energy", "Mean energy"), ("max_generation", "Max generation"),
              ("species_count", "Species count"), ("lineage_diversity", "Lineage diversity"),
              ("mean_connections", "Mean connections"), ("_birth_rate", "Births / tick (rolling)")]
    fig, axes = plt.subplots(2, 3, figsize=(16, 8.5), sharex=True)
    fig.suptitle("Phase 7 — Sexual vs asexual reproduction (mean ± sd across seeds)",
                 fontsize=14, fontweight="bold", color="#c9d1d9")
    all_dfs = list(on.values()) + list(off.values())
    t_end = min(df["tick"].iloc[-1] for df in all_dfs)
    step = min(log_interval(df) for df in all_dfs)
    grid = np.arange(0, t_end + step, step)
    for ax, (c, title) in zip(axes.flat, panels):
        for runs, color, lab in ((on, ON_COLOR, "sexual"), (off, OFF_COLOR, "asexual")):
            ys = []
            for df in runs.values():
                t = df["tick"].values.astype(float)
                if c == "_birth_rate":
                    y = np.gradient(col(df, "total_births"), t)
                    y = smooth(y, 20)
                else:
                    y = smooth(col(df, c), 10)
                ys.append(np.interp(grid, t, y, right=np.nan))
            band(ax, grid, ys, color, lab)
        ax.set_title(title, fontsize=10)
        ax.grid(True, alpha=0.3)
        ax.xaxis.set_major_formatter(KTICK)
    axes.flat[0].legend(loc="lower right", fontsize=8)
    for ax in axes[1]:
        ax.set_xlabel("Tick")
    fig.tight_layout()
    save(fig, fig_dir, "sex_vs_asex.png")


def plot_effects(comparisons, fig_dir):
    rows = []
    for comp, res in comparisons.items():
        for key, m in res["metrics"].items():
            if np.isfinite(m["cohens_d"]) and not m["note"].startswith("sanity"):
                rows.append((f"{comp}: {m['label']}", m["cohens_d"], m["p"]))
    if not rows:
        return
    fig, ax = plt.subplots(figsize=(11, 0.38 * len(rows) + 1.5))
    y = np.arange(len(rows))[::-1]
    for yi, (lab, d, p) in zip(y, rows):
        color = GREEN if (np.isfinite(p) and p < 0.05) else MUTED
        dc = float(np.clip(d, -6, 6))
        ax.barh(yi, dc, color=color, alpha=0.8, height=0.6)
        ax.text(max(dc, 0) + 0.08, yi, f"p={p:.2g}", va="center", ha="left",
                fontsize=8, color=MUTED)
    ax.set_yticks(y)
    ax.set_yticklabels([r[0] for r in rows], fontsize=8)
    ax.axvline(0, color="#30363d", lw=1)
    for v in (-0.8, 0.8):
        ax.axvline(v, color="#30363d", ls=":", lw=1)
    ax.set_xlabel("Cohen's d (feature ON − OFF; clipped to ±6)   green = p<0.05 unadjusted")
    ax.set_title("Phase 7 — Effect sizes across all ablations", fontsize=13, fontweight="bold")
    ax.grid(True, alpha=0.3, axis="x")
    fig.tight_layout()
    save(fig, fig_dir, "effect_sizes.png")


# ── Reporting ────────────────────────────────────────────────────────────────

def fmt(x, prec=4):
    if x is None or not np.isfinite(x):
        return "—"
    if x != 0 and (abs(x) < 1e-3 or abs(x) >= 1e5):
        return f"{x:.2e}"
    return f"{x:.{prec}g}"


def print_report(oee, comparisons, run_info, common_tick):
    print("\n# Phase 7 ablation results\n")
    if common_tick is not None:
        print(f"_All non-extinct runs truncated to common tick {common_tick}._\n")
    print("## Runs\n")
    print("| group | seed | max tick | extinct | used |")
    print("|---|---|---|---|---|")
    for (g, s), i in sorted(run_info.items()):
        print(f"| {g} | {s} | {i['max_tick']} | {i['extinct']} | {i.get('used', True)} |")

    print("\n## 1. OEE — D(t) in baseline\n")
    if oee["per_seed"]:
        print("| seed | log a | log R² | lin a | lin R² | sat τ/run | sat R² | best (AIC) | late slope | late/overall | still ↑ |")
        print("|---|---|---|---|---|---|---|---|---|---|---|")
        for seed, r in sorted(oee["per_seed"].items()):
            lg, ln, st = r.get("log") or {}, r.get("linear") or {}, r.get("saturating") or {}
            print(f"| {seed} | {fmt(lg.get('a'))} | {fmt(lg.get('r2'), 3)} | {fmt(ln.get('a'))} | "
                  f"{fmt(ln.get('r2'), 3)} | {fmt(st.get('tau_over_run'), 3)} | {fmt(st.get('r2'), 3)} | "
                  f"{r.get('best_by_aic')} | {fmt(r.get('late_slope'))} | "
                  f"{fmt(r.get('late_to_overall_slope_ratio'), 3)} | {r.get('still_increasing')} |")
        s = oee["summary"]
        print(f"\nStill increasing in last 20%: **{s['n_still_increasing']}/{s['n_seeds']}** seeds; "
              f"best model counts: {s['best_model_counts']}.")
        print("Note: late-slope p-values ignore autocorrelation and are optimistic; "
              "the late/overall slope ratio (<1 = decelerating) is the more honest signal.")
    else:
        print("_no baseline data_")

    for comp, res in comparisons.items():
        print(f"\n## {res['title']}  ({res['on_group']} vs {res['off_group']})\n")
        print("| metric | ON mean ± sd (n) | OFF mean ± sd (n) | Δ (ON−OFF) | p (Welch) | p (Holm) | d | verdict | note |")
        print("|---|---|---|---|---|---|---|---|---|")
        for key, m in res["metrics"].items():
            on, off = m["on"], m["off"]
            delta = on["mean"] - off["mean"] if np.isfinite(on["mean"]) and np.isfinite(off["mean"]) else math.nan
            p, ph = m["p"], m["p_holm"]
            if m["note"].startswith("sanity"):
                verdict = "sanity check"
            elif not np.isfinite(p) and on["n"] >= 2 and off["n"] >= 2 and on["sd"] == 0 and off["sd"] == 0:
                verdict = "constant (sd=0)"
            elif not np.isfinite(p):
                verdict = "insufficient data"
            elif np.isfinite(ph) and ph < 0.05:
                verdict = "significant"
            elif p < 0.05:
                verdict = "nominal only"
            else:
                verdict = "null"
            print(f"| {m['label']} | {fmt(on['mean'])} ± {fmt(on['sd'])} ({on['n']}) | "
                  f"{fmt(off['mean'])} ± {fmt(off['sd'])} ({off['n']}) | {fmt(delta)} | "
                  f"{fmt(p, 3)} | {fmt(ph, 3)} | {fmt(m['cohens_d'], 3)} | {verdict} | {m['note']} |")


def to_jsonable(obj):
    if isinstance(obj, dict):
        return {str(k): to_jsonable(v) for k, v in obj.items() if not str(k).startswith("_")}
    if isinstance(obj, (list, tuple)):
        return [to_jsonable(v) for v in obj]
    if isinstance(obj, (np.floating, float)):
        return float(obj) if np.isfinite(obj) else None
    if isinstance(obj, (np.integer,)):
        return int(obj)
    if isinstance(obj, np.bool_):
        return bool(obj)
    if isinstance(obj, np.ndarray):
        return to_jsonable(obj.tolist())
    return obj


# ── Main ─────────────────────────────────────────────────────────────────────

def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--root", default="output/phase7", help="phase7 output root")
    ap.add_argument("--fig-dir", default="docs/images/phase7")
    ap.add_argument("--summary", default=None, help="JSON path (default <root>/summary.json)")
    ap.add_argument("--min-tick", type=int, default=0,
                    help="skip non-extinct runs that have not reached this tick (partial data)")
    ap.add_argument("--no-align", action="store_true",
                    help="do not truncate runs to the common max tick")
    ap.add_argument("--burn-in", type=float, default=0.05,
                    help="fraction of the run ignored for fits / drought events (default 0.05)")
    ap.add_argument("--pre-window", type=int, default=2000,
                    help="ticks before drought onset used as reference level")
    ap.add_argument("--dip-window", type=int, default=4000,
                    help="ticks after drought onset searched for the minimum (drought lasts 3000)")
    ap.add_argument("--recovery-window", type=int, default=5000,
                    help="ticks after a drought ends used for recovery metrics")
    ap.add_argument("--recovery-level", type=float, default=0.95,
                    help="fraction of pre-drought population counted as 'recovered'")
    ap.add_argument("--no-plots", action="store_true")
    args = ap.parse_args()
    summary_path = args.summary or os.path.join(args.root, "summary.json")

    if not HAVE_SCIPY:
        warn("scipy not found: p-values use a normal approximation; saturating fit skipped")

    groups = {g: load_group(args.root, *spec) for g, spec in GROUPS.items()}
    run_info, common = filter_runs(groups, args.min_tick, not args.no_align)
    for g, runs in groups.items():
        print(f"{g:>10}: {len(runs)} usable runs, seeds {sorted(runs)}"
              + (f", ticks up to {max(int(d['tick'].iloc[-1]) for d in runs.values())}" if runs else ""))
    if not any(groups.values()):
        warn(f"no usable runs under {args.root}; nothing to analyze")
        sys.exit(1)

    per_run = {}
    for g, runs in groups.items():
        per_run[g] = {}
        for seed, df in runs.items():
            m = general_metrics(df)
            m.update(drought_metrics(df, args))
            m["_step"] = log_interval(df)
            per_run[g][seed] = m

    oee = analyze_oee(groups, args)
    comparisons = {}
    for comp, (on_g, off_g, _) in COMPARISONS.items():
        if not groups[on_g] or not groups[off_g]:
            warn(f"{comp}: missing runs ({on_g}={len(groups[on_g])}, {off_g}={len(groups[off_g])}), "
                 "comparison will have no statistics")
        comparisons[comp] = compare(per_run, comp, args)

    print_report(oee, comparisons, run_info, common)

    summary = {
        "meta": {"root": args.root, "common_tick": common, "min_tick": args.min_tick,
                 "burn_in": args.burn_in, "pre_window": args.pre_window,
                 "dip_window": args.dip_window, "recovery_window": args.recovery_window,
                 "recovery_level": args.recovery_level, "scipy": HAVE_SCIPY,
                 "runs": {f"{g}/seed{s}": i for (g, s), i in sorted(run_info.items())}},
        "oee": oee,
        "per_run": per_run,
        "comparisons": comparisons,
    }
    os.makedirs(os.path.dirname(summary_path) or ".", exist_ok=True)
    with open(summary_path, "w") as f:
        json.dump(to_jsonable(summary), f, indent=2)
    print(f"\nSummary JSON: {summary_path}")

    if not args.no_plots:
        print("Figures:")
        plot_oee(groups, oee, args.fig_dir)
        for comp in ("niche", "epigenetics"):
            on_g, off_g, _ = COMPARISONS[comp]
            if groups[on_g] or groups[off_g]:
                plot_drought_comparison(per_run, comparisons[comp], comp, args, args.fig_dir)
        plot_sex(groups, comparisons["sex"], args.fig_dir)
        plot_effects(comparisons, args.fig_dir)


if __name__ == "__main__":
    main()
