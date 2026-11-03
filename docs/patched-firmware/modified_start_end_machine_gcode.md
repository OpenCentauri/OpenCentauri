# Modified machine start/end Gcode

## Features

- Purge line has been moved from the front left to the front middle to reduce cable fatigue
- Preheat the Chamber for engineering-grade Materials (requires patched firmware)
- Chamber light automatically turns on when starting a print (requires patched firmware)
- Chamber light automatically turns off when a print finishes (requires patched firmware)

## Installation

1. Open up OrcaSlicer
2. Edit your printer profile (Edit icon next to `Elegoo Centauri Carbon 0.4 nozzle`)
3. Turn on the advanced toggle
4. Go to Machine G-code
5. Paste in the following sections
6. Save as a new profile

Machine start G-code:

```gcode
;;===== date: 20240520 =====================
;printer_model:[printer_model]
;initial_filament:{filament_type[initial_extruder]}
;curr_bed_type:{curr_bed_type}
M8213 ; Turn on light
M400 ; wait for buffer to clear
M220 S100 ; Set the feed speed to 100%
M221 S100 ; Set the flow rate to 100%
M104 S140
G90
{if chamber_temperature[initial_no_support_extruder] >35} ; Heat the Chamber
G28 ; cold Homing
;(Uncomment if Recirculation Mod is installed) M106 P3 S255 ; Turn on Chamber Fan for Recirulation and Air Filtration
;(Uncomment if Recirculation Mod is installed) M106 P2 S128 ; Turn on Aux Fan to aid Chambber Heating
G1 Z130 ; Set Buildplate at half height for better heat dissipation
{if bed_temperature[initial_no_support_extruder] >60}
M190 S110
{elsif bed_temperature[initial_no_support_extruder] <60}
M190 S60
{endif}
TEMPERATURE_WAIT SENSOR=box MINIMUM=[chamber_temperature]
M140 S[bed_temperature_initial_layer_single] ; initiate cooldown for printing
M106 P2 S0
{endif}
M190 S[bed_temperature_initial_layer_single]
G28 ; home
M729 ; Clean Nozzle

;=============turn on fans to prevent PLA jamming=================
{if filament_type[initial_no_support_extruder]=="PLA"}
    {if (bed_temperature[initial_no_support_extruder] >50)||(bed_temperature_initial_layer[initial_no_support_extruder] >50)}
    M106 P3 S180
    {elsif (bed_temperature[initial_no_support_extruder] >45)||(bed_temperature_initial_layer[initial_no_support_extruder] >45)}
    M106 P3 S180
    {endif};Prevent PLA from jamming
{endif}

;enable_pressure_advance:{enable_pressure_advance[initial_extruder]}
; This value is called if pressure advance is enabled
{if enable_pressure_advance[initial_extruder] == "true"}
SET_PRESSURE_ADVANCE ADVANCE=[pressure_advance]
M400
{endif}
M204 S{min(20000,max(1000,outer_wall_acceleration))} ;Call exterior wall print acceleration


G1 X{print_bed_max[0]*0.5+40+50} Y-1.2 F20000
G1 Z0.3 F900
M109 S[nozzle_temperature_initial_layer]
M83
G92 E0 ; Reset Extruder
G1 F{min(6000, max(900, filament_max_volumetric_speed[initial_no_support_extruder]/0.5/0.3*60))} 
G1 X60 E12 ; Draw the first line
G1 Y-0.3
G1 X{print_bed_max[0]*0.5+40-50} E4.284
G1 F{0.2*min(12000, max(1200, filament_max_volumetric_speed[initial_no_support_extruder]/0.5/0.3*60))} 
G1 X{print_bed_max[0]*0.5+40-30} E2
G1 F{min(12000, max(1200, filament_max_volumetric_speed[initial_no_support_extruder]/0.5/0.3*60))} 
G1 X{print_bed_max[0]*0.5+40-10} E2
G1 F{0.2*min(12000, max(1200, filament_max_volumetric_speed[initial_no_support_extruder]/0.5/0.3*60))} 
G1 X{print_bed_max[0]*0.5+40+10} E2
G1 F{min(12000, max(1200, filament_max_volumetric_speed[initial_no_support_extruder]/0.5/0.3*60))} 
G1 X{print_bed_max[0]*0.5+40+30} E2
G1 F{min(12000, max(1200, filament_max_volumetric_speed[initial_no_support_extruder]/0.5/0.3*60))} 
G1 X{print_bed_max[0]*0.5+40+50} E2
; End PA test.


G3 I-1 J0 Z0.6 F1200.0 ; Move to side a little
G1 F20000
G92 E0 ;Reset Extruder
;LAYER_COUNT:[total_layer_count]
;LAYER:0
SET_PRINT_STATS_INFO TOTAL_LAYER=[total_layer_count]
```

Machine end G-code:

```gcode
;===== date: 20250109 =====================
M400 ; wait for buffer to clear
M140 S0 ; Turn-off bed
M106 S255 ; Cooling nozzle
M83
G92 E0 ; zero the extruder
G2 I1 J0 Z{max_layer_z+0.5} E-1 F3000 ; lower z a little
G90
{if max_layer_z > 50}G1 Z{min(max_layer_z+50, printable_height+0.5)} F20000{else}G1 Z100 F20000 {endif} ; Move print head up 
M204 S5000
M400
M83
G1 X202 F20000
M400
G1 Y250 F20000
G1 Y264.5 F1200
M400
G92 E0
M104 S0 ; Turn-off hotend
M140 S0 ; Turn-off bed
M106 S0 ; turn off fan
M106 P2 S0 ; turn off remote part cooling fan
M106 P3 S0 ; turn off chamber cooling fan
M84 ;Disable all steppers
M8212 ; Turn off light
```
