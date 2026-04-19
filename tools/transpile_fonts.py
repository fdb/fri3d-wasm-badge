#!/usr/bin/env python3
# Transpile fri3d-runtime/src/fonts.rs (u8g2 font byte arrays) into
# firmware/src/fonts.h as C++ constexpr uint8_t[] tables.
#
# Run from repo root:
#   python3 tools/transpile_fonts.py
#
# Once fri3d-runtime/ is deleted, this script becomes the canonical way to
# refresh the C++ font tables from a u8g2 source dump.

import re
import pathlib

SRC = pathlib.Path('fri3d-runtime/src/fonts.rs')
DST = pathlib.Path('firmware/src/fonts.h')

src_text = SRC.read_text()
pattern = re.compile(r'pub const (\w+): &\[u8\] = &\[(.*?)\];', re.DOTALL)

out = [
    '// Auto-generated from fri3d-runtime/src/fonts.rs. Do not edit by hand.',
    '// Regenerate: python3 tools/transpile_fonts.py',
    '// u8g2 source fonts — see fri3d-runtime/src/fonts.rs for LICENSE.',
    '',
    '#pragma once',
    '',
    '#include <stdint.h>',
    '',
    'namespace fri3d {',
    '',
]

for m in pattern.finditer(src_text):
    name, body = m.group(1), m.group(2)
    byte_tokens = re.findall(r'0x[0-9a-fA-F]+', body)
    out.append(f'// {name}: {len(byte_tokens)} bytes.')
    out.append(f'inline constexpr uint8_t {name}[] = {{')
    per_row = 12
    for i in range(0, len(byte_tokens), per_row):
        out.append('    ' + ', '.join(byte_tokens[i:i + per_row]) + ',')
    out[-1] = out[-1].rstrip(',')
    out.append('};')
    out.append('')

out += ['} // namespace fri3d', '']
DST.write_text('\n'.join(out))
print(f'wrote {DST}')
