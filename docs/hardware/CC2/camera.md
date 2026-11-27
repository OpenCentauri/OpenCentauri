# CC2 Camera

The CC2 camera is slightly different, but unremarkable from, the various CC1 camera revisions.

Metric|Value
---|---
Resolution|720p(1280x720)
Aspect Ratio|16:9
Max Refresh Rate|30 Hz
Focus|Fixed (glued barrel adjustment)
FOV|90-110°?
Exposure|Auto
PCB Dimensions|30mmx30mm
Connector|4 pin JST-ZH (1.5mm pitch)
Communication Protocol|USB 2.0
Display Name|"Integrated Camera"
Listed Power|500 mA @5Vs
USB Transceiver|
Flash|

## Camera Revisions
At least two revisions of the Centauri Carbon 2 camera PCB exist. The most recent
version (V1.0.30D) was delivered to a customer to replace a faulty V1.0.30B module
and has a different camera SoC, has a wider field of view, and seems to be outputting
3:2 video squashed down to 4:3, as well as the relocation of several passives on
the front of the PCB, other changes have not been documented at this time.

# V1.0.30B
Front|Back
---|---
![Camera back front image](./assets/camerafront.jpg){ width="800" }|![Camera back image](./assets/cameraback.jpg){ width="800" }
/// caption
Credit to chiaia on the OpenCentauri Discord.
///

# V1.0.30D
Front|Back
---|---
![Camera back front image](./assets/camera2front.jpg){ width="800" }|![Camera back image](./assets/camera2back.jpg){ width="800" }
/// caption
Credit to AdequateToodles on the OpenCentauri Discord.
///

## Camera Pins
Pin|Value
---|---
1/GND| GND
2/DP| D+
3/DN| D-
4/5V| +5v