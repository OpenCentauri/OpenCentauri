# Stability and memory

The mainboard has 128 MB of RAM and sits at high utilisation even when idle, so most stability failures are resource exhaustion rather than logic bugs.

## `Timer too close` or an MCU shutdown mid-print

In typical use without additional plugins COSMOS is very stable at this point but memory is still very limited so extra load can percipitate errors. There is no complete fix for this, it is a limitation of the hardware. What reduces it:

- Keep the bed mesh at the default size. Raising it to something like 30x30 reliably causes crashes.
- Do not use adaptive or dynamic pressure advance.
- Do not install additional Klipper plugins or alternative screen software. HelixScreen in particular is known to crash the printer, as it uses considerably more memory than grumpyscreen.

Running a CANVAS makes this more likely, because AFC adds to the load. 

## Calibration freezes or the screen goes dark partway through

Usually at the resonance and input shaper step, which is the heaviest thing the printer does. Power cycle and run it again — it often succeeds on the second or third attempt. To retry only that step rather than the whole routine, run `SHAPER_CALIBRATE`.

If a failed calibration left the printer in a bad state, COSMOS keeps a copy of your previous `printer.cfg` in the `config-backups` folder.

!!! note "The screen and camera turning off during calibration is normal"

    Both the GUI and camera services are stopped during input shaper calibration and are restarted afterwards.
