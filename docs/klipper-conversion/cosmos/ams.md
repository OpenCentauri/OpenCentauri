# AMS Support

COSMOS can control multimaterial (AMS) units, giving the Centauri Carbon automated multi-color printing. Most units are controlled through **AFC** (Automated Filament Changer), a Klipper add-on that ships as part of COSMOS, so there is nothing extra to install for AFC itself. One unit — the Anycubic ACE Pro — is instead driven by a separate community add-on that does not use AFC.

!!! warning "Only one filament system at a time"

    AFC and the ACE add-on cannot run together. Enabling both breaks Klipper startup until one is removed. Pick one unit and configure only that one.

## Comparison

The **Type** row uses [Happy Hare's MMU classification](https://github.com/moggieuk/Happy-Hare/wiki/Conceptual-MMU): a *Type A* unit has one shared drive motor and a mechanical selector, while a *Type B* unit gives every lane its own motor and merges them with a combiner instead.

| |[Elegoo CANVAS](#elegoo-canvas)|[Box Turtle](#box-turtle)|[QuattroBox](#quattrobox)|[Anycubic ACE Pro](#anycubic-ace-pro)|
|---|---|---|---|---|
|**Type**|[Type B](https://github.com/moggieuk/Happy-Hare/wiki/Conceptual-MMU)|[Type B](https://github.com/moggieuk/Happy-Hare/wiki/Conceptual-MMU)|[Type B](https://github.com/moggieuk/Happy-Hare/wiki/Conceptual-MMU)|[Type A](https://github.com/moggieuk/Happy-Hare/wiki/Conceptual-MMU)|
|**Lanes**|4|4|4|4 (8 with two units)|
|**Approx. cost**|$55|$270 kit with Discount*|$200-250|$250|
|**Build**|Preassembled|Kit|Kit|Preassembled|
|**Spool management**|Unenclosed|Enclosed|Enclosed|Enclosed|
|**Rewinder**|Spring loaded|Motorized|Spring loaded (filamentalist)|Motorized|
|**Drying Temp**|N/A|N/A|Optional upgrade, 90 °C<sup>†</sup>|55 °C|
|**Software solution in COSMOS**|AFC|AFC|AFC|CosmoACE|
|**Status**|Supported|Documentation pending|Soon™|Community add-on|

/// caption
*At [West3D](https://west3d.com/OPENCENTAURI), 
 use discount code "OPENCENTAURI" on box turtle orders to redeem; also available from [Formbot](https://www.formbot3d.com/?DIST=RUNDHFg%3D) or [Onetwo3d](https://www.onetwo3d.co.uk/product/ldo-boxturtle-v1-0-automated-filament-changer/?wpam_id=13)
 
*<sup>†</sup>90 °C sounds like a bad idea*
///




## Elegoo CANVAS

CANVAS is the only unit supported in the COSMOS tree itself. COSMOS builds and flashes Klipper and Katapult firmware onto the CANVAS mainboard for you, and exposes the four channels as the AFC lanes `CANVAS_1` to `CANVAS_4`, mapped to tools `T0` to `T3`.

CANVAS support is available in the `26.08.0` release and later.

### Setup

1. Install the CANVAS hardware following [Elegoo's manual](https://raw.githubusercontent.com/OpenCentauri/tools/refs/heads/main/pdf/CC1_canvas_manual_EN.pdf). See the [CANVAS hardware pages](../../hardware/CANVAS/CANVAS_components.md) for how the unit is put together.
2. Open `cosmos.conf` in the web interface and enable the unit under the `[extras]` section:

    ```
    [extras]
    # Enable support for the CANVAS unit. Also enables the AFC subsystem.
    elegoo_canvas = True
    ```

3. Reboot the entire printer. A Klipper firmware restart alone does not reload `cosmos.conf` — power cycle or run `REBOOT_MACHINE`.
4. The first boot after enabling CANVAS takes longer than usual while firmware is flashed to the CANVAS board. Do not power off while the screen reports flashing.

COSMOS pulls in the matching Klipper configuration automatically once the setting is enabled. If a CANVAS toolhead is detected while the configuration is *not* loaded klippy will not start.

### Configuration

Lanes are defined in `klipper-readonly/canvas.cfg`. Do not edit that file — firmware updates reset it. To change a value, copy the relevant section into `printer.cfg` and edit it there.

### Known gaps

- RFID spool tag reading is not implemented
- The CANVAS buzzer is not implemented
- Filament eject is disabled
- Runout handling is implemented but lightly tested

## Box Turtle

[Box Turtle](https://github.com/ArmoredTurtle/BoxTurtle) is Armored Turtle's open-source four-lane changer, and is the unit AFC was originally written for. It uses its own [AFC-Lite](https://github.com/xbst/AFC-Lite) controller board, which connects to the Klipper host over USB or CAN.

!!! warning "Not yet documented on COSMOS"

    No one has documented a working Box Turtle setup on COSMOS, and no Box Turtle configuration ships with the firmware. The AFC Klipper extras *are* installed as part of COSMOS, so the software side should be capable.

In principle the work involved is:

1. Build and wire the Box Turtle per [Armored Turtle's manuals](https://armoredturtle.xyz/), connecting the AFC-Lite board to one of the printer's USB ports.
2. Flash Klipper MCU firmware to the AFC-Lite board and identify its serial device.
3. Add the AFC-Lite MCU and the Box Turtle unit and lane definitions to `printer.cfg`, following the [AFC-Klipper-Add-On documentation](https://github.com/ArmoredTurtle/AFC-Klipper-Add-On). COSMOS ships a [fork of that add-on](https://github.com/suchmememanyskill/AFC-Klipper-Add-On), so upstream configuration should broadly apply, but the shipped `canvas.cfg` is the only worked example available on COSMOS.



## QuattroBox

QuattroBox is another four-lane changer supported by AFC. In AFC's configuration it is defined as a unit type inheriting from Box Turtle, so it shares the same lane-based Type B architecture: one motor per lane feeding into a combiner, with no selector. What sets it apart is lighting, it adds addressable LED control for a logo element and for illuminating individual spools, which AFC exposes through `led_logo_index` and `led_spool_index`. It is additionally somewhat lower cost due to the omission of motorized respoolers and an extrusion based frame.

!!! info "Soon™"

    We do not have documentation for QuattroBox on COSMOS yet. Since it is an AFC unit type and the AFC extras ship with COSMOS, the path should resemble [Box Turtle](#box-turtle) above. This section will be filled in properly once someone has run one.

## Anycubic ACE Pro

The ACE Pro is supported through [CosmoACE](https://github.com/shawn-makes-stuff/cosmoace-integration), a community add-on by [shawn-makes-stuff](https://github.com/shawn-makes-stuff). Full installation instructions are available there. It communicates with the ACE system directly over USB serial using the unit's own JSON-RPC protocol, so it **does not use AFC** and CANVAS/AFC support must stay disabled. Two ACE units can be chained for eight colors.


## Adding another unit

AFC supports other changers, such as Night Owl, that nobody has yet tried on COSMOS. If you have one working, or want help getting one running, the `#COSMOS_development` channel on the [OpenCentauri Discord](https://discord.gg/t6Cft3wNJ3) is the place to go, and this page can grow a section for it.
