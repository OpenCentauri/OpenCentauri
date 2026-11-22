# Bed issues

The load cells on the Centauri Carbon are not great, and the post-install autocalibration does not always get it right. If you are getting "probe triggered prior to movement" errors, the mesh looks wrong, or probing behaves oddly, reseating the bed assembly is the fix that has most reliably worked, followed by re-taring.

## Probing fails or the nozzle probes in mid-air

Symptoms include `Probe triggered prior to movement`, runs of `Bad probe detected. Retrying (n/6)...` with wildly varying Z values, `Probe samples exceed samples_tolerance`, or a print that starts several millimetres above the plate.

Work through these in order — the mechanical causes are the most common, and no amount of recalibration fixes them.

1. **Reseat the bed assembly.** Remove and refit the heated bed so all springs seat evenly and the screws are tightened evenly. It is an involved job, but it is the fix that has most reliably resolved these errors. To help with leveling the bed run this command:

    ``` gcode
    MANUAL_LEVEL
    ```
    
2. **Reseat the load cell connectors.** Sometimes unplugging and plugging back in helps.
3. **Re-tare the load cells.** This is safe to run as often as you like:

    ``` gcode
    LOAD_CELL_CALIBRATE TARE=TRUE SAVE=TRUE
    ```

4. **Check `counts_per_gram`.** It should be near `105` for CC1 and `25` for CC2. A value far from that usually means a load cell is dodgy rather than that the calibration is wrong, so recalibrating is unlikely to help.
5. **Check the channels individually.** Run `LOAD_CELL_READ` with an empty bed, then again with a 1-2 kg weight in the middle. All four channels should move in the same direction by broadly similar amounts. A channel with the wrong sign, or one that barely moves, points at a misinstalled or failed cell, which calibration cannot correct.
6. **Try a longer probe pullback.** If the tap fails due to `TAP_PULLBACK_TOO_SHORT` error in `klippy.log`, override the shipped `0.4` in `printer.cfg`:

    ```
    [load_cell_probe]
    pullback_distance: 0.6
    ```

    Values of `0.5` and `0.6` have both helped. Note this is `printer.cfg`, so the syntax is `key: value` with a colon — using `=` here silently does nothing.
7. **Try slower probing moves.** If the tap fails due to `TAP_CHRONOLOGY` or `TAP_SHAPE_INVALID` error in `klippy.log`, override the shipped `2.0` speed in `printer.cfg`:

    ```
    [load_cell_probe]
    speed: 1.0
    ```


## Load cell force safety limit

```
Load Cell Probe Error: current absolute force of 2045.4g exceeds force_safety_limit (+/-2000.0g) before probing!
```

The cells are reading a large standing force before the probe even starts. Reported most often on a hot bed, and it can fail most attempts while a cold first print of the day succeeds.

Unplug the load cell connectors and plug them back in. If that does not hold, the bed assembly or a cell itself is suspect. Recalibrating does **not** fix this — users have completed a clean calibration and still hit it. Do not raise `force_safety_limit` to work around it; it exists to stop the toolhead driving into the plate.

!!! note "This warning is normal"

    `WARNING: Load cell capacity is more than 25Kg! Check wiring and consider using a higher sensor gain.` is expected on this printer and can be ignored. Klipper is not built for four parallel load cells, so the reported capacity is meaningless here.

## Recalibrating the load cells

Try the steps above first, recalibration is worth doing once re-taring has failed. You need something of a known mass to press on the bed. A full spool of filament is roughly 1200 g and works well. The weight must be between 50 g and 25000 g.

1. Clear the bed completely and let the printer sit idle.
2. Start the guided calibration from the console:

    ``` gcode
    LOAD_CELL_CALIBRATE
    ```

3. With nothing on the bed, zero the cells:

    ``` gcode
    TARE
    ```

4. Place your known mass on the bed, centered.
5. Tell the printer what it weighs, in grams. For a 1200 g spool:

    ``` gcode
    CALIBRATE GRAMS=1200
    ```

6. Accept the result:

    ``` gcode
    ACCEPT
    ```

7. Save it, either with the save and restart button in the web interface or by running:

    ``` gcode
    SAVE_CONFIG
    ```


8. [Redo the bed mesh](#redoing-the-bed-mesh) afterwards. The old mesh was probed against the old calibration and is no longer valid.

!!! tip "Checking the result"

    `LOAD_CELL_READ` reports the current force and how much of the sensor range is in use. `LOAD_CELL_DIAGNOSTIC` collects samples and reports sensor health, which is worth running if you suspect a wiring fault rather than a calibration problem.

Your `counts_per_gram` should come out near `105` for CC1 and `25` for CC2. If it lands far from that, suspect a load cell rather than the calibration, and ask for assistance on the [OC Discord server](https://discord.gg/t6Cft3wNJ3).

## Redoing the bed mesh

``` gcode
BED_MESH_CALIBRATE BED_TEMP=60
SAVE_CONFIG
```

Use the temperature you actually print at. Leveling at temperature avoids bed warp and gives noticeably better first layers on materials like ABS. There is also a button for this macro with a dropdown to enter print temperature in the webui.
