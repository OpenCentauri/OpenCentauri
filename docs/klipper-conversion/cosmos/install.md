# Install COSMOS

!!! Note "Actively developed"
    COSMOS is stable enough for everyday printing, but it is still actively developed. Functionality can change between releases, and newer features get less mileage than the rest of the firmware.

!!! Danger "**Stop: Before you add any plugins**"

    The stock mainboard is extremely resource limited and there is currently very little overhead to run any plugins, packages, or features not in standard klipper/kalico. Do not attempt to install others unless you _really_ know what you are doing!

## Installation

!!! Note
    Because COSMOS is actively developed, it is important to read [the changelogs](https://github.com/OpenCentauri/cosmos/releases) on GitHub before installing a new version. They may contain additional instructions for migration from an old version.

=== "Centauri Carbon 1"

    Install using the OpenCentauri installer:

    === "Online"

        !!! warning "Online requirement"

            Your Centauri Carbon must be connected to the internet so the installer can download COSMOS. Choose the `Local/Offline` method if the printer is not connected to the internet.

        1. [Download](https://github.com/OpenCentauri/oc-installer/releases/latest/download/oc-installer.zip) the latest OpenCentauri installer release.
        1. Insert a FAT32-formatted thumbdrive into your PC.
        1. Extract the `install_opencentauri` folder from `oc-installer.zip` onto the root of the thumbdrive.
        1. Eject the thumbdrive from your PC and insert it into your Centauri Carbon.
        1. On your Centauri Carbon, navigate to the files tab, tap `Usb Drive`, then open the `install_opencentauri` folder.
        1. Touch and hold the `IMPORT_ME_DO_NOT_PRINT` file, then tap `Import`.
        1. In the installer, select `Install OpenCentauri` > `Install COSMOS (Online)`.
        1. Select `Reboot` when the installation has finished.
        1. Remove the thumbdrive after COSMOS has booted.

    === "Local/Offline"

        1. [Download](https://github.com/OpenCentauri/oc-installer/releases/latest/download/oc-installer.zip) the latest OpenCentauri installer release.
        1. Insert a FAT32-formatted thumbdrive into your PC.
        1. Extract the `install_opencentauri` folder from `oc-installer.zip` onto the root of the thumbdrive.
        1. [Download the latest COSMOS `update.swu`](https://github.com/OpenCentauri/cosmos/releases/latest/download/update.swu) and place it inside the `install_opencentauri` folder on your thumbdrive.
        1. Eject the thumbdrive from your PC and insert it into your Centauri Carbon.
        1. On your Centauri Carbon, navigate to the files tab, tap `Usb Drive`, then open the `install_opencentauri` folder.
        1. Touch and hold the `IMPORT_ME_DO_NOT_PRINT` file, then tap `Import`.
        1. In the installer, select `Install OpenCentauri` > `Install OpenCentauri From USB`.
        1. Select `Reboot` when the installation has finished.
        1. Remove the thumbdrive after COSMOS has booted.

=== "Centauri Carbon 2 / Centauri 2"

    1. Download the OTA upgrade file for your model from the [latest COSMOS release](https://github.com/OpenCentauri/cosmos/releases/latest):
        - **Centauri Carbon 2 / Centauri Carbon 2 Combo:** the `cc2_eeb001_*.zip.sig` file.
        - **Centauri 2 / Centauri 2 Combo:** the `c2_eeb001_*.zip.sig` file.
    1. Place the `.zip.sig` file on a FAT32-formatted USB drive.
    1. Eject the USB drive from your computer and insert it into the printer.
    1. On the printer, open the offline updates menu and install the COSMOS update from the USB drive.
    1. After the printer reboots into COSMOS, copy `ELEGOO.txt` from the USB drive to a safe location on your computer.
    1. Complete the calibration and import the CC2 slicer profile described below before printing.

    !!! warning "Keep your Elegoo credentials backup"

        Installation saves the printer's Elegoo credentials to `ELEGOO.txt` on the USB drive. If the internal storage is ever erased, this file is needed to restore Elegoo cloud connectivity when returning to stock firmware. Keep it safe and do not share it.

## After installation

Connect to the printer's IP address in a web browser to open its web interface. Run `CHECK_CALIBRATION` from the Macros section, then select `Calibrate All`. This calibrates resonance compensation, the extruder PID, the load cells, the Z offset, and the bed mesh. If the automatic routine cannot finish, see the [manual calibration steps](./features.md#manual-calibration).

### Required OrcaSlicer profile

Import and use the latest COSMOS profile for your printer before printing. These profiles provide the required machine start and end G-code.

=== "Centauri Carbon 1"

    Use the [OpenCentauri COSMOS OrcaSlicer profile for CC1](https://cloud.orcaslicer.com/b/3fad3c38f25f), required from COSMOS 26.07.0 onwards.

=== "Centauri Carbon 2 / Centauri 2"

    Use the [OpenCentauri COSMOS OrcaSlicer profile for CC2](https://cloud.orcaslicer.com/b/331b32d95aab), required from COSMOS 26.10.0 onwards.

!!! warning "Old profiles are rejected"

    Older machine G-code that calls `M729` or `M8213` triggers an emergency stop and displays a message telling you to use the COSMOS start and end machine G-code. Update the profile before trying to print again.

Set the printer connection in OrcaSlicer to:

- Host Type: `Moonraker (Klipper)`
   - Note: The Moonraker host type lets you select the upload location during a print/upload dialog. Upload to the `gcodes` folder.
- Printer Agent: Moonraker
- Hostname, IP or URL: your printer's IP address

## Update

These methods apply to printers already running COSMOS. For a first installation on stock firmware, use the model-specific instructions above.

By default, COSMOS checks for updates shortly after startup while the printer is idle. When an update is available, the printer displays an `Update Available` prompt with an `Update Now` button. The automatic check and release channel can be changed in [`cosmos.conf`](./features.md#cosmos-settings).

=== "Via Screen (Online)"

    1. Navigate to the settings tab on the printer's screen.
    1. Click `Update COSMOS`.
    1. Wait for the process to complete.

=== "Via SSH (Online)"

    1. Log into your printer via SSH.
    1. Run the command `update-cosmos`
    1. Wait for the process to complete.

=== "Via USB (Offline)"

    1. Download the latest COSMOS SWU for your printer:
        - **CC1:** [`cosmos-centauri-carbon-1.swu`](https://github.com/OpenCentauri/cosmos/releases/latest/download/cosmos-centauri-carbon-1.swu).
        - **CC2 / C2:** [`cosmos-centauri-carbon-2.swu`](https://github.com/OpenCentauri/cosmos/releases/latest/download/cosmos-centauri-carbon-2.swu).
    1. Rename it to `emergency.swu`
    1. Insert a fat32-formatted thumbdrive into your pc.
    1. Put `emergency.swu` on the root of the thumbdrive.
    1. Eject your thumbdrive from your pc and insert it into your Centauri Carbon.
    1. Wait for the update process to complete. This should be indicated by an on-screen UI prompt (only appears if Klipper is loaded)

=== "Via SSH (Manual, Offline)"

    Download the `.swu` for your printer model using the links in the USB tab above. You can also flash it via SSH. Just place the .swu somewhere accessible to the machine (like in /user-resource via SCP/SSH), SSH into your machine, then run `flash /path/to/.swu`.

## Uninstall

!!! Warning
    Please do not try to flash an official .swu or OpenCentauri patched .swu directly. This will skip downgrading the toolhead/bed and will cause a brick!

=== "Via Screen (Online)"

    1. Navigate to the settings tab on the printer's screen.
    1. Click `Switch to OC Patched`.
    1. Wait for the process to complete.

=== "Via SSH (Online)"

    1. Log into your printer via SSH.
    1. Run the command `switch-to-oc-patched`
    1. Wait for the process to complete.
