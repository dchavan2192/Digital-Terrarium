"""Plot actual movement traces from `sim-runner delayed-choice` JSON.

Usage: analysis/.venv/bin/python analysis/plot_delayed_choice.py CONTROLS_JSON OUTPUT_PNG
"""
import json
import math
from pathlib import Path
import sys

import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt
from matplotlib.patches import Circle


def main(source, destination):
    result = json.loads(Path(source).read_text())
    if not result.get('instrument_validated') or result.get('assay') != 'delayed_choice_controls':
        raise ValueError('Expected a validated manufactured movement-control report')
    config = result['arena']
    if len(result['controls']) != 3:
        raise ValueError('Expected all three declared controls')
    points = [(config['start_x'], config['start_y'])]
    for name in ('left', 'right'):
        x, y, radius = config[f'goal_{name}_x'], config[f'goal_{name}_y'], config['goal_radius']
        points.extend([(x-radius, y-radius), (x+radius, y+radius)])
    points.extend((step['pose_after']['x'], step['pose_after']['y'])
                  for control in result['controls'] for trial in control['trials']
                  for step in trial['trajectory'])
    xlim = (min(p[0] for p in points)-0.75, max(p[0] for p in points)+0.75)
    ylim = (min(p[1] for p in points)-0.75, max(p[1] for p in points)+1.0)
    titles = {'recurrent_reference': 'Intact recurrent reference',
              'feed_forward_control': 'Feed-forward control',
              'same_network_state_cleared': 'Same network, state cleared'}
    fig, axes = plt.subplots(1, 3, figsize=(12, 4.6), layout='constrained')
    for ax, control in zip(axes, result['controls']):
        for name in ('left', 'right'):
            x, y = config[f'goal_{name}_x'], config[f'goal_{name}_y']
            ax.add_patch(Circle((x, y), config['goal_radius'], facecolor='#eeeeee',
                                edgecolor='#777777', linewidth=1.2))
            ax.text(x, y + config['goal_radius'] + 0.3, f'{name.capitalize()} goal',
                    ha='center', fontsize=8)
        for trial in control['trials']:
            poses = [trial['initial_pose']] + [step['pose_after'] for step in trial['trajectory']]
            positive = trial['cue'] == 1
            ax.plot([p['x'] for p in poses], [p['y'] for p in poses],
                    color='#1a8a71' if positive else '#7053a5',
                    linestyle='--' if positive else '-', linewidth=2,
                    label=f"Cue {trial['cue']:+d}: {trial['choice']}, reward {trial['reward']}")
            ax.scatter(poses[-1]['x'], poses[-1]['y'], s=18, color='#1a8a71' if positive else '#7053a5')
        ax.scatter(config['start_x'], config['start_y'], s=20, color='#263544', zorder=5)
        ax.annotate('', xy=(config['start_x']+0.6*math.cos(config['start_heading']),
                            config['start_y']+0.6*math.sin(config['start_heading'])),
                    xytext=(config['start_x'], config['start_y']),
                    arrowprops={'arrowstyle': '->', 'color': '#263544'})
        ax.set(title=f"{titles[control['name']]}\nMean reward {control['mean_reward']:.1f}",
               xlabel='Task x', ylabel='Task y', xlim=xlim, ylim=ylim)
        ax.set_aspect('equal')
        ax.spines[['top', 'right']].set_visible(False)
        ax.legend(loc='upper center', bbox_to_anchor=(0.5, -0.22), fontsize=7, frameon=False)
    visible = result.get('cue_visible', False)
    mode = 'visible cue' if visible else 'delayed cue'
    fig.suptitle(f'Delayed-choice movement task · {mode} · manufactured controls', fontsize=13)
    caption = ('Cue remains visible during movement; both goal markers look identical.\n'
               'No training or evolution: stimulus-response competence control.' if visible else
               'Cue removed before release; both goal markers look identical.\n'
               'No training or evolution: this validates a memory-dependent task.')
    fig.supxlabel(caption, fontsize=9)
    fig.savefig(destination, dpi=180)
    plt.close(fig)


if __name__ == '__main__':
    main(*sys.argv[1:])
