"""Plot every run; no tick-level uncertainty intervals or omitted failures.
analysis/.venv/bin/python analysis/plot_memory_growth.py SUMMARY_JSON OUTPUT_PNG
"""
import json
from pathlib import Path
import sys
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt
from matplotlib.lines import Line2D

s = json.loads(Path(sys.argv[1]).read_text())
if not s['complete']:
    raise ValueError('refusing to present an unfinished batch')
fig, axes = plt.subplots(4, 2, figsize=(12, 13), layout='constrained')
metrics = [('population', 'Living population'), ('mean_connections', 'Enabled connections / living creature'),
           ('mean_recurrent_connections', 'Enabled recurrent connections / living creature'),
           ('recurrent_brain_fraction', 'Fraction with recurrent links'),
           ('mean_brain_activity', 'Raw weighted-signal activity (pre-action)'),
           ('total_cap_blocked_births', 'Cumulative cap-blocked births'),
           ('total_births', 'Cumulative births'), ('total_deaths', 'Cumulative deaths')]
styles = ['-', '--', ':', '-.']
for ax, (key, label) in zip(axes.flat, metrics):
    for run in s['trajectories']:
        rows = run['rows']
        color = '#d66b2c' if run['recurrence_probability'] else '#29789e'
        ax.plot([r['tick'] / 1000 for r in rows], [r[key] for r in rows], color=color,
                linestyle=styles[run['seed'] - 3000], linewidth=1, alpha=.8)
    ax.axvspan(150, 200, color='#888888', alpha=.10)
    ax.set(title=label, xlabel='Tick (thousands)')
    ax.spines[['top', 'right']].set_visible(False)
    ax.grid(alpha=.15)
    if key == 'recurrent_brain_fraction':
        ax.set_ylim(-.03, 1.03)
handles = [Line2D([0], [0], color=color, label=label, lw=2) for color, label in
           [('#29789e', 'Recurrence mutation off'), ('#d66b2c', 'Recurrence mutation 0.3')]]
handles += [Line2D([0], [0], color='#555555', linestyle=style, label=f'Seed {seed}')
            for seed, style in zip(range(3000, 3004), styles)]
fig.legend(handles=handles, loc='outside upper center', ncols=3, frameon=False,
           title='Demo-setting structural-growth pilot · all eight runs', title_fontsize=14)
fig.supxlabel('Pinned engine 0da954f (dirty build recorded) · 200,000 ticks · demo cap 500\n'
              'Shaded: inclusive 150k–200k late window. Structural presence is not evidence of useful memory.', fontsize=10)
fig.savefig(sys.argv[2], dpi=170)
plt.close(fig)
