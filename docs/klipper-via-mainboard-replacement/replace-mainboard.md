# Replace mainboard

## Picking a mainboard

You will need any 3D printer mainboard that:

- accepts 24V power
- has three fan outputs (ideally 3 pins, for mainboard cooling, side and chamber)
- has three stepper drivers (ideally TMC2209 or similar, for X, Y and Z)
- has support for sensorless homing with those two of those drivers (X and Y axes)
- has support for two thermistors (bed and chamber)
- has support for one heater output (bed)
- has support for at least two endstops (optical Z endstop and filament sensor)

You will also need a Linux computer to run the Klipper host on, which can be a Raspberry Pi or similar.

!!! warning

    The Raspberry Pi Zero W and Zero 2 W are not recommended due to their limited performance and occasional dropouts when restarting Klipper.

From our testing, the following boards are known to work:

<!-- TODO: Discuss if we want affiliate links -->

- [BigTreeTech SKR Mini E3 V3.0](https://biqu.equipment/products/bigtreetech-skr-mini-e3-v2-0-32-bit-control-board-for-ender-3)
- [BigTreeTech Octopus v1.1](https://biqu.equipment/products/bigtreetech-octopus-v1-1)
  - On the BTT Octopus, you will have to snip off the JST-XA lever because the sockets are too close together

## Wiring up the mainboard

You will need to make three custom cables to connect various components to the new mainboard:

### Bed heater cable

The bed heater cable is a 2-pin JST-XA connector, but most mainboards use a 2-pin terminal block instead.

There is not much power going through this cable, so we recommend finding a 2-pin JST-XA socket, soldering two cables to it, then crimping the other ends with ferrules to connect to the terminal block.

!!! note

    Make sure you connect the bed heater cable in the correct polarity. If you connect it backwards, the bed won't heat up.

!!! warning

    TODO(devminer): add picture of the cable

### Optical Z endstop cable

The endstop needs 24V power, ground and a signal wire, but most mainboards only either have a 2-pin connector for ground and signal or a 3-pin connector for power, ground and signal, where the power pin is 3.3V or 5V. Because of this we have to wire up our own harness for this.

!!! warning

    TODO(devminer): add picture of the cable

### Mainboard fan cable

The mainboard fan cable is a 4-pin JST-XA connector, but most mainboards use a 3-pin JST-XH instead. We need to omit the 4th pin (the tachometer pin) since it's a signal pin on 24V level and needs some more wiring to a octocoupler to work properly.

!!! warning

    TODO(devminer): add picture of the cable
