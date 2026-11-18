#!/usr/bin/env python3
"""Plot every saved policy and frozen case without pooling cohorts."""
import argparse
import json
from pathlib import Path
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt
from matplotlib.colors import ListedColormap, BoundaryNorm

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('summary', type=Path)
parser.add_argument('output', type=Path)
args = parser.parse_args()
data = json.loads(args.summary.read_text())
cases = ['training_reference', 'rotation_minus_45', 'rotation_plus_45',
         'near', 'far_forward', 'narrow', 'wide']
keys = [(cohort, seed) for cohort, seeds in [('original', [2000, 2001, 2002]),
        ('terminal_only', [2100, 2101, 2102]), ('terminal_progress', [2100, 2101, 2102])]
        for seed in seeds]
lookup = {(r['cohort'], r['seed'], r['case']): r for r in data['policies']}
matrix = [[lookup[(cohort, seed, case)]['correct'] for case in cases] for cohort, seed in keys]
fig, ax = plt.subplots(figsize=(10.5, 6.2))
ax.imshow(matrix, cmap=ListedColormap(['#efe9e4', '#e8bf70', '#63b6a2']),
          norm=BoundaryNorm([-.5, .5, 1.5, 2.5], 3), aspect='auto')
for i, row in enumerate(matrix):
    for j, count in enumerate(row):
        trials = lookup[(*keys[i], cases[j])]['trials']
        outcomes = '/'.join('T' if t['choice'] == 'timeout' else ('C' if t['reward'] else 'W') for t in trials)
        ax.text(j, i, f'{count}/2\n{outcomes}', ha='center', va='center', fontsize=10)
ax.set_xticks(range(7), ['Training\nreference', 'Rotate\n−45°', 'Rotate\n+45°',
                         'Near', 'Far forward', 'Narrow', 'Wide'])
ax.set_yticks(range(9), [f'{cohort.replace("_", " ")} · {seed}' for cohort, seed in keys])
for y in (2.5, 5.5):
    ax.axhline(y, color='white', linewidth=4)
ax.axvline(.5, color='white', linewidth=3)
ax.axvline(2.5, color='white', linewidth=3)
ax.set_title('Visible-task competence is brittle across changed goal geometries', loc='left', pad=15)
ax.tick_params(length=0)
for spine in ax.spines.values():
    spine.set_visible(False)
fig.text(.02, .035, 'All 9 training-selected policies retained. Cells: correct cues out of 2; cue −1 / +1 outcomes.\n'
         'C = correct, W = wrong goal, T = timeout. Fixed manufactured references solve all 7 cases.\n'
         'Saved-policy diagnostic; no fresh-seed, memory or ecological-benefit inference.', fontsize=10)
fig.tight_layout(rect=[0, .14, 1, 1])
fig.savefig(args.output, dpi=180)
