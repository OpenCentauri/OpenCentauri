# COSMOS Extra Features

This page outlines features specific to COSMOS (not to Klipper!), like how configuration works, what macros we added and what recovery options we added.

## Custom Macros

### CHECK_CALIBRATION

Checks whether the input shaper, default bed mesh, load cells, and extruder PID have been calibrated. If anything is missing, select `Calibrate All` in the prompt to run the complete calibration routine.

The routine uses `full_calibrate_hotend_temperature` and `full_calibrate_bed_temperature` from `cosmos.conf`; it no longer accepts temperature arguments in the macro command.

Example: `CHECK_CALIBRATION`

### BED_MESH_CALIBRATE

Heats the bed to the requested temperature, homes the printer, cleans the nozzle, calibrates the load-cell Z home, and creates a bed mesh. It also calibrates the Z offset when nozzle Z homing is disabled.

Args:

- `BED_TEMP`: number. Optional; defaults to the bed's current target or 60 °C.

Example: `BED_MESH_CALIBRATE BED_TEMP=60`

Run `SAVE_CONFIG` after the calibration to save the new mesh.

### LOAD_FILAMENT
Loads filament into the toolhead. After loading, a prompt asks if you want to extrude more.

Args:

- EXTRUDER_TEMP: number. Optional.

Example: `LOAD_FILAMENT EXTRUDER_TEMP=250`

### UNLOAD_FILAMENT
Unloads filament from the toolhead. This utilises the filament cutter and keeps the extruder cold.

Example: `UNLOAD_FILAMENT`

## Klipper Configuration

COSMOS uses Klipper under the hood, and allows the user to edit its configuration. The configuration can be edited via the webui.

Relevant files are:

- `printer.cfg`: User facing configuration. Add your custom Klipper macros in here. SAVE_CONFIG saves its configuration to this file as well.
- `klipper-readonly/*.cfg`: System configuration managed by COSMOS. Do not edit these files because firmware updates reset them and discard any changes. To override a section, copy the relevant section into `printer.cfg` and edit it there.

## COSMOS settings

Edit `cosmos.conf` from the web interface to configure COSMOS. The file contains the available values and descriptions for settings including:

- the built-in screen and web UI
- update checks and the release channel
- camera and toolhead LEDs
- print heat soak, adaptive meshing, and adaptive purging
- nozzle Z homing and the end-of-print bed position
- automatic calibration temperatures

Save the file, then reboot the entire printer for changes to take effect. A Klipper firmware restart alone does not reload these settings; power cycle the printer or run `REBOOT_MACHINE`.

## Manual calibration

`CHECK_CALIBRATION` normally completes every initial calibration in one routine. Sometimes errors can prevent some printers from reaching the end of the calibration routine. If that happens, use the tabs below to run only the missing parts from the web interface's console.

The examples use the default 250 °C hotend and 60 °C bed calibration temperatures. If you changed `full_calibrate_hotend_temperature` or `full_calibrate_bed_temperature` in `cosmos.conf`, use those values instead.

=== "Input shaper"

    Remove loose objects from the bed and chamber before starting. These commands calibrate and save each axis independently:

    ``` gcode
    G28
    ACCELEROMETER_DEBUG_READ CHIP=lis2dw REG=0x0F
    SHAPER_CALIBRATE AXIS=X FORCE_SHAPER=mzv
    SAVE_CONFIG RESTART=0
    SHAPER_CALIBRATE AXIS=Y FORCE_SHAPER=mzv
    SAVE_CONFIG
    ```

    `SAVE_CONFIG RESTART=0` after the X axis preserves its successful result without intentionally restarting before the Y calibration. If X succeeds but Y fails, X remains saved and you only need to retry the Y command after the next boot. The final `SAVE_CONFIG` saves the Y result and restarts Klipper normally.

=== "Extruder PID"

    These commands move the toolhead over the purge tray, tune the extruder at the configured calibration temperature, and clean the nozzle afterward:

    ``` gcode
    MOVE_TO_TRAY
    PID_CALIBRATE HEATER=extruder TARGET=250
    CLEAN_NOZZLE
    SAVE_CONFIG
    ```

=== "Load cells"

    Clear the bed and chamber, then tare, calibrate, and save the load cells:

    ``` gcode
    G28
    LOAD_CELL_CALIBRATE TARE=TRUE SAVE=TRUE
    SAVE_CONFIG
    ```

=== "Z offset and bed mesh"

    This macro performs the load-cell Z home, Z-offset calibration when required, and the bed mesh at the chosen temperature:

    ``` gcode
    BED_MESH_CALIBRATE BED_TEMP=60
    SAVE_CONFIG
    ```

After the printer restarts, run `CHECK_CALIBRATION` again. Each item should be marked `OK` before you start a print.

## Import G-code from USB

Place `.gcode` or `.gco` files in the root of a USB drive and insert it into the printer. COSMOS copies the files into its internal `gcodes` folder so they appear in the print interface. Uppercase `.GCODE` and `.GCO` extensions are also supported.

Files already present in the internal `gcodes` folder with the same name are skipped rather than overwritten.

## Recovery

=== "USB"

    COSMOS offers 2 recovery options on USB devices

    ### Wifi recovery: Set a wifi network via USB

    Create a file called `wpa_supplicant.conf` on the root of your USB drive. Use the following template:

    ```
    ctrl_interface=/var/run/wpa_supplicant
    network={
        ssid="ssid"
        psk="plaintext_password"
    }
    ```

    When the USB is inserted, the `wpa_supplicant.conf` will be deleted. Reboot your machine for the changes to take effect.

    ### Firmware recovery: Install an .swu via USB

    Copy a .swu file as `emergency.swu` on the root of your USB device. When the USB is inserted, it will move the file to `emergency.swu.installed` and install it. If klipper is running, it will show a pop up dialog with the installation progress. Reboot after the intstallation has finished.

=== "UART/FEL"

    UART/FEL is supported under COSMOS. See [the UART/FEL setup docs page](../../hardware/CC1/fel-uart-setup.md) for more information.
