# quadro-ctl

CLI tool for bulk read/write operations on the [Aqua Computer QUADRO](https://www.aquacomputer.de/quadro.html) fan controller via hidraw.

## Why

The sysfs/hwmon interface performs individual USB round-trips for each operation (~9 seconds each). Configuring all 4 fans (PWM, temperature curves, sensor assignment) takes ~20 minutes via sysfs.

`quadro-ctl` reads and writes the control report directly via `/dev/hidrawX`, configuring everything in a single read-modify-write cycle (~18 seconds total).

## Usage

### Read current configuration

```sh
sudo quadro-ctl read
```

```json
{
  "fans": {
    "fan1": {
      "mode": "curve",
      "sensor": 0,
      "points": [
        { "temp": 20000, "percentage": 20 },
        { "temp": 25000, "percentage": 40 },
        ...
      ]
    },
    "fan2": {
      "mode": "manual",
      "percentage": 40
    }
  }
}
```

### Apply configuration

Set fan2 to manual 60%:

```sh
echo '{"fans":{"fan2":{"mode":"manual","percentage":60}}}' > config.json
sudo quadro-ctl apply --config-file config.json
```

Set fan1 to a temperature curve on sensor 0:

```json
{
  "fans": {
    "fan1": {
      "mode": "curve",
      "sensor": 0,
      "points": [
        { "temp": 20000, "percentage": 20 },
        { "temp": 22000, "percentage": 25 },
        { "temp": 24000, "percentage": 30 },
        { "temp": 26000, "percentage": 35 },
        { "temp": 28000, "percentage": 40 },
        { "temp": 30000, "percentage": 45 },
        { "temp": 32000, "percentage": 50 },
        { "temp": 34000, "percentage": 55 },
        { "temp": 36000, "percentage": 60 },
        { "temp": 38000, "percentage": 65 },
        { "temp": 40000, "percentage": 70 },
        { "temp": 42000, "percentage": 75 },
        { "temp": 44000, "percentage": 80 },
        { "temp": 46000, "percentage": 85 },
        { "temp": 48000, "percentage": 90 },
        { "temp": 50000, "percentage": 100 }
      ]
    }
  }
}
```

Curves require exactly 16 points with strictly increasing temperatures. Temperatures are in millicelsius (20000 = 20.0°C). Sensors are 0-3 (the 4 QUADRO temperature inputs).

### Specify device path

```sh
sudo quadro-ctl read --device /dev/hidraw0
sudo quadro-ctl apply --device /dev/hidraw0 --config-file config.json
```

If `--device` is omitted, the QUADRO is auto-detected by scanning `/dev/hidraw*`.

## Config format

The config is partial — only include the fans you want to change. Unconfigured fans are preserved as-is.

| Field | Values |
|-------|--------|
| `mode` | `"manual"` or `"curve"` |
| `percentage` | 0-100 (manual mode) |
| `sensor` | 0-3 (curve mode, selects QUADRO temp input) |
| `points` | Array of 16 `{ "temp", "percentage" }` objects (curve mode) |

## Building

```sh
cargo build --release
```

Requires Linux (uses hidraw ioctls). Runs on x86_64 and aarch64.

## Reference

Protocol derived from the [aquacomputer_d5next](https://github.com/aleksamagicka/aquacomputer_d5next-hwmon) out-of-tree kernel driver.
