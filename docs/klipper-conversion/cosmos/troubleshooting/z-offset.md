# Z offset

## `Z_OFFSET_APPLY_ENDSTOP is not supported when nozzle z homing is enabled`

With `nozzle_z_homing` enabled the Z offset belongs to the printer rather than the slicer:

1. Set the Z offset in your slicer to `0`.
2. Run the calibrate Z offset macro, or a full calibration, and save.
3. For a one-off adjustment, babystep during a print and press save in the web interface. The value is written into the `SAVE_CONFIG` block at the bottom of `printer.cfg` — leave that block alone otherwise, it is managed by the firmware.

Any change to the Z offset invalidates the bed mesh, so [redo the mesh](bed.md#redoing-the-bed-mesh) afterwards.

You can of course babystep z offset from the webui or printer screen and this change will persist between prints untill the machine is powercycled.
