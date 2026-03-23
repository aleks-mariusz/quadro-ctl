# CLAUDE.md

## Overview

CLI tool in Rust for bulk read/write operations on the Aqua Computer QUADRO fan controller via hidraw.

Bypasses the sysfs/hwmon interface (where each operation takes ~9 seconds due to individual USB round-trips) by reading and writing the control report directly via `/dev/hidrawX`. This allows configuring all 4 fans (PWM, temperature curves, sensor assignment) in a single read-modify-write cycle (~18 seconds total vs ~20 minutes via sysfs).

## What it does

- Reads the QUADRO control report from `/dev/hidrawX` (1 USB read)
- Modifies fan configuration in the buffer (in memory, instant)
- Computes CRC-16/USB checksum (init/xorout 0xffff)
- Writes the full control report back (1 USB write)

### Supported operations per fan (fan1-fan4)

- **Manual mode**: set fixed PWM percentage (0-100), sets pwm_enable=1
- **Curve mode**: program 16-point temperature curve (temp + PWM pairs), sets pwm_enable=2
- **Sensor assignment**: select which QUADRO temperature sensor (1-4) governs the fan's curve

## Reference driver

The protocol, offsets, buffer format, and checksum algorithm are all derived from the out-of-tree kernel driver:

- **Path**: `/Users/danielramos/Documents/repos/others/aquacomputer_d5next-hwmon/aquacomputer_d5next.c`
- **Upstream**: https://github.com/aleksamagicka/aquacomputer_d5next-hwmon
- **Key constants to extract**: `QUADRO_CTRL_REPORT_SIZE`, `quadro_ctrl_fan_offsets`, `AQC_FAN_CTRL_*` offsets, `ctrl_report_id`, CRC-16 parameters

## Consumer

This tool is consumed by the NixOS module `services.quadro-fans` in the `nas` repository (`/Users/danielramos/Documents/repos/infra/nas`). The NixOS module generates a systemd service that calls `quadro-ctl` to configure fans at boot.

## Code Guidelines

- **Never add comments to code** - Code should be self-explanatory through clear naming and structure
- **Commits must be single-line** - No multi-line messages, no co-authors
