#!/usr/bin/env python3
"""Apply the frozen demo settings to a full config, checking the entire TOML tree."""
import copy
from pathlib import Path
import re
import sys
import tomllib

source, destination = map(Path, sys.argv[1:])
text = source.read_text()
expected = copy.deepcopy(tomllib.loads(text))
for section, key, value in [
    ('food', 'regrowth_rate', 0.002), ('creature', 'hard_cap', 500),
    ('food.seasons', 'period', 3600.0), ('food.droughts', 'spawn_rate', 0.0006),
    ('food.droughts', 'duration', 1500), ('daynight', 'period', 600.0),
]:
    node = expected
    for part in section.split('.'):
        node = node[part]
    node[key] = value
    lines = text.splitlines(keepends=True)
    active = False
    count = 0
    for index, line in enumerate(lines):
        if re.match(r'^\s*\[', line):
            active = line.strip().split('#', 1)[0].strip() == f'[{section}]'
        elif active and re.match(rf'^\s*{re.escape(key)}\s*=', line):
            lines[index] = f'{key} = {value}\n'
            count += 1
    if count != 1:
        raise ValueError(f'expected exactly one {section}.{key}, found {count}')
    text = ''.join(lines)
assert tomllib.loads(text) == expected, 'full configuration differs beyond planned overrides'
with destination.open('x') as file:
    file.write(text)
