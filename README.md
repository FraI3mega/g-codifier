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
`0xF3` followed by u132 byte count = 5 bytes