# Patched Firmware

!!! warning "**Deprecation notice**"
    Please note as of **<ins>November 25th 2026</ins>**, the "patched" firmware has been deprecated.
    Future development efforts will be focussed on COSMOS and bug reports for patched will
    not be prioritized. Newly discovered major issues may still receive fixes but we
    recommend migrating to COSMOS if you're having any problems.

    If your printer is working and you aren't interested in updating, it is safe to
    remain on patched firmware, just be aware new features and fixes likely won't be
    backported to patched.

    [COSMOS installation instructions](../klipper-conversion/cosmos/cosmos.md)

Install a patched version of Elegoo's official CC1 firmware with bug fixes, quality-of-life improvements, and developer features — no extra hardware required.

Read the [installation instructions](./install.md)

Read the [full features list](./features.md)

If you find bugs or want to suggest new features, please use the [cc-fw-tools](https://github.com/OpenCentauri/cc-fw-tools){target="_blank"} repository. Not everything is possible within the limits of patching existing firmware, especially large features or full overhauls.

If you'd like to support our work, you can do so on [Ko-Fi](https://ko-fi.com/opencentauri) :heart:! If spending money isn't your thing, we also have a [Makerworld page](https://makerworld.com/en/models/1924078-opencentauri-logo#profileId-2064746) where you can throw some boosts towards!

!!! warning "**Looking for Full Klipper/COSMOS?**"

    This is regular OpenCentauri patched from official Elegoo firmware. For the full Klipper/Kalico firmware with COSMOS go [here](../klipper-conversion/cosmos/cosmos.md)

!!! warning "Winding down"
    The patched firmware will stop receiving active updates soon. It remains installable and usable, but new features are going into [COSMOS](../klipper-conversion/cosmos/cosmos.md) instead, and users who want continued feature development are encouraged to move over. If you need support, feel free to [join the Discord](https://discord.gg/t6Cft3wNJ3).

## FAQ

??? question "**What is OpenCentauri patched firmware?**"

    A modified version of Elegoo's official CC1 firmware (based on 1.4.46 as of v0.4.0) that adds bug fixes, quality-of-life improvements, and developer features — without replacing the underlying software stack.

??? question "**What does the patched firmware get me that stock firmware doesn't?**"

    - SSH access (user: `root`, password: `OpenCentauri`)
    - Developer tools including a package manager and the ability to prevent Klipper from booting
    - Configurable boot logo
    - Exhaust fan no longer turns on automatically during prints
    - Homing position changed to front right to prevent USB toolhead cable damage
    - WebUI accepts modifications during a print (backported fix from 1.1.42)
    - WebUI store button removed and logo replaced with OpenCentauri branding
    - Z offset can be adjusted while the printer is idle
    - Files can be uploaded while printing
    - Filament usage reported via the API
    - Outgoing connectivity checks blocked
    - OpenCentauri OTA updates instead of official Elegoo updates
    - USB Ethernet adapter support
    - New gcode commands: `M8212`/`M8213` for chamber light and `TEMPERATURE_WAIT SENSOR=box`

    See the [Features page](./features.md) for the full annotated list.

??? question "**Why was the homing position changed to the front right?**"

    The stock front-left homing position routes the USB toolhead cable in a way that causes wear and eventual failure over time. Homing to the front right keeps the cable in a less stressful position and significantly increases its lifespan.

??? question "**Do I need any extra hardware?**"

    No. The patched firmware runs entirely on stock hardware. Only a FAT32-formatted USB drive is needed for installation.

??? question "**Does it work with the CANVAS multimaterial upgrade?**"

    Yes, from v0.4.0 onwards. That release rebased the OpenCentauri patches onto Elegoo 1.4.46, the official firmware that introduced CANVAS support, so multi-color printing behaves the same as it does on stock. v0.3.0 and earlier were based on 1.1.40 and are only compatible with non-CANVAS CC1s on the older mainboard — if you have CANVAS or a printer with the newer Wi-Fi chip, use patched firmware v0.4.0 or newer (or stock 1.4.42 or newer).

    If you would rather drive CANVAS from a full Klipper stack, [COSMOS](../klipper-conversion/cosmos/cosmos.md) supports it through AFC.

??? question "**Is this the same as COSMOS?**"

    No. Patched firmware is still based on Elegoo's official software stack with targeted patches applied on top. [COSMOS](../klipper-conversion/cosmos/cosmos.md) is a full replacement firmware based on Klipper/Kalico that gives complete control over the hardware, and is where active development is now focused.

??? question "**Is it compatible with the Centauri Carbon 2?**"

    Not currently. The patched firmware is based on Elegoo's CC1 firmware. CC2 support is not available at this time.

??? question "**How do I install it?**"

    Instructions are on the [Install page](./install.md).

??? question "**How do I update it?**"

    You can accept OTA updates directly on the printer screen, or repeat the installation steps. See the [Install page](./install.md#update).

??? question "**How do I uninstall it?**"

    Follow the installation steps again and select `Install Official` > `Install 1.4.46 (Online)`. See the [Install page](./install.md#uninstall).

??? question "**Does installing the patched firmware break my warranty?**"

    While we cannot say with certainty what Elegoo's position is, we have not heard of any reports of customers being denied warranty service or part replacements after installing third-party firmware such as OpenCentauri.

??? question "**What do I do if I find a bug?**"

    Open an issue on the [cc-fw-tools GitHub repository](https://github.com/OpenCentauri/cc-fw-tools) with a description of the issue and steps to reproduce it. You can also drop by the [OpenCentauri Discord](https://discord.gg/t6Cft3wNJ3) to report it.

??? question "**How can I support the project?**"

    You can make a one-time or monthly donation on [Ko-Fi](https://ko-fi.com/opencentauri), or boost our models on [Makerworld](https://makerworld.com/en/models/1924078-opencentauri-logo#profileId-2064746)!
