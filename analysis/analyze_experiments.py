"""Analyze Terrarium sweep experiments and produce publication-quality figures.

Three experiments:
  1. Cooperation (sharing.share_amount sweep) — Does kin-selected sharing evolve?
  2. Food scarcity (food.regrowth_rate sweep) — Does scarcity drive predation?
  3. Brain cost (energy.brain_cost sweep) — Does taxing brains constrain complexity?

Each experiment has 5 parameter values x 5 seeds x 200k ticks, logged every 100 ticks.

Usage:
    python analysis/analyze_experiments.py [--remote pi@100.127.61.93]

If --remote is given, data is fetched via scp first.
Outputs PNG figures to docs/images/experiments/.
"""

import argparse
import os
import subprocess
import sys
from pathlib import Path
from glob import glob

import numpy as np
import pandas as pd
import matplotlib.pyplot as plt
import matplotlib.ticker as ticker
from scipy import stats

# --- Style ---
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

PALETTE = ["#3fb950", "#58a6ff", "#bc8cff", "#d29922", "#f85149"]
PROJECT_ROOT = Path(__file__).resolve().parent.parent
OUTPUT_DIR = PROJECT_ROOT / "docs" / "images" / "experiments"


def smooth(s, window=30):
    return s.rolling(window=window, min_periods=1).mean()


def save(fig, name):
    OUTPUT_DIR.mkdir(parents=True, exist_ok=True)
    path = OUTPUT_DIR / name
    fig.savefig(str(path), dpi=150, bbox_inches="tight", facecolor=fig.get_facecolor())
    plt.close(fig)
    print(f"  Saved {path}")


# ---------------------------------------------------------------------------
# Data loading
# ---------------------------------------------------------------------------

def load_sweep(sweep_dir):
    """Load all CSVs from a sweep directory into a dict keyed by (param_value, seed).

    Returns (param_name, dict_of_dataframes, sorted_values).
    """
    csv_files = sorted(glob(os.path.join(sweep_dir, "sweep", "stats_seed*.csv")))
    if not csv_files:
        return None, {}, []

    data = {}
    param_name = None
    values_seen = set()

    for f in csv_files:
        # Filename: stats_seed{N}_{param}={value}_seed{N}.csv
        basename = os.path.basename(f)
        # Extract label from filename: everything between "stats_seed{N}_" and ".csv"
        # Example: stats_seed0_sharing.share_amount=0.000000_seed0.csv
        parts = basename.replace("stats_", "").replace(".csv", "")
        # parts: seed0_sharing.share_amount=0.000000_seed0
        # Find the param=value segment
        segments = parts.split("_")
        # Reconstruct: find the segment with "="
        param_val_parts = []
        seed_val = None
        for i, seg in enumerate(segments):
            if "=" in seg:
                param_val_parts.append(seg)
            elif seg.startswith("seed") and i == len(segments) - 1:
                seed_val = int(seg.replace("seed", ""))

        if not param_val_parts or seed_val is None:
            continue

        param_val_str = "_".join(param_val_parts)
        pname, pval_str = param_val_str.rsplit("=", 1)
        pval = float(pval_str)

        if param_name is None:
            param_name = pname
        values_seen.add(pval)

        try:
            df = pd.read_csv(f)
            if len(df) > 0:
                df["tick_k"] = df["tick"] / 1000
                data[(pval, seed_val)] = df
        except Exception as e:
            print(f"  Warning: failed to load {f}: {e}")

    sorted_values = sorted(values_seen)
    return param_name, data, sorted_values


def aggregate_by_value(data, sorted_values):
    """For each parameter value, compute mean and std across seeds at each tick."""
    agg = {}
    for val in sorted_values:
        seed_dfs = [df for (v, s), df in data.items() if v == val]
        if not seed_dfs:
            continue
        # Align on tick column (they should all have the same ticks)
        # Use the shortest run's tick range
        min_len = min(len(df) for df in seed_dfs)
        aligned = pd.concat(
            [df.iloc[:min_len].set_index("tick") for df in seed_dfs],
            keys=range(len(seed_dfs)),
        )
        mean = aligned.groupby("tick").mean()
        std = aligned.groupby("tick").std().fillna(0)
        mean["tick_k"] = mean.index / 1000
        std["tick_k"] = std.index / 1000
        agg[val] = (mean, std)
    return agg


def final_stats(data, sorted_values, columns):
    """Extract final-tick values for each seed, grouped by parameter value.

    Returns a dict: {col: {val: [final_values_per_seed]}}
    """
    result = {col: {} for col in columns}
    for val in sorted_values:
        seed_dfs = [(s, df) for (v, s), df in data.items() if v == val]
        for col in columns:
            vals = []
            for seed, df in seed_dfs:
                if col in df.columns and len(df) > 0:
                    vals.append(df[col].iloc[-1])
            result[col][val] = vals
    return result


# ---------------------------------------------------------------------------
# Experiment 1: Cooperation
# ---------------------------------------------------------------------------

def plot_exp1_cooperation(data, sorted_values):
    """Does kin-selected sharing evolve under different share_amount settings?

    Key metrics: cooperation_rate, kin_share_ratio, total_shares
    Citation: Hamilton (1964) — kin selection predicts rB > C altruism
    """
    print("\n--- Experiment 1: Cooperation ---")
    agg = aggregate_by_value(data, sorted_values)
    if not agg:
        print("  No data available.")
        return

    # Check which columns exist
    sample_mean = next(iter(agg.values()))[0]
    has_coop = "cooperation_rate" in sample_mean.columns
    has_kin = "kin_share_ratio" in sample_mean.columns
    has_shares = "total_shares" in sample_mean.columns

    if not has_coop:
        print("  Warning: cooperation_rate column missing. Older CSV format?")
        print("  Available columns:", list(sample_mean.columns))
        return

    # --- Figure 1: Cooperation rate over time per share_amount ---
    fig, axes = plt.subplots(2, 2, figsize=(14, 10))

    # Top-left: cooperation rate over time
    ax = axes[0, 0]
    for i, val in enumerate(sorted_values):
        if val in agg:
            mean, std = agg[val]
            ax.plot(mean["tick_k"], smooth(mean["cooperation_rate"]),
                    color=PALETTE[i % len(PALETTE)], linewidth=1.2,
                    label=f"share={val:.0f}")
    ax.set_ylabel("Cooperation Rate")
    ax.set_title("Cooperation Rate Over Time", fontweight="bold")
    ax.legend(fontsize=8)
    ax.grid(True, linewidth=0.5)

    # Top-right: kin share ratio over time
    ax = axes[0, 1]
    if has_kin:
        for i, val in enumerate(sorted_values):
            if val in agg and val > 0:  # skip share_amount=0
                mean, std = agg[val]
                ax.plot(mean["tick_k"], smooth(mean["kin_share_ratio"]),
                        color=PALETTE[i % len(PALETTE)], linewidth=1.2,
                        label=f"share={val:.0f}")
        ax.set_ylabel("Kin Share Ratio")
        ax.set_title("Fraction of Shares to Kin (hue < 30 deg)", fontweight="bold")
        ax.axhline(y=0.5, color="#8b949e", linestyle="--", alpha=0.5, label="random baseline")
        ax.legend(fontsize=8)
        ax.grid(True, linewidth=0.5)
    else:
        ax.text(0.5, 0.5, "kin_share_ratio\nnot available", transform=ax.transAxes,
                ha="center", va="center", fontsize=12)

    # Bottom-left: population over time
    ax = axes[1, 0]
    for i, val in enumerate(sorted_values):
        if val in agg:
            mean, std = agg[val]
            ax.plot(mean["tick_k"], smooth(mean["population"]),
                    color=PALETTE[i % len(PALETTE)], linewidth=1.2,
                    label=f"share={val:.0f}")
    ax.set_ylabel("Population")
    ax.set_xlabel("Tick (thousands)")
    ax.set_title("Population Stability", fontweight="bold")
    ax.legend(fontsize=8)
    ax.grid(True, linewidth=0.5)

    # Bottom-right: bar chart of final cooperation rate
    ax = axes[1, 1]
    finals = final_stats(data, sorted_values, ["cooperation_rate"])
    vals_for_bar = []
    means_for_bar = []
    stds_for_bar = []
    for val in sorted_values:
        if val in finals["cooperation_rate"] and finals["cooperation_rate"][val]:
            vals_for_bar.append(f"{val:.0f}")
            arr = finals["cooperation_rate"][val]
            means_for_bar.append(np.mean(arr))
            stds_for_bar.append(np.std(arr))
    if vals_for_bar:
        x = np.arange(len(vals_for_bar))
        ax.bar(x, means_for_bar, yerr=stds_for_bar, color=PALETTE[:len(x)],
               capsize=4, edgecolor="#30363d")
        ax.set_xticks(x)
        ax.set_xticklabels(vals_for_bar)
        ax.set_xlabel("share_amount")
        ax.set_ylabel("Final Cooperation Rate")
        ax.set_title("End-of-Run Cooperation (mean +/- std)", fontweight="bold")
        ax.grid(True, linewidth=0.5, axis="y")

    fig.suptitle("Exp 1: Does Kin-Selected Sharing Evolve?\nHamilton (1964): rB > C",
                 fontsize=14, fontweight="bold", y=1.02)
    fig.tight_layout()
    save(fig, "exp1_cooperation.png")

    # Print summary statistics
    print("  Final cooperation rates (mean +/- std across seeds):")
    for val in sorted_values:
        if val in finals["cooperation_rate"] and finals["cooperation_rate"][val]:
            arr = finals["cooperation_rate"][val]
            print(f"    share_amount={val:6.1f}: {np.mean(arr):.4f} +/- {np.std(arr):.4f}")


# ---------------------------------------------------------------------------
# Experiment 2: Food Scarcity
# ---------------------------------------------------------------------------

def plot_exp2_food_scarcity(data, sorted_values):
    """Does food scarcity drive a trophic shift toward predation?

    Key metrics: mean_trophic_level, predator_count, herbivore_count, population
    Citation: Lotka (1925), Volterra (1926), Lindeman (1942)
    """
    print("\n--- Experiment 2: Food Scarcity ---")
    agg = aggregate_by_value(data, sorted_values)
    if not agg:
        print("  No data available.")
        return

    sample_mean = next(iter(agg.values()))[0]
    has_trophic = "mean_trophic_level" in sample_mean.columns

    fig, axes = plt.subplots(2, 2, figsize=(14, 10))

    # Top-left: population over time
    ax = axes[0, 0]
    for i, val in enumerate(sorted_values):
        if val in agg:
            mean, _ = agg[val]
            ax.plot(mean["tick_k"], smooth(mean["population"]),
                    color=PALETTE[i % len(PALETTE)], linewidth=1.2,
                    label=f"regrowth={val}")
    ax.set_ylabel("Population")
    ax.set_title("Population vs Food Scarcity", fontweight="bold")
    ax.legend(fontsize=8)
    ax.grid(True, linewidth=0.5)

    # Top-right: mean trophic level over time
    ax = axes[0, 1]
    if has_trophic:
        for i, val in enumerate(sorted_values):
            if val in agg:
                mean, _ = agg[val]
                ax.plot(mean["tick_k"], smooth(mean["mean_trophic_level"]),
                        color=PALETTE[i % len(PALETTE)], linewidth=1.2,
                        label=f"regrowth={val}")
        ax.set_ylabel("Mean Trophic Level")
        ax.set_title("Trophic Shift Under Scarcity", fontweight="bold")
        ax.axhline(y=1.0, color="#8b949e", linestyle="--", alpha=0.5, label="pure herbivore")
        ax.legend(fontsize=8)
        ax.grid(True, linewidth=0.5)
    else:
        ax.text(0.5, 0.5, "trophic data\nnot available", transform=ax.transAxes,
                ha="center", va="center", fontsize=12)

    # Bottom-left: attacks per tick over time
    ax = axes[1, 0]
    for i, val in enumerate(sorted_values):
        if val in agg:
            mean, _ = agg[val]
            if "attacks" in mean.columns:
                ax.plot(mean["tick_k"], smooth(mean["attacks"], 50),
                        color=PALETTE[i % len(PALETTE)], linewidth=1.2,
                        label=f"regrowth={val}")
    ax.set_ylabel("Attacks per Tick")
    ax.set_xlabel("Tick (thousands)")
    ax.set_title("Predation Intensity", fontweight="bold")
    ax.legend(fontsize=8)
    ax.grid(True, linewidth=0.5)

    # Bottom-right: bar chart — final predator fraction vs regrowth rate
    ax = axes[1, 1]
    if has_trophic:
        finals = final_stats(data, sorted_values,
                             ["predator_count", "herbivore_count", "population"])
        pred_fracs = []
        herb_fracs = []
        labels = []
        for val in sorted_values:
            preds = finals["predator_count"].get(val, [])
            herbs = finals["herbivore_count"].get(val, [])
            pops = finals["population"].get(val, [])
            if pops and np.mean(pops) > 0:
                labels.append(f"{val}")
                pred_fracs.append(np.mean(preds) / np.mean(pops))
                herb_fracs.append(np.mean(herbs) / np.mean(pops))

        if labels:
            x = np.arange(len(labels))
            w = 0.35
            ax.bar(x - w/2, herb_fracs, w, label="Herbivores", color="#3fb950",
                   edgecolor="#30363d")
            ax.bar(x + w/2, pred_fracs, w, label="Predators", color="#f85149",
                   edgecolor="#30363d")
            ax.set_xticks(x)
            ax.set_xticklabels(labels)
            ax.set_xlabel("food.regrowth_rate")
            ax.set_ylabel("Fraction of Population")
            ax.set_title("Trophic Composition at End of Run", fontweight="bold")
            ax.legend(fontsize=8)
            ax.grid(True, linewidth=0.5, axis="y")

    fig.suptitle("Exp 2: Does Scarcity Drive Predation?\nLotka-Volterra (1925/1926), Lindeman (1942)",
                 fontsize=14, fontweight="bold", y=1.02)
    fig.tight_layout()
    save(fig, "exp2_food_scarcity.png")

    # Correlation test: regrowth_rate vs final trophic level
    if has_trophic:
        finals = final_stats(data, sorted_values, ["mean_trophic_level"])
        all_regrowth = []
        all_trophic = []
        for val in sorted_values:
            for t in finals["mean_trophic_level"].get(val, []):
                all_regrowth.append(val)
                all_trophic.append(t)
        if len(all_regrowth) >= 4:
            r, p = stats.pearsonr(all_regrowth, all_trophic)
            print(f"  Pearson correlation (regrowth vs trophic): r={r:.3f}, p={p:.4f}")
            if p < 0.05:
                direction = "negative" if r < 0 else "positive"
                print(f"  --> Significant {direction} correlation: less food = {'more' if r < 0 else 'less'} predation")
            else:
                print(f"  --> Not significant (p={p:.3f})")


# ---------------------------------------------------------------------------
# Experiment 3: Brain Cost
# ---------------------------------------------------------------------------

def plot_exp3_brain_cost(data, sorted_values):
    """Does taxing brains constrain neural complexity?

    Key metrics: mean_connections, mean_brain_neurons, mean_brain_entropy,
                 behavioral_diversity, species_count
    Citation: Laughlin et al. (1998) "The metabolic cost of neural information"
    """
    print("\n--- Experiment 3: Brain Cost ---")
    agg = aggregate_by_value(data, sorted_values)
    if not agg:
        print("  No data available.")
        return

    sample_mean = next(iter(agg.values()))[0]
    has_connections = "mean_connections" in sample_mean.columns
    has_entropy = "mean_brain_entropy" in sample_mean.columns
    has_diversity = "behavioral_diversity" in sample_mean.columns

    fig, axes = plt.subplots(2, 2, figsize=(14, 10))

    # Top-left: brain connections over time
    ax = axes[0, 0]
    if has_connections:
        for i, val in enumerate(sorted_values):
            if val in agg:
                mean, _ = agg[val]
                ax.plot(mean["tick_k"], smooth(mean["mean_connections"]),
                        color=PALETTE[i % len(PALETTE)], linewidth=1.2,
                        label=f"cost={val}")
        ax.set_ylabel("Mean Connections")
        ax.set_title("Brain Complexity Over Time", fontweight="bold")
        ax.legend(fontsize=8)
        ax.grid(True, linewidth=0.5)
    else:
        # Fall back to mean_brain_neurons
        for i, val in enumerate(sorted_values):
            if val in agg:
                mean, _ = agg[val]
                ax.plot(mean["tick_k"], smooth(mean["mean_brain_neurons"]),
                        color=PALETTE[i % len(PALETTE)], linewidth=1.2,
                        label=f"cost={val}")
        ax.set_ylabel("Mean Neurons")
        ax.set_title("Brain Size Over Time", fontweight="bold")
        ax.legend(fontsize=8)
        ax.grid(True, linewidth=0.5)

    # Top-right: brain entropy over time
    ax = axes[0, 1]
    if has_entropy:
        for i, val in enumerate(sorted_values):
            if val in agg:
                mean, _ = agg[val]
                ax.plot(mean["tick_k"], smooth(mean["mean_brain_entropy"]),
                        color=PALETTE[i % len(PALETTE)], linewidth=1.2,
                        label=f"cost={val}")
        ax.set_ylabel("Brain Entropy")
        ax.set_title("Behavioral Complexity (Shannon, 1948)", fontweight="bold")
        ax.legend(fontsize=8)
        ax.grid(True, linewidth=0.5)
    else:
        ax.text(0.5, 0.5, "entropy data\nnot available", transform=ax.transAxes,
                ha="center", va="center", fontsize=12)

    # Bottom-left: population over time (survival pressure)
    ax = axes[1, 0]
    for i, val in enumerate(sorted_values):
        if val in agg:
            mean, _ = agg[val]
            ax.plot(mean["tick_k"], smooth(mean["population"]),
                    color=PALETTE[i % len(PALETTE)], linewidth=1.2,
                    label=f"cost={val}")
    ax.set_ylabel("Population")
    ax.set_xlabel("Tick (thousands)")
    ax.set_title("Population (Does High Brain Cost Kill?)", fontweight="bold")
    ax.legend(fontsize=8)
    ax.grid(True, linewidth=0.5)

    # Bottom-right: bar chart — final mean_connections vs brain_cost
    ax = axes[1, 1]
    col = "mean_connections" if has_connections else "mean_brain_neurons"
    finals = final_stats(data, sorted_values, [col])
    vals_for_bar = []
    means_for_bar = []
    stds_for_bar = []
    for val in sorted_values:
        if val in finals[col] and finals[col][val]:
            vals_for_bar.append(f"{val}")
            arr = finals[col][val]
            means_for_bar.append(np.mean(arr))
            stds_for_bar.append(np.std(arr))
    if vals_for_bar:
        x = np.arange(len(vals_for_bar))
        ax.bar(x, means_for_bar, yerr=stds_for_bar, color=PALETTE[:len(x)],
               capsize=4, edgecolor="#30363d")
        ax.set_xticks(x)
        ax.set_xticklabels(vals_for_bar)
        ax.set_xlabel("energy.brain_cost")
        ylabel = "Final Mean Connections" if has_connections else "Final Mean Neurons"
        ax.set_ylabel(ylabel)
        ax.set_title("Brain Complexity at End of Run", fontweight="bold")
        ax.grid(True, linewidth=0.5, axis="y")

    fig.suptitle("Exp 3: Does Taxing Brains Constrain Complexity?\nLaughlin et al. (1998): metabolic cost of neural information",
                 fontsize=14, fontweight="bold", y=1.02)
    fig.tight_layout()
    save(fig, "exp3_brain_cost.png")

    # Correlation test: brain_cost vs final connections
    all_cost = []
    all_conn = []
    for val in sorted_values:
        for c in finals[col].get(val, []):
            all_cost.append(val)
            all_conn.append(c)
    if len(all_cost) >= 4:
        r, p = stats.pearsonr(all_cost, all_conn)
        print(f"  Pearson correlation (brain_cost vs {col}): r={r:.3f}, p={p:.4f}")
        if p < 0.05:
            print(f"  --> Significant: higher brain cost = {'fewer' if r < 0 else 'more'} connections")
        else:
            print(f"  --> Not significant (p={p:.3f})")


# ---------------------------------------------------------------------------
# Summary table
# ---------------------------------------------------------------------------

def print_summary_table(all_experiments):
    """Print a text summary table across all experiments."""
    print("\n" + "=" * 70)
    print("EXPERIMENT SUMMARY")
    print("=" * 70)
    for name, param_name, data, sorted_values in all_experiments:
        if not data:
            print(f"\n{name}: NO DATA")
            continue
        print(f"\n{name} (sweep: {param_name})")
        print(f"  {'Value':>12}  {'Seeds':>5}  {'Survived':>8}  {'Final Pop':>10}  {'Final Tick':>10}")
        print(f"  {'-'*12}  {'-'*5}  {'-'*8}  {'-'*10}  {'-'*10}")
        for val in sorted_values:
            seed_dfs = [(s, df) for (v, s), df in data.items() if v == val]
            if not seed_dfs:
                continue
            n_seeds = len(seed_dfs)
            survived = sum(1 for _, df in seed_dfs if len(df) > 0 and df["population"].iloc[-1] > 0)
            final_pops = [df["population"].iloc[-1] for _, df in seed_dfs if len(df) > 0]
            final_ticks = [df["tick"].iloc[-1] for _, df in seed_dfs if len(df) > 0]
            print(f"  {val:12.6f}  {n_seeds:5d}  {survived:5d}/{n_seeds:<2d}  "
                  f"{np.mean(final_pops):10.0f}  {np.mean(final_ticks):10.0f}")


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

def fetch_from_remote(remote, local_output):
    """Fetch experiment data from Pi via scp."""
    for exp in ["exp1_cooperation", "exp2_food_scarcity", "exp3_brain_cost"]:
        remote_path = f"{remote}:~/Digital-Terrarium/output/{exp}/"
        local_path = os.path.join(local_output, exp)
        os.makedirs(local_path, exist_ok=True)
        print(f"Fetching {exp} from {remote}...")
        subprocess.run(
            ["scp", "-r", remote_path, local_path + "/"],
            check=False,
        )


def main():
    parser = argparse.ArgumentParser(description="Analyze Terrarium experiments")
    parser.add_argument("--remote", type=str, default=None,
                        help="SSH remote to fetch data from (e.g. pi@100.127.61.93)")
    parser.add_argument("--data-dir", type=str, default=None,
                        help="Local directory containing exp1/exp2/exp3 subdirs")
    args = parser.parse_args()

    if args.data_dir:
        base = args.data_dir
    else:
        base = str(PROJECT_ROOT / "output")

    if args.remote:
        fetch_from_remote(args.remote, base)

    print(f"Loading data from {base}...")

    all_experiments = []

    # Experiment 1
    exp1_dir = os.path.join(base, "exp1_cooperation")
    pname, data1, vals1 = load_sweep(exp1_dir)
    all_experiments.append(("Exp 1: Cooperation", pname, data1, vals1))
    if data1:
        print(f"  Exp1: {len(data1)} runs loaded ({len(vals1)} values)")
        plot_exp1_cooperation(data1, vals1)
    else:
        print(f"  Exp1: No data found in {exp1_dir}")

    # Experiment 2
    exp2_dir = os.path.join(base, "exp2_food_scarcity")
    pname, data2, vals2 = load_sweep(exp2_dir)
    all_experiments.append(("Exp 2: Food Scarcity", pname, data2, vals2))
    if data2:
        print(f"  Exp2: {len(data2)} runs loaded ({len(vals2)} values)")
        plot_exp2_food_scarcity(data2, vals2)
    else:
        print(f"  Exp2: No data found in {exp2_dir}")

    # Experiment 3
    exp3_dir = os.path.join(base, "exp3_brain_cost")
    pname, data3, vals3 = load_sweep(exp3_dir)
    all_experiments.append(("Exp 3: Brain Cost", pname, data3, vals3))
    if data3:
        print(f"  Exp3: {len(data3)} runs loaded ({len(vals3)} values)")
        plot_exp3_brain_cost(data3, vals3)
    else:
        print(f"  Exp3: No data found in {exp3_dir}")

    print_summary_table(all_experiments)
    print(f"\nFigures saved to {OUTPUT_DIR}")


if __name__ == "__main__":
    main()
