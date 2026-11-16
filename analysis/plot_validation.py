"""Plot independent run means and exploratory 95% t intervals, not tick replicates.

Usage: analysis/.venv/bin/python analysis/plot_validation.py SUMMARY_JSON OUTPUT_PNG
Requires matplotlib and scipy. The selection rule remains in summarize_validation.py.
"""
import json
from pathlib import Path
import statistics
import sys

import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt
from scipy.stats import t


def main(summary_path, output_path):
    summary = json.loads(Path(summary_path).read_text())
    if not summary['complete']:
        raise ValueError('Only a completed batch may be plotted as validation')
    fig, ax = plt.subplots(figsize=(9, 5), layout='constrained')
    ax.axhspan(800, 1300, color='#dceedd', label='Predefined target range')
    for index, group in enumerate(summary['by_rate']):
        values = [r['late_mean_population'] for r in summary['runs']
                  if r['regrowth_rate'] == group['regrowth_rate'] and r['survived']]
        n = len(values)
        mean = statistics.mean(values)
        half = t.ppf(0.975, n - 1) * statistics.stdev(values) / n**0.5
        # Fixed offsets expose every independent run without suggesting a seed trend.
        offsets = [(i - (n - 1) / 2) * 0.025 for i in range(n)]
        ax.scatter([index + d for d in offsets], values, color='#367d91', alpha=0.7,
                   s=30, label='One run (late mean)' if index == 0 else None)
        ax.errorbar(index, mean, yerr=half, fmt='D', color='#182d3c',
                    capsize=7, markersize=7, linewidth=2,
                    label='Mean and 95% t interval' if index == 0 else None)
        print(f"rate={group['regrowth_rate']:.3f}, n={n}, mean={mean:.2f}, "
              f"95% CI=[{mean-half:.2f}, {mean+half:.2f}]")
    ax.set_xticks(range(len(summary['by_rate'])),
                  [f"{r['regrowth_rate']:.3f}" for r in summary['by_rate']])
    counts = ', '.join(str(r['seeds']) for r in summary['by_rate'])
    ax.set(xlabel='Food regrowth per cell per tick', ylabel='Late mean population',
           title=f'Food calibration · independent run counts: {counts}', ylim=(750, 1500))
    ax.spines[['top', 'right']].set_visible(False)
    ax.legend(loc='upper left', frameon=False, fontsize=9)
    source = summary['launched_source_revision'][:7]
    fig.supxlabel(f"Pinned source {source} · {summary['target_ticks']:,} ticks · "
                  f"cap {summary['criteria']['hard_cap']:,}\n"
                  f"Late window: {summary['windows']['late']}", fontsize=9)
    fig.savefig(output_path, dpi=180)
    plt.close(fig)


if __name__ == '__main__':
    main(*sys.argv[1:])
