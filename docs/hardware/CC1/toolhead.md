# CC1 Toolhead

Front|Back
---|---
![Toolhead board image](./assets/centauri-hotend.jpg){ width="800" }|![Toolhead board back image](./assets/centauri-hotend-back.jpg){ width="800" }
Credit to thijskunst on the OpenCentauri Discord.|Credit to thijskunst on the OpenCentauri Discord.

The toolhead board is connected over a USB-C cable. This USB-C interface carries 24v. Communication is done via a serial-over-usb interface. The MCU provides a virtual com port when booted. The toolhead board runs Klipper MCU firmware, specifically [v0.9.1-616-g28f60f7e-dirty-20220408_035823-fluiddpi](https://github.com/Klipper3d/klipper/commit/28f60f7ef69847f1514371d1c6788c3c0df98533)


!!! example
    The board boots from a simple 5v USB connection.

!!! warning
    You can make the toolhead board boot into DFU mode by bridging the boot and 3.3v pins on the back during power-on. The board is in Read Out Protection mode. See [Embedded Firmware](../../software/embedded-firmware.md) for flashing instructions and important caveats.

## Supplementary board

![Toolhead supplementary board image](./assets/centauri-hotend-supplementary.jpg){ width="600" }
/// caption
Credit to rabirx on the OpenCentauri Discord.
///

The Toolhead board has an 2x4 pin port at the bottom of the board. This connector connects to a separate pcb, that breaks out the necessary connectors for the hotend (Temperature sensor, heater, hotend fan).

## Toolhead Board Pins

![Mainboard diagram](./assets/cc1_pinmap.svg){ width="1000" }
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
    |TX|PE9|
    |UART|PE6|
    |DIAG|PG4|

=== "Supplementary Board"
    Type: 8-Pin connector (2x4), connects to [supplementary board](#supplementary-board)

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

=== "LED Test Points"
    |marking|pin|
    |---|---|
    |led2|PG15|
    |typec_led|PC2|

## MCU

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

## Hardware

Metric|Value
---|---
Motor type|10T NEMA14 (round, 20.5mm long)
Motor P/N|BJY36D12-04V28
Motor MFG|SHENZHEN  KELI MOTOR  LTD
Extruder gear ratio|52:10
Extruder hobbed gear diameter|10mm nominal
Extruder hobbed gear material|SDK11 tool steel
Heater type|Ceramic plate-type PTC heater
Heater resistance|~9.6Ω
Heater power|60W
Thermistor Type| Glass bead NTC-100k
Thermistor Beta| 4300
Fan manufacturer| Shenzhen Hua Xinrong Plastic Electronics Co., Ltd
Part cooling fan type|5020 Wide mouth radial fan, 4 pin (tach+5V PWM)
Part cooling fan P/N|EFC-05D24D
Part cooling fan power|0.50A @ 24V
Part cooling fan speed|12,000 RPM
Hotend fan type|3010 axial fan, 3 pin (tach)
Hotend fan P/N|BFC-03A24L
Hotend fan power|0.10A @ 24V
Hotend fan speed|12,000 RPM

The Part cooling and hotend fans use variable frequency tachometer outputs with a set 50% duty cycle. Fan speed is 60/2*[Tach Hz].

![Part cooling fan speed vs duty cycle](./assets/partcooling_fanspeed.jpg){ width="400" }
/// caption
Plot of part cooling fan speed as measured by oscilloscope vs input PWM duty cycle.
Credit to baconmilkshake on the OpenCentauri Discord.
///
