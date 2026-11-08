"""Generate README plots from Terrarium simulation data.

Usage:
    python analysis/plot_readme.py output/pi_500k_v2.csv

Produces PNG files in docs/images/ for embedding in README.md.
"""

import sys
import os
import pandas as pd
import matplotlib.pyplot as plt
import matplotlib.ticker as ticker
import numpy as np

# Style
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

COLORS = {
    "green": "#3fb950",
    "blue": "#58a6ff",
    "purple": "#bc8cff",
    "orange": "#d29922",
    "red": "#f85149",
    "cyan": "#39d2c0",
    "pink": "#f778ba",
}


def load_data(path):
    df = pd.read_csv(path)
    # Convert ticks to thousands for readability
    df["tick_k"] = df["tick"] / 1000
    return df


def smooth(series, window=50):
    return series.rolling(window=window, min_periods=1).mean()


def save(fig, name, output_dir):
    os.makedirs(output_dir, exist_ok=True)
    path = os.path.join(output_dir, name)
    fig.savefig(path, dpi=150, bbox_inches="tight", facecolor=fig.get_facecolor())
    plt.close(fig)
    print(f"  Saved {path}")


def plot_population_energy(df, output_dir):
    """Population and mean energy over time — the core vital signs."""
    fig, (ax1, ax2) = plt.subplots(2, 1, figsize=(12, 6), sharex=True)

    ax1.fill_between(df["tick_k"], df["population"], alpha=0.3, color=COLORS["green"])
    ax1.plot(df["tick_k"], df["population"], color=COLORS["green"], linewidth=0.8)
    ax1.set_ylabel("Population")
    ax1.set_title("Population & Energy Over 500k Ticks", fontsize=14, fontweight="bold")
    ax1.grid(True, linewidth=0.5)

    ax2.plot(df["tick_k"], smooth(df["mean_energy"]), color=COLORS["orange"], linewidth=1.2)
    ax2.set_ylabel("Mean Energy")
    ax2.set_xlabel("Tick (thousands)")
    ax2.grid(True, linewidth=0.5)

    fig.tight_layout()
    save(fig, "population_energy.png", output_dir)


def plot_birth_death(df, output_dir):
    """Cumulative births/deaths and per-interval turnover."""
    fig, (ax1, ax2) = plt.subplots(2, 1, figsize=(12, 6), sharex=True)

    ax1.plot(df["tick_k"], df["total_births"] / 1000, color=COLORS["green"], linewidth=1.2, label="Births")
    ax1.plot(df["tick_k"], df["total_deaths"] / 1000, color=COLORS["red"], linewidth=1.2, label="Deaths")
    ax1.set_ylabel("Cumulative (thousands)")
    ax1.set_title("Birth/Death Dynamics", fontsize=14, fontweight="bold")
    ax1.legend()
    ax1.grid(True, linewidth=0.5)

    ax2.plot(df["tick_k"], smooth(df["births"], 20), color=COLORS["green"], linewidth=0.8, alpha=0.8, label="Births/tick")
    ax2.plot(df["tick_k"], smooth(df["deaths"], 20), color=COLORS["red"], linewidth=0.8, alpha=0.8, label="Deaths/tick")
    ax2.set_ylabel("Per tick (smoothed)")
    ax2.set_xlabel("Tick (thousands)")
    ax2.legend()
    ax2.grid(True, linewidth=0.5)

    fig.tight_layout()
    save(fig, "birth_death.png", output_dir)


def plot_combat(df, output_dir):
    """Attack frequency and cumulative attacks over time."""
    if "attacks" not in df.columns or df["total_attacks"].max() == 0:
        print("  Skipping combat plot (no attack data)")
        return

    fig, (ax1, ax2) = plt.subplots(2, 1, figsize=(12, 6), sharex=True)

    ax1.plot(df["tick_k"], df["total_attacks"] / 1000, color=COLORS["red"], linewidth=1.2)
    ax1.set_ylabel("Cumulative attacks (thousands)")
    ax1.set_title("Predator-Prey Combat", fontsize=14, fontweight="bold")
    ax1.grid(True, linewidth=0.5)

    ax2.plot(df["tick_k"], smooth(df["attacks"], 20), color=COLORS["red"], linewidth=0.8, alpha=0.8)
    ax2.set_ylabel("Attacks per tick (smoothed)")
    ax2.set_xlabel("Tick (thousands)")
    ax2.grid(True, linewidth=0.5)

    fig.tight_layout()
    save(fig, "combat.png", output_dir)


def plot_evolution(df, output_dir):
    """Generation depth and species count — evolutionary progress."""
    if "mean_generation" not in df.columns:
        print("  Skipping evolution plot (no generation data)")
        return

    fig, (ax1, ax2) = plt.subplots(2, 1, figsize=(12, 6), sharex=True)

    ax1.plot(df["tick_k"], df["mean_generation"], color=COLORS["purple"], linewidth=1.2)
    ax1.set_ylabel("Mean Generation")
    ax1.set_title("Evolutionary Progress", fontsize=14, fontweight="bold")
    ax1.grid(True, linewidth=0.5)

    if "species_count" in df.columns:
        ax2.plot(df["tick_k"], smooth(df["species_count"], 10), color=COLORS["cyan"], linewidth=1.2)
        ax2.set_ylabel("Species Count")
    ax2.set_xlabel("Tick (thousands)")
    ax2.grid(True, linewidth=0.5)

    fig.tight_layout()
    save(fig, "evolution.png", output_dir)


def plot_dashboard(df, output_dir):
    """Single 2x2 dashboard — the hero image for the README."""
    fig, axes = plt.subplots(2, 2, figsize=(14, 8))

    # Population
    ax = axes[0, 0]
    ax.fill_between(df["tick_k"], df["population"], alpha=0.3, color=COLORS["green"])
    ax.plot(df["tick_k"], df["population"], color=COLORS["green"], linewidth=0.8)
    ax.set_ylabel("Population")
    ax.set_title("Population", fontsize=12, fontweight="bold")
    ax.grid(True, linewidth=0.5)

    # Energy
    ax = axes[0, 1]
    ax.plot(df["tick_k"], smooth(df["mean_energy"]), color=COLORS["orange"], linewidth=1.2)
    ax.set_ylabel("Mean Energy")
    ax.set_title("Mean Energy", fontsize=12, fontweight="bold")
    ax.grid(True, linewidth=0.5)

    # Births & Deaths
    ax = axes[1, 0]
    ax.plot(df["tick_k"], smooth(df["births"], 20), color=COLORS["green"], linewidth=0.8, label="Births")
    ax.plot(df["tick_k"], smooth(df["deaths"], 20), color=COLORS["red"], linewidth=0.8, label="Deaths")
    ax.set_ylabel("Per tick")
    ax.set_xlabel("Tick (thousands)")
    ax.set_title("Birth/Death Rate", fontsize=12, fontweight="bold")
    ax.legend(fontsize=9)
    ax.grid(True, linewidth=0.5)

    # Evolution
    ax = axes[1, 1]
    if "mean_generation" in df.columns:
        ax.plot(df["tick_k"], df["mean_generation"], color=COLORS["purple"], linewidth=1.2)
        ax.set_ylabel("Mean Generation")
    ax.set_xlabel("Tick (thousands)")
    ax.set_title("Evolutionary Depth", fontsize=12, fontweight="bold")
    ax.grid(True, linewidth=0.5)

    fig.suptitle("Terrarium — 500k Tick Simulation", fontsize=16, fontweight="bold", y=1.01)
    fig.tight_layout()
    save(fig, "dashboard.png", output_dir)


def main():
    if len(sys.argv) < 2:
        print(f"Usage: python {sys.argv[0]} <csv_path>")
        sys.exit(1)

    csv_path = sys.argv[1]
    output_dir = os.path.join(os.path.dirname(os.path.dirname(csv_path)), "docs", "images")
    # Default to project docs/images if csv is in output/
    if "output" in csv_path:
        output_dir = "docs/images"

    print(f"Loading {csv_path}...")
    df = load_data(csv_path)
    print(f"  {len(df)} rows, ticks 0-{df['tick'].max():.0f}")

    print("Generating plots...")
    plot_dashboard(df, output_dir)
    plot_population_energy(df, output_dir)
    plot_birth_death(df, output_dir)
    plot_combat(df, output_dir)
    plot_evolution(df, output_dir)

    print("Done! Images saved to", output_dir)


if __name__ == "__main__":
    main()
