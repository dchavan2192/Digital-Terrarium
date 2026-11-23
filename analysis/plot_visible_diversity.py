#!/usr/bin/env python3
"""Plot all eight audited champions without treating cue trials as replicates."""
import argparse
import json
from pathlib import Path
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt
from matplotlib.colors import ListedColormap

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('summary', type=Path)
parser.add_argument('output', type=Path)
args = parser.parse_args()
data = json.loads(args.summary.read_text())
rows = sorted(data['rows'], key=lambda r: (r['seed'], r['arm']))
if not data['complete'] or len(rows) != 8 or data['available_pairs'] != 4:
    raise ValueError('Requires the complete audited four-pair pilot')
values = [[sum(t['reward'] for t in c['trials']) for c in r['cases']] for r in rows]
fig, ax = plt.subplots(figsize=(11, 6), layout='constrained')
ax.imshow(values, cmap=ListedColormap(['#f1e1d5', '#bdd8dc', '#408b96']), vmin=-.5, vmax=2.5, aspect='auto')
for y, r in enumerate(rows):
    for x, c in enumerate(r['cases']):
        outcomes = ''.join('C' if t['reward'] else 'T' if t['choice'] == 'timeout' else 'W' for t in sorted(c['trials'], key=lambda t: t['cue']))
        ax.text(x, y, f'{values[y][x]}/2  {outcomes}', ha='center', va='center', fontsize=11, color='white' if values[y][x] == 2 else '#19282b')
ax.set_xticks(range(6), ['Forward\nshift', 'Wide\nshift', 'Near\nnarrow', 'Far\ndiagonal', 'Heading\n−30°', 'Heading\n+30°'])
ax.set_yticks(range(8), [f"{r['seed']} · {r['arm'].capitalize()} · train {r['training_correct']}/6" for r in rows])
ax.axvline(3.5, color='white', linewidth=4)
for y in [1.5, 3.5, 5.5]:
    ax.axhline(y, color='white', linewidth=3)
ax.tick_params(length=0)
ax.spines[:].set_visible(False)
ax.set_title('Fixed versus varied training · all eight selected champions\nPrimary geometry: fixed 12/32, varied 14/32 · headings: both 10/16', pad=18, fontsize=14)
fig.supxlabel('C = correct, W = wrong goal, T = timeout; cue order −1 then +1. Each cell is two responses.\nFour seed pairs; mixed differences (−0.25, 0, +0.375, +0.125) do not establish a reliable benefit.\nPinned c7324b6 · visible cue · feed-forward · six held-outs consumed by this pilot.', fontsize=10)
fig.savefig(args.output, dpi=170)
plt.close(fig)
