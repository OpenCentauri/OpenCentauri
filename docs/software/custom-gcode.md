This page contains misc information about some of the custom GCODE commands obtained by reverse engineering

## M8803

Executes the following 3 system commands:

- `cp /board-resource/printer.cfg /mnt/exUDISK/printer.cfg`
- `cp /board-resource/user_printer.cfg /mnt/exUDISK/user_printer.cfg`
- `cp /board-resource/unmodifiable.cfg /mnt/exUDISK/unmodifiable.cfg`

(Note: exUDISK is the USB Stick)

## M8807

Executes the following 2 system commands:

- `cp /mnt/exUDISK/printer.cfg /board-resource/printer.cfg`
- `cp /mnt/exUDISK/user_printer.cfg /board-resource/user_printer.cfg`

(Note: exUDISK is the USB Stick)

!!! warning
    Invalid configs cause the printer to not boot!

### Quick Reference - Hidden Commands

#### Developer Mode
- `M8000` - Enable ROOT mode
- `M8001` - Disable ROOT mode

#### Advanced Calibration
- `M8233 S<offset> P0` - Standard Z-offset profile
- `M8233 S<offset> P1` - Enhancement Z-offset profile
- `CALIBRATION_HX711_SAMPLE` - Calibrate pressure sensors

#### Filament Operations
- `CUT_OFF_FILAMENT ZERO_Z=0/1` - Cut filament with optional Z reset
- `MOVE_TO_EXTRUDE TARGET_TEMP=<temp> MOVE=1 ZERO_Z=0/1` - Move to extrusion position

#### Input Shaper
- `SET_INPUT_SHAPER SHAPER_TYPE_X=<type> SHAPER_FREQ_X=<freq>` - Configure X-axis
- `SET_INPUT_SHAPER SHAPER_TYPE_Y=<type> SHAPER_FREQ_Y=<freq>` - Configure Y-axis

#### LED Control
- `SET_LED_led2 RED=<0-1> GREEN=<0-1> BLUE=<0-1> WHITE=<0-1> TRANSMIT=1` - RGBW control

#### System Reset
- `RESET_EXTRUDER` - Reset extruder MCU
- `RESET_PRINTER_PARAM` - Reset printer parameters

For complete documentation of all hidden commands, see [Hidden Commands](hidden-commands.md).
