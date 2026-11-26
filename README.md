# g-codifier

A little tool for hiding data in G-code

## How It Works

A G-code file is made up many lines, each containing one command for the machine.

```gcode
G1 X129.283 Y113.963 E.02253
G1 X136.037 Y120.717 E.34412
G1 X136.037 Y121.342 E.02253
G1 X128.658 Y113.963 E.37598
G1 X128.032 Y113.963 E.02253
G1 X136.037 Y121.968 E.40784
G1 X136.037 Y122.593 E.02253
```

By adding or modifying the least significant digit of the cordinates we can hide data in them.

```gcode
G1 X129.283|1 Y113.963|1 E.02253|1
G1 X136.037|1 Y120.717|1 E.34412|1
G1 X136.037|1 Y121.342|1 E.02253|1
G1 X128.658|1 Y113.963|1 E.37598|1
G1 X128.032|1 Y113.963|1 E.02253|1
G1 X136.037|1 Y121.968|1 E.40784|1
G1 X136.037|1 Y122.593|1 E.02253|1
```

data is encoded as octal digits, storing 8 bits of data and 1 parity bit per row.

Preamble
`0xF3` followed by u32 byte count = 5 bytes


# Review TODO

- [ ] **High** — `src/encoded.rs:39-42` — `encode_line` drops all content outside X/Y/E (feedrate, extra fields, comments).
- [ ] **High** — `src/main.rs:62-64`, `86` — encode truncates the input when `input` == `output`; same pattern in decode (`105-106`).
- [ ] **Medium** — `src/encoded.rs:6` — regex greedily captures X/Y/E from trailing comments instead of the command.
- [ ] **Medium** — `src/main.rs:106` — decode creates/truncates the output data file before validating input.
- [ ] **Medium** — `src/main.rs:64`, `86-102` — encode leaves a corrupt partial output file when data doesn't fit.
- [ ] **Medium** — `src/main.rs:138-139` — payload allocation uses an unvalidated 4-byte header length.
- [ ] **Low** — `src/main.rs:138-150` — no check that declared payload bytes were collected; keeps reading lines after `data_len` hits 0.
- [ ] **Low** — `src/main.rs:155`, `159`, `162` — `Capacity` can print a negative value.
- [ ] **Low** — `src/main.rs:78`, `81`, `83` — compressed length truncated to `u32` for output ≥ 4 GiB.
- [ ] **Low** — `src/main.rs:110-120` vs `134-136` — wrong error message when no encoded lines exist.
- [ ] **Low** — `README.md:34` — documents "u132" instead of `u32`.
