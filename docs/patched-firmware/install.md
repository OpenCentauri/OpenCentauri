# Install Patched Firmware

!!! warning "**Deprecation notice**"
    Please note as of **<ins>November 25th 2026</ins>**, the "patched" firmware has been deprecated.
    Future development efforts will be focussed on COSMOS and bug reports for patched will
    not be prioritized. Newly discovered major issues may still receive fixes but we
    recommend migrating to COSMOS if you're having any problems.

    If your printer is working and you aren't interested in updating, it is safe to
    remain on patched firmware, just be aware new features and fixes likely won't be
    backported to patched.

    [COSMOS installation instructions](../klipper-conversion/cosmos/cosmos.md)

=== "Online"
    !!! warning "Online requirement"

        Your Centauri Carbon needs to be connected to the internet in order to download OpenCentauri firmware. Choose the `Local/Offline` installation method if your Centauri Carbon is not connected to the internet.

    1. [Download](https://github.com/OpenCentauri/oc-installer/releases/latest/download/oc-installer.zip) the latest OpenCentauri installer release
    1. Insert a fat32-formatted thumbdrive into your pc
    1. Extract the `install_opencentauri` folder from `oc-installer.zip` onto the root of the thumbdrive
    1. Eject your thumbdrive from your pc and insert it into your Centauri Carbon
    1. On your Centauri Carbon, navigate to the files tab, then tap the `Usb Drive` menu, then tap the `install_opencentauri` folder
    1. Touch hold the `IMPORT_ME_DO_NOT_PRINT` file, then tap `Import`
    1. In the installer, select `Install OpenCentauri` > `Install Patched (Online)`
    1. Select `Reboot` when the installation has finished

=== "Local/Offline"

    1. [Download](https://github.com/OpenCentauri/oc-installer/releases/latest/download/oc-installer.zip) the latest OpenCentauri installer release
    1. Insert a fat32-formatted thumbdrive into your pc
    1. Extract the `install_opencentauri` folder from `oc-installer.zip` onto the root of the thumbdrive
    1. [Download update.swu](https://github.com/OpenCentauri/cc-fw-tools/releases/latest/download/update.swu) and place it inside the `install_opencentauri` folder on your thumbdrive
    1. Eject your thumbdrive from your pc and insert it into your Centauri Carbon
    1. On your Centauri Carbon, navigate to the files tab, then tap the `Usb Drive` menu, then tap the `install_opencentauri` folder
    1. Touch hold the `IMPORT_ME_DO_NOT_PRINT` file, then tap `Import`
    1. In the installer, select `Install OpenCentauri` > `Install OpenCentauri From USB`
    1. Select `Reboot` when the installation has finished

You should now be greeted by the OpenCentauri splash screen :tada:

This firmware works well with [modified machine start and end gcode in OrcaSlicer](./modified_start_end_machine_gcode.md), which centers the purge line and turns on the chamber light at print start. Installing this is optional but recommended.

## Update

You can either accept OTA updates directly on the device or repeat the steps in the [Installation section](#install-patched-firmware).

## Uninstall

Follow the steps again in the [Installation section](#install-patched-firmware). In the installer, select `Install Official` > `Install 1.4.46 (Online)`.
