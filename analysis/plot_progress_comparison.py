"""Plot all three paired searches, true training scores and final distributions.
analysis/.venv/bin/python analysis/plot_progress_comparison.py SUMMARY OUTPUT_PNG
"""
import json
from pathlib import Path
import sys
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt

summary = json.loads(Path(sys.argv[1]).read_text())
if not summary['complete'] or summary['paired_seed_count'] != 3:
    raise ValueError('expected complete fixed paired pilot')
fig, axes = plt.subplots(3, 2, figsize=(11, 10), layout='constrained')
colors = {'terminal_only': '#29789e', 'terminal_progress': '#d66b2c'}
labels = {'terminal_only': 'Terminal-only control', 'terminal_progress': 'Progress tiebreak'}
for row, seed in zip(axes, [2100, 2101, 2102]):
    curve, counts = row
    for mode in colors:
        run = next(r for r in summary['modes'][mode]['runs'] if r['seed'] == seed)
        generations = run['generations']
        curve.step([g['generation'] for g in generations], [g['best_mean_reward'] for g in generations],
                   where='post', label=labels[mode], color=colors[mode])
        offset = -.18 if mode == 'terminal_only' else .18
        counts.bar([v + offset for v in range(3)],
                   [run['final_score_counts'][str(v)] for v in range(3)],
                   width=.36, color=colors[mode], label=labels[mode])
    curve.set(title=f'Paired search seed {seed}', xlabel='Generation (0 = founders)',
              ylabel='Best true visible-trial reward', ylim=(-.04, 1.04), yticks=[0, .5, 1])
    counts.set(title='All final controllers', xlabel='Correct choices / two fixed visible trials',
               ylabel='Controllers', xticks=[0, 1, 2], ylim=(0, 67))
    curve.legend(fontsize=8, frameon=False)
    for ax in row:
        ax.spines[['top', 'right']].set_visible(False)
        ax.grid(axis='y', alpha=.15)
fig.suptitle('Terminal progress tiebreak · all three declared seed pairs', fontsize=14)
fig.supxlabel('64 founders · 25 replacements per arm · identical initial genotypes by ID\n'
              'True correct count stays primary. Fixed training trials only; no held-out or memory-benefit test.', fontsize=10)
fig.savefig(sys.argv[2], dpi=170)
plt.close(fig)
