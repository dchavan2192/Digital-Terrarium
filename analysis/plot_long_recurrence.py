#!/usr/bin/env python3
"""Plot every audited long-study world; no pooled-tick confidence intervals."""
import argparse
import json
from pathlib import Path
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt
from matplotlib.lines import Line2D

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('summary', type=Path)
parser.add_argument('output', type=Path)
args = parser.parse_args()
data = json.loads(args.summary.read_text())
if not data['complete'] or len(data['runs']) != 8:
    raise ValueError('This final-study figure requires all eight audited worlds')
fig, axes = plt.subplots(3, 2, figsize=(12, 10), layout='constrained')
metrics = [('population', 'Living population'),
           ('mean_connections', 'Enabled total links / living creature'),
           ('mean_recurrent_connections', 'Enabled recurrent links / living creature'),
           ('recurrent_brain_fraction', 'Fraction with recurrent links'),
           ('mean_brain_activity', 'Raw weighted-signal activity (pre-action)'),
           ('total_cap_blocked_births', 'Cumulative cap-blocked births')]
styles = ['-', '--', ':', '-.']
for ax, (metric, label) in zip(axes.flat, metrics):
    for run in data['runs']:
        rows = run['trajectory']
        color = '#bb5727' if run['recurrence_probability'] else '#267a9c'
        ax.plot([r['tick'] / 1000 for r in rows], [r[metric] for r in rows],
                color=color, linestyle=styles[run['seed'] - 4000], linewidth=1, alpha=.85)
    ax.axvspan(50, 100, color='#9bb7c4', alpha=.15)
    ax.axvspan(375, 500, color='#999999', alpha=.12)
    ax.set(title=label, xlabel='Tick (thousands)', xlim=(0, 500))
    ax.spines[['top', 'right']].set_visible(False)
    ax.grid(alpha=.15)
    if metric == 'recurrent_brain_fraction':
        ax.set_ylim(-.03, 1.03)
    if metric == 'total_cap_blocked_births' and all(
            r[metric] == 0 for run in data['runs'] for r in run['trajectory']):
        ax.set_ylim(0, 1)
handles = [Line2D([0], [0], color=color, label=label, lw=2) for color, label in
           [('#267a9c', 'Recurrence add 0'), ('#bb5727', 'Recurrence add 0.3')]]
handles += [Line2D([0], [0], color='#555555', linestyle=style, label=f'Seed {seed}')
            for seed, style in zip(range(4000, 4004), styles)]
fig.legend(handles=handles, loc='outside upper center', ncols=3, frameon=False,
           title='Baseline recurrence-pressure pilot · all eight completed worlds', title_fontsize=14)
fig.supxlabel('Pinned b40ae9c engine · food 0.008 / cap 2,000 · 500,000 ticks\n'
              'Shaded: fixed early 50k–100k and late 375k–500k windows. Structural presence does not establish useful memory.', fontsize=10)
fig.savefig(args.output, dpi=170)
plt.close(fig)
