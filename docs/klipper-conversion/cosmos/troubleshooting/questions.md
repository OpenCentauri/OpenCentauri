# Common questions

??? question "**How do I enable adaptive meshing?**"

    Adaptive meshing is not enabled by default. If you always print with the same material and you don't swap build plates often then any benefit it provides may be negligible and it will slow down printing times.
    
    If you want to enable it go to `cosmos.conf`, in the config editor next to `printer.cfg`:

    ```
    [klipper]
    # Probe only the print area for each print instead of loading the default bed mesh. Requires nozzle_z_homing to be enabled.
    adaptive_mesh = True
    ```

    It requires `nozzle_z_homing` to be enabled as well, so set both:

    ```
    nozzle_z_homing = True
    ```

    Reboot the whole machine afterwards. With adaptive meshing off, COSMOS loads the saved `default` mesh instead of probing the area each print uses.

??? question "**I changed something in `cosmos.conf` and nothing happened**"

    Some changes in cosmos.conf only take effect on a full boot. Power cycle the printer or run `REBOOT_MACHINE`, then check again.

??? question "**Klipper won't start after enabling CANVAS**"

    If CANVAS hardware is detected while its configuration is not loaded klippy will not start. Confirm `elegoo_canvas = True` under `[extras]` in `cosmos.conf`, and that you are on a build that includes CANVAS support. See the [AMS support](../ams.md#elegoo-canvas) page.

??? question "**Why doesn't ___ update people are talking about show up?**"

    Check the `[update]` section of `cosmos.conf`. `release` selects the channel; `stable` follows tagged releases, `nightly` follows the latest build. Some features reach nightly well before they appear in a stable release. `check_for_updates` controls whether COSMOS looks for updates on startup at all.

??? question "**`CHECK_CALIBRATION` says something isn't calibrated**"

    Run `CHECK_CALIBRATION` and pick `Calibrate All` to run the whole routine. If the routine fails partway through on your machine, the [features page](../features.md#manual-calibration) has each step broken out so you can run only the part that failed.

??? question "**Can I install a Klipper plugin to fix this?**"

    Probably not. The stock mainboard is extremely resource limited and there is very little headroom for anything outside standard Klipper/Kalico. Adding plugins is a common cause of problems rather than a fix for them.

??? question "**How do I stop the screen turning off after 10 minutes?**"

    That is the screen's own setting rather than a COSMOS one — `cosmos.conf` only controls brightness. Edit `grumpyscreen.cfg` and set the sleep timeout to never:

    ```
    [ui]
    display_sleep_sec: -1
    ```

    Then use the restart GUI button for it to take effect.

??? question "**PID calibration keeps getting interrupted**"

    `calibration did not finish in time` during extruder PID tuning was caused by the extruder `smooth_time` being set too high. Current builds sets `smooth_time: 1.5`, so update first. If you are still affected, override it in `printer.cfg`:

    ```
    [extruder]
    smooth_time: 1.5
    ```

    While we have found 1.5 is a good consensus option for stock and available aftermarket alternative hotends every printer is different. If you are having issues getting issues with 1.5 second smoothing time you may want to increase or decrease it slightly. Report results to the [OpenCentauri Discord](https://discord.gg/t6Cft3wNJ3).

??? question "**Something else is broken**"

    [Open an issue on GitHub](https://github.com/OpenCentauri/cosmos/issues) with a description of what happened and how to reproduce it, or ask in the `#COSMOS_development` channel on the [OpenCentauri Discord](https://discord.gg/t6Cft3wNJ3).
