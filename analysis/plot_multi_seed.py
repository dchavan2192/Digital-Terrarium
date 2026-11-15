"""Compare D(t) curves across multiple seeds with confidence intervals.

Usage:
    python plot_multi_seed.py <output_dir> [plot_dir]
    e.g. python plot_multi_seed.py output/oee_5seeds docs/images
"""

import sys, os, glob
import pandas as pd
import numpy as np
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt
import matplotlib.ticker as ticker

plt.rcParams.update({
    "figure.facecolor": "#0d1117", "axes.facecolor": "#161b22",
    "axes.edgecolor": "#30363d", "axes.labelcolor": "#c9d1d9",
    "text.color": "#c9d1d9", "xtick.color": "#8b949e", "ytick.color": "#8b949e",
    "grid.color": "#21262d", "grid.alpha": 0.8, "font.family": "monospace",
    "font.size": 11, "legend.facecolor": "#161b22", "legend.edgecolor": "#30363d",
})

ACCENT = "#58a6ff"
GREEN = "#3fb950"
ORANGE = "#d29922"
RED = "#f85149"
PURPLE = "#bc8cff"
PINK = "#f778ba"
COLORS = [ACCENT, GREEN, ORANGE, RED, PURPLE, PINK, "#e3b341", "#79c0ff"]

def load_seeds(data_dir):
    csvs = sorted(glob.glob(os.path.join(data_dir, "stats_seed*.csv")))
    dfs = {}
    for path in csvs:
        name = os.path.basename(path).replace("stats_seed", "").replace(".csv", "")
        df = pd.read_csv(path)
        if len(df) > 10:
            dfs[name] = df
    print(f"Loaded {len(dfs)} seeds: {list(dfs.keys())}")
    return dfs

def plot_dt_comparison(dfs, out_dir):
    """D(t) for each seed + mean with 95% CI."""
    fig, ax = plt.subplots(figsize=(12, 6))
    fig.suptitle("Open-Ended Evolution — D(t) Across Seeds",
                 fontsize=14, fontweight="bold", color="#c9d1d9")

    # Individual seed curves (thin, transparent)
    all_dt = []
    for i, (name, df) in enumerate(dfs.items()):
        ax.plot(df["tick"], df["mean_cumulative_activity"],
                color=COLORS[i % len(COLORS)], linewidth=0.8, alpha=0.4, label=f"Seed {name}")
        all_dt.append(df.set_index("tick")["mean_cumulative_activity"])

    # Mean + CI
    combined = pd.concat(all_dt, axis=1)
    mean_dt = combined.mean(axis=1)
    std_dt = combined.std(axis=1)
    n = combined.shape[1]
    ci = 1.96 * std_dt / np.sqrt(n)

    ax.plot(mean_dt.index, mean_dt.values, color="white", linewidth=2, label=f"Mean (n={n})")
    ax.fill_between(mean_dt.index, (mean_dt - ci).values, (mean_dt + ci).values,
                     color="white", alpha=0.15, label="95% CI")

    # Log fit on mean
    t = mean_dt.index.values[1:]
    d = mean_dt.values[1:]
    c = np.polyfit(np.log(t), d, 1)
    ax.annotate(f"Mean fit: D(t) ≈ {c[0]:.2f}·ln(t) + ({c[1]:.1f})\nn = {n} seeds, 200k ticks each",
                xy=(0.02, 0.95), xycoords="axes fraction",
                fontsize=10, color="#8b949e", va="top",
                bbox=dict(boxstyle="round,pad=0.3", fc="#161b22", ec="#30363d"))

    ax.set_xlabel("Tick")
    ax.set_ylabel("Mean Cumulative Activity D(t)")
    ax.legend(loc="lower right", fontsize=9)
    ax.grid(True, alpha=0.3)
    ax.xaxis.set_major_formatter(ticker.FuncFormatter(lambda x, _: f"{x/1000:.0f}k"))
    plt.tight_layout()
    path = os.path.join(out_dir, "oee_multi_seed_dt.png")
    fig.savefig(path, dpi=150, bbox_inches="tight")
    plt.close()
    print(f"Saved: {path}")

def plot_novelty_comparison(dfs, out_dir):
    """Novel traits + active traits across seeds."""
    fig, (ax1, ax2) = plt.subplots(2, 1, figsize=(12, 8), sharex=True)
    fig.suptitle("Trait Novelty & Activity Across Seeds",
                 fontsize=14, fontweight="bold", color="#c9d1d9")

    all_novel, all_active = [], []
    for name, df in dfs.items():
        all_novel.append(df.set_index("tick")["novel_traits"])
        all_active.append(df.set_index("tick")["active_traits"])

    for series_list, ax, label, color in [
        (all_novel, ax1, "Novel traits", GREEN),
        (all_active, ax2, "Active traits", ORANGE)
    ]:
        combined = pd.concat(series_list, axis=1)
        mean_s = combined.mean(axis=1).rolling(50, min_periods=1).mean()
        std_s = combined.std(axis=1).rolling(50, min_periods=1).mean()
        n = combined.shape[1]
        ci = 1.96 * std_s / np.sqrt(n)

        ax.plot(mean_s.index, mean_s.values, color=color, linewidth=1.5, label=f"Mean {label} (n={n})")
        ax.fill_between(mean_s.index, (mean_s - ci).values, (mean_s + ci).values,
                         color=color, alpha=0.15)
        ax.set_ylabel(label)
        ax.legend(loc="upper right")
        ax.grid(True, alpha=0.3)

    ax2.set_xlabel("Tick")
    ax2.xaxis.set_major_formatter(ticker.FuncFormatter(lambda x, _: f"{x/1000:.0f}k"))
    plt.tight_layout()
    path = os.path.join(out_dir, "oee_multi_seed_novelty.png")
    fig.savefig(path, dpi=150, bbox_inches="tight")
    plt.close()
    print(f"Saved: {path}")

def print_summary(dfs):
    """Print summary statistics across seeds."""
    print("\n" + "="*60)
    print("MULTI-SEED OEE SUMMARY")
    print("="*60)
    rows = []
    for name, df in dfs.items():
        final = df.iloc[-1]
        rows.append({
            "seed": name,
            "final_dt": final["mean_cumulative_activity"],
            "final_novel": final["novel_traits"],
            "final_active": final["active_traits"],
            "max_gen": final["max_generation"],
            "final_meth": final["mean_methylation"],
            "lineage_div": final["lineage_diversity"],
        })
    summary = pd.DataFrame(rows)
    print(summary.to_string(index=False))
    print(f"\nD(t) mean ± std: {summary.final_dt.mean():.2f} ± {summary.final_dt.std():.2f}")
    print(f"Active traits mean ± std: {summary.final_active.mean():.1f} ± {summary.final_active.std():.1f}")
    print(f"Max generation mean ± std: {summary.max_gen.mean():.1f} ± {summary.max_gen.std():.1f}")

    # Log fit for each seed
    print("\nLog fits per seed:")
    for name, df in dfs.items():
        t = df["tick"].values[1:]
        d = df["mean_cumulative_activity"].values[1:]
        c = np.polyfit(np.log(t), d, 1)
        print(f"  Seed {name}: D(t) ≈ {c[0]:.2f}·ln(t) + ({c[1]:.1f})")

def main():
    data_dir = sys.argv[1]
    out_dir = sys.argv[2] if len(sys.argv) > 2 else "."
    os.makedirs(out_dir, exist_ok=True)

    dfs = load_seeds(data_dir)
    if not dfs:
        print("No seed CSVs found!")
        sys.exit(1)

    print_summary(dfs)
    plot_dt_comparison(dfs, out_dir)
    plot_novelty_comparison(dfs, out_dir)
    print("\nAll multi-seed plots generated.")

if __name__ == "__main__":
    main()
