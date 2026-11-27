# CANVAS and AFC

## The print pauses after the purge, or reports the hub is not clear

```
CANVAS_1 Hub not clear when trying to load.
Please check that hub does not contain broken filament and is clear
```

Migrating to COSMOS does not carry CANVAS state across, so filament that is physically loaded is invisible to AFC. Heat the extruder from the web interface, run the unload filament macro until the toolhead is definitely clear, then set your lanes up in the web interface. Alternatively force the lane's state with `SET_LANE_LOADED LANE=CANVAS_1`, substituting the lane you actually have loaded.

`BT_TOOL_UNLOAD` and `BT_MOVE` will not help here, because as far as AFC is concerned no lane is loaded.

## The print comes out in the wrong colours

Tool-to-colour mapping cannot be remapped in OrcaSlicer — `T0` is always `T0`. Either upload from Orca and then **start** the print from the web interface, where the mapping menu lives, or use Orca's sync filaments button. Syncing only works once the spools are added in AFC, otherwise there is nothing to map to.

## "Front cover has fallen off" on a printer without a CANVAS

The cover sensor is part of the CANVAS configuration, so enabling AFC for another unit such as a Box Turtle brings it along. Disable it in `printer.cfg`:

```
[!gcode_button toolhead_front_cover_detection]
```
