"""Compare OEE metrics between baseline and ablation conditions.

Usage:
    python plot_ablation.py <baseline_dir> <ablation_dir> <label> [plot_dir]
    e.g. python plot_ablation.py output/oee_5seeds output/ablation_no_epigenetics "No Epigenetics" docs/images
"""

import sys, os, glob
import pandas as pd
import numpy as np
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt
import matplotlib.ticker as ticker
from scipy import stats

plt.rcParams.update({
    "figure.facecolor": "#0d1117", "axes.facecolor": "#161b22",
    "axes.edgecolor": "#30363d", "axes.labelcolor": "#c9d1d9",
    "text.color": "#c9d1d9", "xtick.color": "#8b949e", "ytick.color": "#8b949e",
    "grid.color": "#21262d", "grid.alpha": 0.8, "font.family": "monospace",
    "font.size": 11, "legend.facecolor": "#161b22", "legend.edgecolor": "#30363d",
})

BLUE = "#58a6ff"
GREEN = "#3fb950"
RED = "#f85149"

def load_seeds(data_dir):
    csvs = sorted(glob.glob(os.path.join(data_dir, "stats_seed*.csv")))
    dfs = {}
    for path in csvs:
        name = os.path.basename(path).replace("stats_seed", "").replace(".csv", "")
        df = pd.read_csv(path)
        if len(df) > 10:
            dfs[name] = df
    return dfs

def get_dt_series(dfs):
    series = []
    for df in dfs.values():
        series.append(df.set_index("tick")["mean_cumulative_activity"])
    combined = pd.concat(series, axis=1)
    return combined.mean(axis=1), combined.std(axis=1), len(series)

def get_series(dfs, col):
    series = []
    for df in dfs.values():
        series.append(df.set_index("tick")[col])
    combined = pd.concat(series, axis=1)
    return combined.mean(axis=1), combined.std(axis=1), len(series)

def plot_dt_comparison(base_dfs, abl_dfs, label, out_dir):
    fig, ax = plt.subplots(figsize=(12, 6))
    fig.suptitle(f"D(t) Comparison: Baseline vs {label}",
                 fontsize=14, fontweight="bold", color="#c9d1d9")

    for dfs, color, name in [(base_dfs, BLUE, "Baseline"), (abl_dfs, RED, label)]:
        mean_dt, std_dt, n = get_dt_series(dfs)
        ci = 1.96 * std_dt / np.sqrt(n)
        ax.plot(mean_dt.index, mean_dt.values, color=color, linewidth=2, label=f"{name} (n={n})")
        ax.fill_between(mean_dt.index, (mean_dt - ci).values, (mean_dt + ci).values,
                         color=color, alpha=0.15)

        t = mean_dt.index.values[1:]
        d = mean_dt.values[1:]
        c = np.polyfit(np.log(t), d, 1)
        ax.annotate(f"{name}: D(t) ≈ {c[0]:.2f}·ln(t)",
                    xy=(0.02, 0.95 if name == "Baseline" else 0.88),
                    xycoords="axes fraction", fontsize=10, color=color,
                    bbox=dict(boxstyle="round,pad=0.3", fc="#161b22", ec="#30363d"))

    ax.set_xlabel("Tick")
    ax.set_ylabel("Mean Cumulative Activity D(t)")
    ax.legend(loc="lower right", fontsize=10)
    ax.grid(True, alpha=0.3)
    ax.xaxis.set_major_formatter(ticker.FuncFormatter(lambda x, _: f"{x/1000:.0f}k"))
    plt.tight_layout()
    slug = label.lower().replace(" ", "_")
    path = os.path.join(out_dir, f"ablation_{slug}_dt.png")
    fig.savefig(path, dpi=150, bbox_inches="tight")
    plt.close()
    print(f"Saved: {path}")

def plot_population_stability(base_dfs, abl_dfs, label, out_dir):
    fig, (ax1, ax2) = plt.subplots(2, 1, figsize=(12, 8), sharex=True)
    fig.suptitle(f"Population & Novelty: Baseline vs {label}",
                 fontsize=14, fontweight="bold", color="#c9d1d9")

    for dfs, color, name in [(base_dfs, BLUE, "Baseline"), (abl_dfs, RED, label)]:
        mean_pop, std_pop, n = get_series(dfs, "population")
        ci = 1.96 * std_pop / np.sqrt(n)
        ax1.plot(mean_pop.index, mean_pop.values, color=color, linewidth=1.5, label=name)
        ax1.fill_between(mean_pop.index, (mean_pop - ci).values, (mean_pop + ci).values,
                          color=color, alpha=0.15)

        mean_act, std_act, n = get_series(dfs, "active_traits")
        ci = 1.96 * std_act / np.sqrt(n)
        smoothed = mean_act.rolling(50, min_periods=1).mean()
        ax2.plot(smoothed.index, smoothed.values, color=color, linewidth=1.5, label=name)

    ax1.set_ylabel("Population")
    ax1.legend(loc="lower right")
    ax1.grid(True, alpha=0.3)
    ax2.set_ylabel("Active Traits (smoothed)")
    ax2.set_xlabel("Tick")
    ax2.legend(loc="upper right")
    ax2.grid(True, alpha=0.3)
    ax2.xaxis.set_major_formatter(ticker.FuncFormatter(lambda x, _: f"{x/1000:.0f}k"))
    plt.tight_layout()
    slug = label.lower().replace(" ", "_")
    path = os.path.join(out_dir, f"ablation_{slug}_pop.png")
    fig.savefig(path, dpi=150, bbox_inches="tight")
    plt.close()
    print(f"Saved: {path}")

def statistical_tests(base_dfs, abl_dfs, label):
    print(f"\n{'='*60}")
    print(f"STATISTICAL COMPARISON: Baseline vs {label}")
    print(f"{'='*60}")

    base_final_dt = [df.iloc[-1]["mean_cumulative_activity"] for df in base_dfs.values()]
    abl_final_dt = [df.iloc[-1]["mean_cumulative_activity"] for df in abl_dfs.values()]

    print(f"\nFinal D(t):")
    print(f"  Baseline: {np.mean(base_final_dt):.2f} ± {np.std(base_final_dt):.2f}")
    print(f"  {label}: {np.mean(abl_final_dt):.2f} ± {np.std(abl_final_dt):.2f}")

    u_stat, p_value = stats.mannwhitneyu(base_final_dt, abl_final_dt, alternative='two-sided')
    print(f"  Mann-Whitney U = {u_stat:.1f}, p = {p_value:.4f}")
    print(f"  {'SIGNIFICANT' if p_value < 0.05 else 'NOT significant'} at α=0.05")

    base_final_active = [df.iloc[-1]["active_traits"] for df in base_dfs.values()]
    abl_final_active = [df.iloc[-1]["active_traits"] for df in abl_dfs.values()]

    print(f"\nActive traits at 200k:")
    print(f"  Baseline: {np.mean(base_final_active):.1f} ± {np.std(base_final_active):.1f}")
    print(f"  {label}: {np.mean(abl_final_active):.1f} ± {np.std(abl_final_active):.1f}")

    u2, p2 = stats.mannwhitneyu(base_final_active, abl_final_active, alternative='two-sided')
    print(f"  Mann-Whitney U = {u2:.1f}, p = {p2:.4f}")

    base_final_gen = [df.iloc[-1]["max_generation"] for df in base_dfs.values()]
    abl_final_gen = [df.iloc[-1]["max_generation"] for df in abl_dfs.values()]

    print(f"\nMax generation:")
    print(f"  Baseline: {np.mean(base_final_gen):.1f} ± {np.std(base_final_gen):.1f}")
    print(f"  {label}: {np.mean(abl_final_gen):.1f} ± {np.std(abl_final_gen):.1f}")

    # Log fit comparison
    print(f"\nLog fit coefficients:")
    for name, dfs in [("Baseline", base_dfs), (label, abl_dfs)]:
        coeffs = []
        for df in dfs.values():
            t = df["tick"].values[1:]
            d = df["mean_cumulative_activity"].values[1:]
            c = np.polyfit(np.log(t), d, 1)
            coeffs.append(c[0])
        print(f"  {name}: {np.mean(coeffs):.2f} ± {np.std(coeffs):.2f} (range {min(coeffs):.2f}-{max(coeffs):.2f})")

    # Population CV (stability metric)
    print(f"\nPopulation stability (CV):")
    for name, dfs in [("Baseline", base_dfs), (label, abl_dfs)]:
        cvs = []
        for df in dfs.values():
            pop = df["population"].values[100:]  # skip initial transient
            cvs.append(np.std(pop) / np.mean(pop))
        print(f"  {name}: CV = {np.mean(cvs):.4f} ± {np.std(cvs):.4f}")

def main():
    baseline_dir = sys.argv[1]
    ablation_dir = sys.argv[2]
    label = sys.argv[3]
    out_dir = sys.argv[4] if len(sys.argv) > 4 else "."
    os.makedirs(out_dir, exist_ok=True)

    base_dfs = load_seeds(baseline_dir)
    abl_dfs = load_seeds(ablation_dir)

    if not base_dfs or not abl_dfs:
        print(f"Missing data: baseline={len(base_dfs)} seeds, ablation={len(abl_dfs)} seeds")
        sys.exit(1)

    print(f"Baseline: {len(base_dfs)} seeds, Ablation ({label}): {len(abl_dfs)} seeds")

    statistical_tests(base_dfs, abl_dfs, label)
    plot_dt_comparison(base_dfs, abl_dfs, label, out_dir)
    plot_population_stability(base_dfs, abl_dfs, label, out_dir)
    print(f"\nAll ablation plots generated for: {label}")

if __name__ == "__main__":
    main()
