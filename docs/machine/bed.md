Metric|Value
---|---
MCU|STM32F402RCT6
Vendor Id|1d50
Product Id|614e
Device BCD|2.00
Product|STM32 Virtual ComPort
Manufacturer|ShenZhenCBD

![Bed overview](../assets/centauri-bed-overview.jpg){ width="600" }
/// caption
Credit to rabirx on the OpenCentauri Discord.
///

Front|Back
---|---
![Bed board image](../assets/centauri-bed.jpg){ width="800" }|![Bed board back image](../assets/centauri-bed-back.jpg){ width="800" }
Credit to rabirx on the OpenCentauri Discord.|Credit to rabirx on the OpenCentauri Discord.

The bed is its own Klipper MCU with 4 pressure sensors that get read out by HX711 ADC chips. The heating is not controlled by the MCU, but via a seperate AC board.

The board connects with serial (not over USB) to the mainboard.
