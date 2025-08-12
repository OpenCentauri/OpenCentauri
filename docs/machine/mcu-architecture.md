# MCU Architecture

The Centauri Carbon uses a 3-MCU architecture with DSP shared memory communication.

## Overview

Unlike traditional 3D printers, the Centauri Carbon distributes control across three separate MCUs, each with specific responsibilities:

```
┌──────────────────┐
│   AllWinner SoC  │
│  ┌─────────────┐ │     DSP Remote Processor     ┌──────────────┐
│  │  ARM Core   │ ├──────── dsp_rproc@0 ───────► │     DSP      │
│  │  (app/app)  │ │                              │(Memory       │
│  └─────────────┘ │     Shared Memory (ION)      │Communication)│
│         │        │ ◄────── /dev/kbuf-mgr-0 ────►└──────────────┘
│         │        │
│  ┌──────┴──────┐ │     RPMSG msgbox_demo
│  │   Klipper   │ │ ◄─────────────────────────────►
│  │   (C++)     │ │
│  └─────────────┘ │
└──────────┬───────┘
           │
    ┌──────┴─────┬────────────────┐
    │            │                │
/dev/ttyS4    /dev/ttyACM0    mem_interface
@115200        @1000000           (DSP)
    │            │                │
┌───┴───┐    ┌───┴────┐      ┌────┴─────┐
│  Bed  │    │ Hotend │      │ Main MCU │
│  MCU  │    │  MCU   │      │ (Klipper)│
└───┬───┘    └────────┘      └──────────┘
    │
┌───┴────────────────┐
│ 4x HX711 sensors   │
│ with Kalman filter │
└────────────────────┘
```

## Main MCU

The main MCU runs on the AllWinner SoC and uses a unique shared memory interface for communication.

### Key Characteristics

- **Interface**: `mem_interface` (DSP shared memory)
- **No serial port** - communicates entirely through shared memory
- **Memory Manager**: `/dev/kbuf-mgr-0`
- **Transport**: RPMSG with endpoint `msgbox_demo`

### Klipper Configuration

```ini
[mcu]
mem_interface : mem_interface_DSP
max_pending_blocks : 12
```

### DSP Communication

The DSP handles memory communication using ION (Android memory allocator):

1. **Buffer Manager**: `/dev/kbuf-mgr-0`
2. **Mapped Buffers**: `/dev/kbuf-map-%d-%s` format
3. **Debug Interface**: `/dev/dsp_debug`

!!! note "Unique Architecture"
    This shared memory approach is unique among consumer 3D printers and provides deterministic real-time communication.

## Bed MCU (strain_gauge_mcu)

The bed MCU manages the innovative pressure-based bed leveling system.

### Specifications

| Parameter | Value |
|-----------|-------|
| MCU Model | STM32F402RCT6 |
| Serial Port | `/dev/ttyS4` |
| Baud Rate | 115200 |
| Power Pin | PG9 |
| Sensors | 4x HX711 |

### Klipper Configuration

```ini
[mcu strain_gauge_mcu]
serial: /dev/ttyS4
baud: 115200
restart_method: mcu_reset
power_pin: PG9
```

### HX711 Pressure Sensors

The bed uses 4 HX711 load cells with advanced signal processing:

- **Kalman filtering** for noise reduction
- **Temperature compensation**
- **Real-time force feedback**

!!! info "Strain Gauge Protocol"
    ```
    config_hx711s oid=%c hx711_count=%c channels=%u rest_ticks=%u
                  kalman_q=%u kalman_r=%u max_th=%u min_th=%u k=%u
    ```

## Hotend MCU

Controls the extruder and monitors vibrations for input shaping.

### Specifications

| Parameter | Value |
|-----------|-------|
| Serial Port | `/dev/ttyACM0` |
| Baud Rate | 1000000 |
| Power Pin | PE12 |
| Accelerometer | LIS2DW12 |

### Klipper Configuration

```ini
[mcu stm32]
serial: /dev/ttyACM0
baud: 1000000
restart_method: mcu_reset
power_pin: PE12
```

### Accelerometer

Despite configuration mentioning ADXL345, the actual chip is a **LIS2DW12**:

```ini
[adxl345 X]
adxl_type: lis2dw12
axes_map: x,z,-y
```

## Communication Protocols

### Supported Interfaces

| Interface | Usage | MCUs |
|-----------|-------|------|
| `mem_interface` | Shared memory via DSP | Main MCU |
| `serial` | Traditional UART | Bed & Hotend MCUs |
| `canbus_uuid` | CAN bus (unused) | None |
| `canbus_interface` | CAN bus (unused) | None |

### Dead Code

The firmware contains references to unused interfaces:
- `/dev/ttyS2` - Present in code but not connected
- `init_dsp_uart` - Function exists but never called
- `SetupSerial ttyS1` - Unused serial setup

## Power Control

Each auxiliary MCU has GPIO-controlled power:

```python
# Power cycle example
GPIO.output(PG9, GPIO.LOW)   # Power off bed MCU
time.sleep(0.5)
GPIO.output(PG9, GPIO.HIGH)  # Power on bed MCU
```

!!! warning "Power Sequencing"
    Always ensure proper delay between power cycles to avoid MCU lockup.

## Debugging

### Crash Reporting

The system captures MCU crashes with full ARM register dumps:

```
extruder_bootup_info oid=%c crash_flag=%c rest_cause=%c
                     R0=%u R1=%u R2=%u R3=%u R12=%u
                     LR=%u PC=%u xPSR=%u
```

### Log Locations

- `/board-resource/lastlog` - Last log before crash
- `/user-resource/coredump-*.gz` - Compressed crash dumps
- `/dev/dsp_debug` - DSP debug interface
