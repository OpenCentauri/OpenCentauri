# COSMOS

![COSMOS logo](./assets/COSMOS_bootlogo.jpg){ width="900" }
/// caption
Credit to notgut on the OpenCentauri Discord for logo design.
///

Run Klipper on the stock hardware (stock mainboard).

Read the [installation instructions](./install.md)

Read the [custom features](./features.md) offered by COSMOS.

Read about [AMS support](./ams.md) for multi-color printing.

!!! warning "**Looking for OpenCentauri patched?**"

    This is the full Klipper/Kalico firmware. For OpenCentauri patched from official Elegoo firmware go [here](../../patched-firmware/index.md), though note that it is winding down and will stop receiving active updates soon.

!!! Danger "**Stop: Before you add any plugins**"

    The stock mainboard is extremely resource limited and there is currently very little headroom to run any plugins, packages, or features not in standard klipper/kalico. Do not attempt to install others unless you _really_ know what you are doing!

## FAQ

??? question "**What is COSMOS?**"

    Open source firmware for the Elegoo Centauri Carbon based on Klipper/Kalico that grants full control over the hardware.


??? question "**What does COSMOS get me that the stock firmware or OC doesn't?**"

    - Ability to directly enter gcode commands in webui console and calibrate from the webui
    - Ability to view the bed mesh in the webui
    - Display input shaper data and compare how mods effect achievable acceleration
    - Ability to level the bed at other temperatures which will give much better ABS first layers since you don't need to worry about bed warp if you level at temp
    - Automatically load the saved `default` bed mesh, or enable adaptive meshing to probe only the area used by each print.
    - Better leveling scripts that increase accuracy
    - See fan RPM in the webUI
    - Directly set exhaust fan speed
    - Support for the CANVAS multimaterial upgrade through AFC, including automatic flashing of the CANVAS board
    - Ability to add an aftermarket AMS such as the Anycubic ACE Pro (see [AMS support](./ams.md))
    - Full control over I/O pins- this should make it possible repurpose model fan - tachometer pin for a toolhead filament detector
    - Ability control and dim the toolhead led for those that have added it, from webui and printer screen
    - Dimming control on the main light
    - Additionally all the major benefits of OC V3.0 (eliminating excessive outgoing traffic, homing changes to increase cable durability, fixed mid-print fan control)

??? question "**Do I need any additional hardware to run COSMOS?**"

    No, COSMOS runs entirely on the stock hardware and no additional boards or equipment is required other than a flash drive 

??? question "**How do I install COSMOS?**"

    Instructions are available [here](./install.md)

??? question "**How long does it take to install COSMOS?**"

    The above process takes <5 minutes to prepare if you already have OC installed, however the first boot after install will take longer than usual because new firmware is being flashed to the toolhead and bed boards. This usually takes 5-10 minutes. Do not power off the printer while the startup screen says that the bed or toolhead is being flashed.

??? question "**How do I uninstall COSMOS?**"

    There is a button in the COSMOS main menu that allows you to switch the printer to OpenCentauri Patched firmware. See the [uninstall instructions](./install.md#uninstall).

??? question "**Does installing COSMOS break my printers warranty?**"

    While we cannot say with certainty what elegoo's position is we have not heard of any reports of customers being denied warranty services and part replacements after installing 3rd party firmware such as OpenCentauri.

??? question "**What do I do if I find a bug?**"

    [Open an issue on GitHub](https://github.com/OpenCentauri/cosmos/issues) and provide a brief description of what happened and the steps to reproduce it. Alternatively you can also drop by the #COSMOS_development channel on the [Opencentauri Discord server](https://discord.gg/t6Cft3wNJ3) to let us know.

??? question "**Will COSMOS be available for the Centauri Carbon 2?**"

    Maybe, but developer efforts are focused on the CC1 for the time being

??? question "**Will COSMOS work with the CANVAS upgrade for the Centauri Carbon 1?**"

    Yes. COSMOS drives CANVAS through AFC (see the next question), and builds and flashes Klipper firmware onto the CANVAS board for you. The four channels appear as the AFC lanes `CANVAS_1` to `CANVAS_4`, mapped to tools `T0` to `T3`, so slicers and macros address them like any other toolchanger. Enable CANVAS in `cosmos.conf` and COSMOS pulls in the matching Klipper configuration automatically. If a CANVAS toolhead is detected while the configuration is not loaded, COSMOS shuts down rather than driving hardware it is not configured for.

    Setup steps are on the [AMS support](./ams.md) page. CANVAS support is available in the `26.08.0` release and later. It is newer and less tested than the rest of COSMOS, and a few things are not implemented yet — most notably RFID spool tag reading and the CANVAS buzzer, and filament eject is disabled. Do not edit `klipper-readonly/canvas.cfg` directly; override the sections you need in `printer.cfg` instead, as updates reset it.

??? question "**What is AFC, and why does COSMOS use it for CANVAS?**"

    AFC (Automated Filament Changer) is an existing Klipper add-on for driving multimaterial units, so it already handles lanes, tool mapping, loading and unloading, runout, and LED status. COSMOS ships a [fork of AFC-Klipper-Add-On](https://github.com/suchmememanyskill/AFC-Klipper-Add-On) with the changes needed for CANVAS hardware rather than writing a CANVAS-only implementation from scratch. It is installed as part of COSMOS, so there is nothing extra to add — and, as with any other package, you should not install a different copy of it yourself.

??? question "**Is COSMOS related to the OpenCentauri board?**"

    No, the OpenCentauri board is another ongoing project to create a much more powerful drop in mainboard replacement for the Centauri Carbon. 

??? question "**Can I add a raspberrypi to offload work from the stock mainboard?**"

    Not currently. The stock hardware is not very powerful and has limited memory which is why adding a pi or other SBC may interest some, however the primary focus of the project right now is to create stable firmware for the stock board.

??? question "**How can I support COSMOS development?**"

    You can make a one time or monthly donation to [support the OpenCentauri project on our KoFi](https://ko-fi.com/opencentauri)



![COSMOS logo](./assets/AMS_test.jpg){ width="500" }
/// caption
Credit to [shawn-makes-stuff](https://github.com/shawn-makes-stuff) for Demo print on CC1 and anycubic ACE multicolor print with COSMOS.
///
