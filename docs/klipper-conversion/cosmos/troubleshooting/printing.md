# Printing and slicer issues

## The printer emergency stops as soon as a print starts

```
Shutdown due to M112 command
Printer is shutdown
```

This is deliberate. COSMOS 26.07.0 moved printing into `PRINT_START`/`PRINT_END` macros, and the old Elegoo/OpenCentauri commands `M729` and `M8213` now trigger an emergency stop on purpose, to tell you the slicer profile is out of date. Import the current profile — see [the required OrcaSlicer profile](../install.md#required-orcaslicer-profile). CANVAS users need the AFC variant of the profile, which carries the extra tool change G-code.

If you maintain your own start G-code, the machine start block should call `PRINT_START` and pass the temperatures through:

``` gcode
PRINT_START EXTRUDER=[first_layer_temperature] BED=[first_layer_bed_temperature] CHAMBER=[chamber_temperature]
```

## Reprinting an old file fails

Files sliced before the macro change carry the old parameter names and will error out — the parameters are now `EXTRUDER`, `BED`, and `CHAMBER`. Reslice with the current profile rather than reprinting from history. Overriding `PRINT_START` or `PRINT_END` in your own `printer.cfg` causes the same class of failure.

## The purge line lands in the middle of the bed

That is how adaptive purging works — the purge is placed near the print and moves with it. It can collide with the model when supports sit in front of it, because the placement does not account for supports. To opt out, set `adaptive_purge = False` in `cosmos.conf` and add your own purge line in the slicer.

## The screen shows a grey block instead of the print preview

The G-code thumbnail is at a resolution the screen cannot render. Set the thumbnail to `256x256` PNG in the slicer, or leave it at the stock `144x144`.
