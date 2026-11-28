# Booting into Allwinner FEL mode

!!! warning
    Booting into FEL mode requires shorting out a resistor on the board. You can damage or kill your board if you're not careful!

!!! danger
    Making mistakes while in FEL mode CAN break your board! Be careful!

!!! info "Hardware Setup Required"
    Before following this guide, ensure you have properly connected your board for FEL access. You will need either:
    
    - **USB-C cable connection** (with VCC blocked), OR
    - **Soldered header to FEL pins** (J5 connector)
    
    See the [FEL & UART Bench Setup Guide](../hardware/CC1/fel-uart-setup.md) for detailed hardware connection instructions including both USB connection methods, UART setup, and power/safety considerations.

## Prerequisites

### Install sunxi-fel Tool

Install the sunxi-fel utility (provides `sunxi-fel` command):

```bash
sudo apt install sunxi-tools
```

#### Windows

For Windows, a precompiled `sunxi-fel.exe` from BigTreeTech is available in the
[cc-fw-tools repo](https://github.com/OpenCentauri/cc-fw-tools/blob/main/extra-stuff/emmc/sunxi-fel.exe).
Download it, drop it in your working folder, and run `.\sunxi-fel.exe ...` in place of the
`sunxi-fel ...` commands below. You'll still need the WinUSB driver (see
[Windows: Install USB Driver](#windows-install-usb-driver)).

### Install xfel Tool

Clone and build the allwinner-xfel repository:

```bash
git clone https://gitlab.alerv.net/clone/allwinner-xfel.git
cd allwinner-xfel
make
```

### Windows: Install USB Driver

If you're on Windows, you'll need to install the WinUSB driver:

1. Use [Zadig](https://zadig.akeo.ie/) to install the WinUSB driver. 
    - ![img](assets/DriverInstall.jpg){ width="400" }
1. Check the USB ID of the device. It has to be ``1F3A`` ``EFE8``.
1. Rename the unknown device to a name of your choice by enabling "Edit"
1. Make sure the correct driver is selected: ``WinUSB``
1. Click the install button

## Entering FEL Mode

There are two methods to boot into FEL mode:

### Method 1: Hardware Reset (From Power-On)

Press the reset button while shorting R53 with tweezers. The board should boot into FEL mode and connect to your PC.

![FEL_MODE](./assets/FelMode.jpg){ width="400" }

When successful, you should see on the UART console:

```
[25]HELLO! BOOT0 is starting!
[27]BOOT0 commit : 4d16602
```

!!! note
    This UART output will only display if you are booting from u-Boot (using the `efex` command) and your BOOT0 has UART enabled. OpenCentauri enables UART by default in the latest versions. If booting directly into FEL mode via hardware reset, you may not see this output.

You should also see a red LED on the board blink **once** when it enters FEL mode.

![Red LED that blinks once on FEL entry](assets/fel-mode-red-led.jpg){ width="300" }


### Method 2: Software Boot (From u-Boot)

If you already have u-Boot running and UART access, you can boot directly to FEL mode by issuing this command in u-Boot:

```
efex
```

This will reboot the system directly into FEL mode without needing to short the resistor.

## Working with FEL Mode

### Step 1: Verify FEL Communication

First, verify that FEL mode is working and you can communicate with the chip:

```bash
xfel version
```

You should see output similar to:

```
AWUSBFEX ID=0x00185900(R528/T113) dflag=0x44 dlength=0x08 scratchpad=0x00045000
```

This confirms that FEL mode is active and communication is working.

### Step 2: Test with Hello World SPL

Run a simple test to verify everything is functioning correctly:

```bash
sunxi-fel spl uart0-helloworld-sdboot.sunxi
```

!!! note
    The `uart0-helloworld-sdboot.sunxi` file can be found in the [cc-fw-tools](https://github.com/OpenCentauri/cc-fw-tools) repository at `/extra-stuff/emmc/uart0-helloworld-sdboot.sunxi`.

You should see on the UART console:

```
Hello from Allwinner R528/T113!
Returning back to FEL.
```

!!! success
    If you see this message, everything is working! You can run the `xfel version` and `sunxi-fel spl` commands multiple times and they should continue to function.

!!! info "UART Initialization Required"
    This command appears to be required to run at least once to correctly initialize the Serial UART on the R528 chip before proceeding with DDR initialization and loading u-Boot.

### Alternative: Load u-Boot in One Step (sunxi-fel)

Instead of Steps 3 and 4 below (the `xfel` DDR-init → write → exec sequence), you can load u-Boot
with a single `sunxi-fel` command that handles DRAM init, loading, and execution at once:

```bash
sunxi-fel uboot u-boot-sunxi-with-spl-cc1.bin
```

The [`u-boot-sunxi-with-spl-cc1.bin`](https://github.com/OpenCentauri/cc-fw-tools/blob/main/extra-stuff/emmc/u-boot-sunxi-with-spl-cc1.bin)
build is in the [cc-fw-tools](https://github.com/OpenCentauri/cc-fw-tools) repo. You'll land at the
`=>` U-Boot prompt on the UART console (no key-press needed), then continue from
[eMMC Recovery](#emmc-recovery).

### Step 3: Initialize DDR

Now that FEL mode is verified, initialize the DDR memory:

```bash
./xfel ddr r528-s3
```

The UART console should display:

```
[33211]fes begin commit:4d16602
[33214]set pll start
[33220]periph0 has been enabled
[33223]set pll end
[33225][pmu]: bus read error
[33228]board init ok
[33230]beign to init dram
[33232]ZQ value = 0x2f
[33234]get_pmu_exist() = -1
[33237]ddr_efuse_type: 0xf
[33239]trefi:7.8ms
[33242][AUTO DEBUG] single rank and full DQ!
[33246]ddr_efuse_type: 0xf
[33248]trefi:7.8ms
[33251][AUTO DEBUG] rank 0 row = 13
[33254][AUTO DEBUG] rank 0 bank = 8
[33257][AUTO DEBUG] rank 0 page size = 2 KB
[33261]DRAM BOOT DRIVE INFO: V0.33
[33265]DRAM CLK = 792 MHz
[33267]DRAM Type = 3 (2:DDR2,3:DDR3)
[33270]DRAMC read ODT  off.
[33273]DRAM ODT value: 0x42.
[33276]ddr_efuse_type: 0xf
[33279]DRAM SIZE =128 M
[33281]dram_tpr4:0x0
[33283]PLL_DDR_CTRL_REG:0xf8004100
[33286]DRAM_CLK_REG:0xc0000000
[33289][TIMING DEBUG] MR2= 0x18
[33293]DRAM simple test OK.
[33295]rtc standby flag is 0x0, super standby flag is 0x0
[33301]init dram ok
```

### Step 4: Load and Execute u-Boot

Load a u-Boot image to DRAM via FEL:

```bash
xfel write 0x43000000 uboot239.bin
```

!!! note
    The `uboot239.bin` file can be found in the [cc-fw-tools](https://github.com/OpenCentauri/cc-fw-tools) repository at `/extra-stuff/emmc/uboot239.bin`.

Execute the loaded u-Boot:

```bash
xfel exec 0x43000b50
```

Hold down `s` on the UART console to boot into interactive mode.

## eMMC Recovery

Once you have u-Boot running, you can recover the eMMC from a USB stick.

!!! tip "Complete eMMC Recovery Guide"
    For detailed instructions on eMMC recovery, backup, and restore procedures, see the [EMMC_RESTORE.md](https://github.com/OpenCentauri/cc-fw-tools/blob/main/docs/EMMC_RESTORE.md) documentation in the [cc-fw-tools](https://github.com/OpenCentauri/cc-fw-tools) repository.
    
    This guide covers:
    
    - Loading files from USB stick
    - Flashing to eMMC partitions
    - Creating custom `.scr` scripts for automated recovery
    - Pre-built scripts for full eMMC backup and restore (found in `./extra-stuff/emmc/`)

With u-Boot, you can load files from a USB stick and flash them to any eMMC partition by creating and building custom `.txt` scripts into `.scr` files.
