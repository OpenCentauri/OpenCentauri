# CC1 CANVAS

CANVAS is a multimaterial upgrade kit for the CC1. The CANVAS module hardware is shared with the CC2; shared documentation is on the [CANVAS General](../CANVAS/CANVAS_components.md) page. This page covers CC1-specific integration differences.

The official manual is available [here](https://raw.githubusercontent.com/OpenCentauri/tools/refs/heads/main/pdf/CC1_canvas_manual_EN.pdf).

<iframe src="https://smith150.autodesk360.com/shares/public/SH90d2dQT28d5b602811c32347388b2b10da?mode=embed" width="800" height="800" allowfullscreen="true" webkitallowfullscreen="true" mozallowfullscreen="true" frameborder="0"></iframe>


![CC1 CANVAS upgrade kit contents](./assets/canvas_kit.jpg){ width="800" }
/// caption
Contents of a CC1 CANVAS upgrade kit, excluding cables. A top cover is not included.
///

## CANVAS Module

![CC1 CANVAS core module](./assets/canvas.jpg){ width="800" }
/// caption
Credit to anna_devminer on the OpenCentauri Discord.
///


The CANVAS core module mounts on the top frame insert of the CC1. It is mechanically identical to the CC2 CANVAS but uses a different mainboard revision and a slightly different [RFID board revision](../CANVAS/CANVAS_components.md#rfid-board).

### CANVAS Mainboard

Metric|Value
---|---
MCU|GD32F303RCT6
Vendor Id|
Product Id|
Device BCD|
Product|
Manufacturer|GigaDevice Semicon Beijing
Stepper driver|4× AT8833 (DRV8833 clone)

Front|Back
---|---
![CC1 CANVAS Mainboard front](./assets/canvas_mainboard.jpg){ width="800" }|![CC1 CANVAS Mainboard back](./assets/canvas_mainboard_back.jpg){ width="800" }

## Spool Holders

The CC2 spool holders are reused. Two adapter brackets are supplied to compensate for the absence of additional tapped holes on the CC1 frame (visible in the kit contents image above). See [CANVAS General — Spool Holders](../CANVAS/CANVAS_components.md#spool-holders) for full details.

## Filament Multiplexer

An identical filament multiplexer to the CC2 is supplied. See [CANVAS General — Filament Multiplexer](../CANVAS/CANVAS_components.md#filament-multiplexer) for full details.

## Revised Toolhead Board

A new toolhead PCB is included to support the filament cutter actuator Hall effect sensor, front cover removal Hall effect sensor, and filament detector. This board is distinct from both the original CC1 toolhead board and the CC2 board. It retains the populated LED MOSFET for toolhead lighting, unlike the CC2. The original CC1 breakout board for the thermistor, heater, and hotend fan is reused.

Front|Back
---|---
![Toolhead board front](./assets/canvas_toolheadboard_front.jpg){ width="800" }|![Toolhead board back](./assets/canvas_toolheadboard_rear.jpg){ width="800" }
/// caption
CC1 CANVAS toolhead board. Credit to anna_devminer on the OpenCentauri Discord.
///

## Toolhead Board Pins

![Mainboard diagram](./assets/cc1canvas_pinmap.svg){ width="1000" }
/// caption
Credit to Baconmilkshake on the OpenCentauri Discord.
///

=== "Toolhead USB (24V)"
    Type: USB-C, carries 24V Vbus

    |pin nr|marking|pin|remarks|
    |--|---|----|---|
    |1| D+ | USB D+ ||
    |2| D- | USB D- ||

=== "Part Cooling Fan"
    Type: 4-Pin connector

    |pin nr|marking|pin|remarks|
    |--|---|----|---|
    |1| Tach | PA0 ||
    |2| PWM | PB5 ||
    |3| 24V | +24V ||
    |4| Gnd | GND ||

=== "LIS2DW12 (SPI1)"
    Onboard accelerometer test point

    |marking|pin|
    |---|---|
    |CS|PA4|

=== "Stepper E"
    Type: 4-Pin connector (motor coil)

    |pin nr|marking|remarks|
    |--|---|---|
    |1| 2B ||
    |2| 1A ||
    |3| 2A ||
    |4| 1B ||

    Driver control test points:

    |marking|pin|
    |---|---|
    |EN|PC15|
    |STEP|PC13|
    |DIR|PC14|
    |TX|PC6|
    |UART|PC7|
    |DIAG|PG14|

=== "Filament Detector Board"
    Type: 4-Pin connector, connects to the [CC2 filament detector board](../CC2/toolhead.md#filament-detector-board)

    |pin nr|marking|pin|remarks|
    |--|---|----|---|
    |1| S5 | PB2 | Top row, tangle detection|
    |2| S4 | PC5 | Top row, [cutter actuation sensor](../CC2/toolhead.md#filament-cutter-actuation-sensor)|
    |3| S3 | ~ | Top row, Model detection- not in cfg|
    |4| S2 | PB0 | Top row, optical filament detect|
    |5| NC || Bottom row|
    |6| S1 | PC4 | Bottom row, toolhead cover detection|
    |7| Gnd | GND | Bottom row|
    |8| 3V3 | +3.3V | Bottom row|

=== "Supplementary Board"
    Type: 8-Pin connector (2x4)

    |pin nr|marking|pin|remarks|
    |--|---|----|---|
    |1| FP | PC8 | Hotend fan PWM|
    |2| LED- | PC9 | LED PWM|
    |3| Temp | PA3 | Top row|
    |4| Heat | PB6 | Top row|
    |5| F+ | +24V | Bottom row, also LED+|
    |6| FS | PA1 | Hotend fan tach, 24v|
    |7| F- | GND | Bottom row|
    |8| F+ | +24V | Bottom row|

=== "Misc LEDs"
    |marking|pin|
    |---|---|
    |led2|PG15|
    |typec_led|PC2|

### Toolhead Board MCU

Metric|Value
---|---
MCU|STM32F402RCT6
USB Spec|v1.0 (full-speed)
Vendor Id|1d50
Product Id|614e
Device BCD|2.00
Product|STM32 Virtual ComPort
Manufacturer|ShenZhenCBD
Stepper driver|TMC2209

## Extruder Upgrade with Filament Detector

A CC2 extruder with filament detector board is included in the upgrade kit. The same board revision is used [as on the CC2](../CC2/toolhead.md#filament-detector-board).

## Revised Front Cover and Hotend Fan Assembly

A new hotend fan duct and fan are provided, with a Hall effect sensor for filament cutter actuation detection. This assembly is identical to the [corresponding CC2 part](../CC2/toolhead.md#filament-cutter-actuation-sensor).

A new toolhead cover is also provided. The filament detector board's forward-facing Hall effect sensor detects cover removal via a magnet in the cover, as [on the CC2](../CC2/toolhead.md#filament-detector-board). The fan supplied is a standard CC1 5020, not the [integrated custom fan used on the CC2](../CC2/toolhead.md#hardware).

![Revised toolhead cover with magnet annotated](./assets/canvas_toolheadcover.jpg){ width="800" }
/// caption
Revised toolhead cover. The magnet location is indicated by the red box. Credit to anna_devminer on the OpenCentauri Discord.
///
