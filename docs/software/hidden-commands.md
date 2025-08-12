# Hidden Commands and Features

This page documents undocumented commands and features discovered through firmware analysis.

## Root Mode

### Enable Developer Mode

The firmware includes a hidden root/developer mode:

| Command | Description |
|---------|-------------|
| `M8000` | Activate ROOT mode |
| `M8001` | Deactivate ROOT mode (UNROOT) |

!!! info "Potential Features"
    Root mode may unlock additional debug features, logging, or configuration options. Further investigation needed.

## Extended M-codes (M8000-M8826)

### Z-offset Profiles (M8233)

Advanced Z-offset calibration with two quality modes:

```gcode
M8233 S<offset> P0  # Standard mode
M8233 S<offset> P1  # Enhancement mode
```

Configuration parameters in `unmodifiable.cfg`:
```ini
[strain_gauge]
standard_fix_z_offset : 0.15
enhancement_fix_z_offset : 0.17
```

### Configuration Management

| Command | Function |
|---------|----------|
| `M8803` | Export configs (Board → USB) |
| `M8807` | Import configs (USB → Board) |

File operations:
```bash
# M8803 executes:
cp /board-resource/printer.cfg /mnt/exUDISK/printer.cfg
cp /board-resource/user_printer.cfg /mnt/exUDISK/user_printer.cfg
cp /board-resource/unmodifiable.cfg /mnt/exUDISK/unmodifiable.cfg

# M8807 executes:
cp /mnt/exUDISK/printer.cfg /board-resource/printer.cfg
cp /mnt/exUDISK/user_printer.cfg /board-resource/user_printer.cfg
```

### Other M-codes

Most M-codes from M8000-M8826 appear reserved but unused. Known functions:

- `M8000-M8001` - Root mode control
- `M8233` - Z-offset profiles
- `M8803` - Export configuration
- `M8807` - Import configuration
- `M8823` - Force strain gauge update (if `update=0` in config)

## Hidden G-code Commands

### Filament Operations

#### Cut Filament

Physical filament cutting mechanism:

```gcode
CUT_OFF_FILAMENT ZERO_Z=0  # Cut without Z axis reset
CUT_OFF_FILAMENT ZERO_Z=1  # Cut with Z axis reset to zero
```

#### Advanced Extrusion

Extrusion with fan control:

```gcode
EXTRUDE_FILAMENT E=100 F=300 FAN_ON=1   # Extrude with fan on
EXTRUDE_FILAMENT E=-50 F=300 FAN_ON=0   # Retract with fan off
```

#### Move to Extrude Position

```gcode
MOVE_TO_EXTRUDE TARGET_TEMP=200 MOVE=1 ZERO_Z=0
MOVE_TO_EXTRUDE TARGET_TEMP=200 MOVE=1 ZERO_Z=1
```

### Bed Mesh Operations

#### Mesh Applications Control

Enable/disable mesh compensation:

```gcode
BED_MESH_APPLICATIONS ENABLE=0  # Disable mesh
BED_MESH_APPLICATIONS ENABLE=1  # Enable mesh
```

#### Fast Calibration Modes

```gcode
G29 BED_MESH_CALIBRATE fast    # Quick calibration
G29 BED_MESH_CALIBRATE normal  # Standard calibration
```

### Strain Gauge Calibration

Special calibration for HX711 sensors:

```gcode
CALIBRATION_HX711_SAMPLE  # Run HX711 calibration routine
```

### Diagnostic Commands

```gcode
CHECK_ITEM_VIBRATION      # Vibration test
FLOW_CALIBRATION         # Flow rate calibration
Z_AXIS_OFF_LIMIT_ACTION  # Z-axis limit behavior
```

## SET_ Commands

### Input Shaper Configuration

Configure resonance compensation:

```gcode
# X-axis shaper
SET_INPUT_SHAPER SHAPER_TYPE_X=mzv SHAPER_FREQ_X=40.0
SET_INPUT_SHAPER SHAPER_TYPE_X=ei SHAPER_FREQ_X=35.0
SET_INPUT_SHAPER SHAPER_TYPE_X=2hump_ei SHAPER_FREQ_X=45.0
SET_INPUT_SHAPER SHAPER_TYPE_X=3hump_ei SHAPER_FREQ_X=50.0

# Y-axis shaper
SET_INPUT_SHAPER SHAPER_TYPE_Y=mzv SHAPER_FREQ_Y=38.0
SET_INPUT_SHAPER SHAPER_TYPE_Y=zv SHAPER_FREQ_Y=42.0

# Both axes
SET_INPUT_SHAPER SHAPER_TYPE=mzv SHAPER_FREQ_X=40 SHAPER_FREQ_Y=38
```

### LED Control

#### RGBW LED Control

```gcode
# Turn off all LEDs
SET_LED_led2 RED=0 GREEN=0 BLUE=0 WHITE=0 TRANSMIT=1

# Set specific color
SET_LED_led2 RED=1 GREEN=0 BLUE=0 WHITE=0 TRANSMIT=1  # Red
SET_LED_led2 RED=0 GREEN=1 BLUE=0 WHITE=0 TRANSMIT=1  # Green
SET_LED_led2 RED=0 GREEN=0 BLUE=1 WHITE=0 TRANSMIT=1  # Blue
SET_LED_led2 RED=0 GREEN=0 BLUE=0 WHITE=1 TRANSMIT=1  # White

# Full brightness
SET_LED_led2 RED=1 GREEN=1 BLUE=1 WHITE=1 TRANSMIT=1
```

Available LED names:
- `led1` - Main LED
- `led2` - Secondary LED (RGBW)
- `typec_led` - USB-C port LED
- `box_led` - Case LED
- `neopixel` - NeoPixel support

### Fan Control

Control fans by index:

```gcode
SET_FAN_SPEED I=0 S=255  # Fan 0 at 100%
SET_FAN_SPEED I=1 S=128  # Fan 1 at 50%
SET_FAN_SPEED I=2 S=0    # Fan 2 off
SET_FAN_SPEED I=3 S=64   # Fan 3 at 25%
```

### Temperature Controls

#### Minimum Extrusion Temperature

```gcode
SET_MIN_EXTRUDE_TEMP S0      # Disable protection (dangerous!)
SET_MIN_EXTRUDE_TEMP RESET   # Reset to default
```

### Z-offset with Movement

```gcode
# Set offset without moving
SET_GCODE_OFFSET Z=0.1 MOVE=0 MOVE_SPEED=5.0

# Set offset and move to new position
SET_GCODE_OFFSET Z=0.1 MOVE=1 MOVE_SPEED=5.0
```

### Print Statistics

Update print progress information:

```gcode
# Layer information
SET_PRINT_STATS_INFO CURRENT_LAYER=5
SET_PRINT_STATS_INFO CURRENT_LAYER=5 TOTAL_LAYERS=100

# Time tracking
SET_PRINT_STATS_INFO LAST_PRINT_TIME=3600
SET_PRINT_STATS_INFO ALREAD_PRINT_TIME=1800
```

### Silent Mode

Enable/disable silent operation:

```gcode
SET_TMC_CURRENT_<stepper> CURRENT=<value>  # Adjust motor current
```

## RESET Commands

System reset commands:

| Command | Function |
|---------|----------|
| `RESET_EXTRUDER` | Reset extruder MCU |
| `RESET_FILAMENT_WIDTH_SENSOR` | Reset width sensor |
| `RESET_PRINTER_PARAM` | Reset printer parameters |
| `SDCARD_RESET_FILE` | Reset SD card file pointer |

## Hidden Print States

The firmware tracks additional print states not exposed in the API:

- `PRINT_STATS_STATE_PRINT_START`
- `PRINT_STATS_STATE_PAUSEING` (sic)
- `PRINT_STATS_STATE_RESUMING`
- `PRINT_STATS_STATE_CANCELLING`

## Break Save Commands

Power loss recovery system:

```gcode
BREAK_SAVE_STATUS  # Check recovery status
```

Files:
- Save location: `/board-resource/break_save0.gcode`
- Calibration: `FirstLayer.gcode`

## Display Control

### Backlight Control

Direct display backlight control via kernel interface:

```bash
# Turn off
echo 0 > /sys/kernel/debug/dispdbg/param

# Set brightness (0-255)
echo 128 > /sys/kernel/debug/dispdbg/param

# Maximum brightness
echo 255 > /sys/kernel/debug/dispdbg/param
```

## Thermistor Types

Additional thermistor support:

```ini
sensor_type: EPCOS 100K B57560G104F
# or
sensor_type: ATC Semitec 104GT-2
```

## Idle Timeout Control

```gcode
SET_IDLE_TIMEOUT TIMEOUT=600  # Set timeout to 10 minutes
UPDATE_IDLE_TIMER ACTIVE_TIME=300  # Update active timer
```

## Explorer Operations

File system operations:

```
EXPLORER_OPERATION_COPY_DONE
EXPLORER_OPERATION_COPY_FAIL
EXPLORER_OPERATION_COPYING
EXPLORER_OPERATION_VERIFYING
```

## Testing and Calibration

### Position Commands

```gcode
GET_POSITION           # Get current position
SET_POSITION           # Set position
SET_KINEMATIC_POSITION # Set kinematic position
```

### Retraction Settings

```gcode
GET_RETRACTION  # Get retraction settings
SET_RETRACTION  # Set retraction parameters
```

## Usage Examples

### Complete Input Shaper Setup

```gcode
# 1. Run vibration test
CHECK_ITEM_VIBRATION

# 2. Configure shaper based on results
SET_INPUT_SHAPER SHAPER_TYPE_X=mzv SHAPER_FREQ_X=40.0
SET_INPUT_SHAPER SHAPER_TYPE_Y=ei SHAPER_FREQ_Y=35.0

# 3. Test with print
```

### Advanced Filament Change

```gcode
# 1. Move to safe position
MOVE_TO_EXTRUDE TARGET_TEMP=200 MOVE=1 ZERO_Z=0

# 2. Retract filament
EXTRUDE_FILAMENT E=-100 F=300 FAN_ON=1

# 3. Cut filament (if cutter installed)
CUT_OFF_FILAMENT ZERO_Z=0

# 4. Load new filament
EXTRUDE_FILAMENT E=100 F=300 FAN_ON=1
```
