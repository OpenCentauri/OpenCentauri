# Troubleshooting

Fixes for common problems on COSMOS. Most of what follows is run from the console in the web interface, or by editing a config file there.

!!! note "Where the config files are"

    `cosmos.conf` and `printer.cfg` sit next to each other in the config editor of the web interface. `cosmos.conf` holds COSMOS settings, `printer.cfg` holds your Klipper configuration.

    You should only edit printer.cfg and add sections as needed following the [Kalico reference](https://docs.kalico.gg/Config_Reference.html). edits to the read only config will be overwritten on update!

!!! tip "Check your version first"

    A large share of reported problems turn out to be already fixed. Before working through anything below, confirm which build you are on and update if you are behind. Some fixes reach the nightly channel well before a stable release.

## Where to look

- [Bed issues](bed.md) — probing failures, the load cell force safety limit, recalibrating the load cells, redoing the bed mesh
- [Printing and slicer issues](printing.md) — emergency stops at print start, old files failing, purge placement, screen previews
- [Stability and memory](stability.md) — `Timer too close`, MCU shutdowns, calibration freezing partway through
- [Z offset](z-offset.md) — saving an offset with nozzle Z homing enabled
- [CANVAS and AFC](canvas.md) — hub not clear, wrong colours, cover detection on non-CANVAS printers
- [Configuration and recovery](recovery.md) — replacing a toolhead or bed board
- [Common questions](questions.md) — adaptive meshing, update channels, plugins, screen sleep, PID tuning

If none of it helps, ask in the `#COSMOS_development` channel on the [OpenCentauri Discord](https://discord.gg/t6Cft3wNJ3).
