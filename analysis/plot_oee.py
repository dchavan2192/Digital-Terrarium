"""Plot Open-Ended Evolution metrics from Terrarium simulation data.

Usage:
    python analysis/plot_oee.py output/oee_200k/stats_seed42.csv
"""

import sys
import os
import pandas as pd
import matplotlib.pyplot as plt
import matplotlib.ticker as ticker

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
})

ACCENT = "#58a6ff"
GREEN = "#3fb950"
ORANGE = "#d29922"
RED = "#f85149"
PURPLE = "#bc8cff"
PINK = "#f778ba"

def smooth(series, window=50):
    return series.rolling(window, min_periods=1).mean()

def plot_oee_activity(df, out_dir):
    fig, (ax1, ax2) = plt.subplots(2, 1, figsize=(12, 8), sharex=True)
    fig.suptitle("Open-Ended Evolution — Cumulative Activity D(t)",
                 fontsize=14, fontweight="bold", color="#c9d1d9")

    ax1.plot(df["tick"], df["mean_cumulative_activity"],
             color=ACCENT, linewidth=1.5, label="D(t)")
    ax1.set_ylabel("Mean Cumulative Activity D(t)")
    ax1.legend(loc="upper left")
    ax1.grid(True, alpha=0.3)

    ticks = df["tick"].values
    dt = df["mean_cumulative_activity"].values
    if len(dt) > 100:
        mid = len(dt) // 2
        rate_early = (dt[mid] - dt[0]) / (ticks[mid] - ticks[0] + 1)
        rate_late = (dt[-1] - dt[mid]) / (ticks[-1] - ticks[mid] + 1)
        ax1.annotate(f"Early rate: {rate_early:.4f}/tick\nLate rate: {rate_late:.4f}/tick",
                     xy=(0.02, 0.95), xycoords="axes fraction",
                     fontsize=9, color="#8b949e", va="top",
                     bbox=dict(boxstyle="round,pad=0.3", fc="#161b22", ec="#30363d"))

    ax2.plot(df["tick"], smooth(df["novel_traits"]),
             color=GREEN, linewidth=1.2, label="Novel traits/window", alpha=0.9)
    ax2.plot(df["tick"], smooth(df["active_traits"]),
             color=ORANGE, linewidth=1.2, label="Active traits", alpha=0.9)
    ax2.set_xlabel("Tick")
    ax2.set_ylabel("Trait Count")
    ax2.legend(loc="upper right")
    ax2.grid(True, alpha=0.3)
    ax2.xaxis.set_major_formatter(ticker.FuncFormatter(lambda x, _: f"{x/1000:.0f}k"))

    plt.tight_layout()
    path = os.path.join(out_dir, "oee_activity.png")
    fig.savefig(path, dpi=150, bbox_inches="tight")
    plt.close()
    print(f"Saved: {path}")

def plot_lineage(df, out_dir):
    fig, (ax1, ax2) = plt.subplots(2, 1, figsize=(12, 8), sharex=True)
    fig.suptitle("Lineage Tracking & Generation Depth",
                 fontsize=14, fontweight="bold", color="#c9d1d9")

    ax1.plot(df["tick"], df["max_generation"],
             color=PURPLE, linewidth=1.2, label="Max generation")
    ax1.plot(df["tick"], smooth(df["mean_generation"]),
             color=ACCENT, linewidth=1.2, label="Mean generation", alpha=0.8)
    ax1.set_ylabel("Generation")
    ax1.legend(loc="upper left")
    ax1.grid(True, alpha=0.3)

    ax2.plot(df["tick"], df["lineage_diversity"],
             color=GREEN, linewidth=1.2, label="Distinct root ancestors")
    ax2.set_xlabel("Tick")
    ax2.set_ylabel("Lineage Diversity")
    ax2.legend(loc="upper right")
    ax2.grid(True, alpha=0.3)
    ax2.xaxis.set_major_formatter(ticker.FuncFormatter(lambda x, _: f"{x/1000:.0f}k"))

    plt.tight_layout()
    path = os.path.join(out_dir, "oee_lineage.png")
    fig.savefig(path, dpi=150, bbox_inches="tight")
    plt.close()
    print(f"Saved: {path}")

def plot_epigenetics(df, out_dir):
    fig, (ax1, ax2) = plt.subplots(2, 1, figsize=(12, 8), sharex=True)
    fig.suptitle("Epigenetics & Niche Construction",
                 fontsize=14, fontweight="bold", color="#c9d1d9")

    ax1.plot(df["tick"], smooth(df["mean_methylation"]),
             color=RED, linewidth=1.2, label="Mean methylation")
    ax1.set_ylabel("Methylation Level [0, 1]")
    ax1.legend(loc="upper right")
    ax1.grid(True, alpha=0.3)
    ax1.set_ylim(-0.05, 1.05)

    ax2.plot(df["tick"], smooth(df["mean_construction_level"]),
             color=PINK, linewidth=1.2, label="Mean construction level")
    ax2.axhline(y=0, color="#30363d", linestyle="--", alpha=0.5)
    ax2.set_xlabel("Tick")
    ax2.set_ylabel("Construction Level")
    ax2.legend(loc="upper right")
    ax2.grid(True, alpha=0.3)
    ax2.xaxis.set_major_formatter(ticker.FuncFormatter(lambda x, _: f"{x/1000:.0f}k"))

    plt.tight_layout()
    path = os.path.join(out_dir, "oee_epigenetics.png")
    fig.savefig(path, dpi=150, bbox_inches="tight")
    plt.close()
    print(f"Saved: {path}")

def main():
    if len(sys.argv) < 2:
        print("Usage: python analysis/plot_oee.py <csv_path>")
        sys.exit(1)

    csv_path = sys.argv[1]
    df = pd.read_csv(csv_path)
    print(f"Loaded {len(df)} rows from {csv_path}")
    print(f"Tick range: {df['tick'].min()} - {df['tick'].max()}")
    print(f"Final D(t): {df['mean_cumulative_activity'].iloc[-1]:.2f}")
    print(f"Final novel_traits: {df['novel_traits'].iloc[-1]}")
    print(f"Final active_traits: {df['active_traits'].iloc[-1]}")
    print(f"Final max_generation: {df['max_generation'].iloc[-1]}")
    print(f"Final methylation: {df['mean_methylation'].iloc[-1]:.4f}")

    out_dir = "docs/images"
    os.makedirs(out_dir, exist_ok=True)

    plot_oee_activity(df, out_dir)
    plot_lineage(df, out_dir)
    plot_epigenetics(df, out_dir)
    print("Done — all OEE plots generated.")

if __name__ == "__main__":
    main()
