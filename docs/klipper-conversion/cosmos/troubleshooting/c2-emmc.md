# Centauri 2 eMMC incompatibility

Some Centauri 2 units _(not Centauri <ins>**Carbon**</ins> 2)_ have been identified with an incompatible eMMC flash
storage chip. Before flashing COSMOS, please open the bottom panel and check your mainboard. These
units can be identified by the PCB revision number and the serial number printed on the eMMC chip
itself.

## Affected board identifiers

| PCB Revision   | eMMC Model    | eMMC Serial No. |
| -------------- | ------------- | --------------- |
| EF-M5-V1.0.154 | PSE3A0SL-08GE | 004288817       |

| PCB Revision                                                           | eMMC Chip Markings                                          |
| ---------------------------------------------------------------------- | ----------------------------------------------------------- |
| ![Affected board revision](../assets/c2_emmc_issue/affected_board.png) | ![Affected eMMC](../assets/c2_emmc_issue/affected_emmc.png) |

If your eMMC chip is labelled "Rayson" or has a different serial number, it's not known to be
affected by this issue and you can proceed with COSMOS installation.

!!! warning "Recovery process"
    If you've already flashed COSMOS to an affected board and are now stuck in a boot loop,
    recovery requires serial console access using a USB-to-Serial adapter and jumper cable.

    Documentation is in progress, if you need help in the meantime please reach out on the
    [OpenCentauri Discord server](https://discord.gg/t6Cft3wNJ3).