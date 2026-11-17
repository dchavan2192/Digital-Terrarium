"""Plot every declared pilot seed, training curves and founder/final score counts.
analysis/.venv/bin/python analysis/plot_acquisition.py SUMMARY_JSON OUTPUT_PNG
"""
import json
from pathlib import Path
import sys
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt

s = json.loads(Path(sys.argv[1]).read_text())
if not s['complete'] or [r['seed'] for r in s['runs']] != [2000, 2001, 2002]:
    raise ValueError('expected complete fixed three-seed pilot')
fig, axes = plt.subplots(3, 2, figsize=(11, 10), layout='constrained')
for row, run in zip(axes, s['runs']):
    curve, counts = row
    generations = run['generations']
    curve.step([g['generation'] for g in generations], [g['best_mean_reward'] for g in generations],
               where='post', color='#2b769c', label='Best training reward')
    curve.plot([g['generation'] for g in generations], [g['population_mean_reward'] for g in generations],
               color='#c86a2c', label='Population mean')
    curve.set(title=f"Search seed {run['seed']}", xlabel='Generation (0 = random founders)',
              ylabel='Balanced visible-trial mean reward', ylim=(-.04, 1.04), yticks=[0, .5, 1])
    curve.legend(fontsize=8, frameon=False)
    x = [0, 1, 2]
    counts.bar([v - .18 for v in x], [run['founder_score_counts'][str(v)] for v in x],
               width=.36, label='Founders', color='#8cb3ca')
    counts.bar([v + .18 for v in x], [run['final_score_counts'][str(v)] for v in x],
               width=.36, label='Final population', color='#d8955f')
    counts.set(title=f"Founder best {run['founder_best']:.1f} → final best {run['final_best']:.1f}",
               xlabel='Correct choices / two fixed visible trials', ylabel='Controllers',
               xticks=x, ylim=(0, 67))
    counts.legend(fontsize=8, frameon=False)
    for ax in row:
        ax.spines[['top', 'right']].set_visible(False)
        ax.grid(axis='y', alpha=.15)
fig.suptitle('Visible-cue acquisition feasibility pilot · all three declared searches', fontsize=14)
fig.supxlabel('Fixed source 5504a20 · 64 founders · 25 replacements · terminal fitness only\n'
              'Training performance: no held-out validation, delayed task or memory-benefit test.', fontsize=10)
fig.savefig(sys.argv[2], dpi=170)
plt.close(fig)
