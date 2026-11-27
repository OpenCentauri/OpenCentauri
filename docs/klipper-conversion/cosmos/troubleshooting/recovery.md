# Configuration and recovery


## I got a new toolhead or bed board and now COSMOS won't boot

A replacement board from Elegoo arrives running stock firmware, which COSMOS cannot talk to. COSMOS decides whether an MCU needs flashing by comparing a small version stamp in `/etc` against the firmware version it ships, so the fix is to delete that stamp and let the flashing service run again.

Stop Klipper first, or it will hold the serial port open:

``` sh
service klipper stop
rm /etc/toolhead.ver
service klipper-firmware-toolhead restart
service klipper start
```

Substitute the board you actually replaced:

Board|Version stamp|Service
---|---|---
Toolhead|`/etc/toolhead.ver`|`klipper-firmware-toolhead`
Bed|`/etc/bed.ver`|`klipper-firmware-bed`
