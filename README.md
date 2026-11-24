# esp32-cam-rs

[![Crates.io](https://img.shields.io/crates/v/esp32-cam-rs.svg)](https://crates.io/crates/esp32-cam-rs)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

Safe Rust wrapper around Espressif's [`esp32-camera`](https://github.com/espressif/esp32-camera)
driver for the **ESP32-CAM (AI-Thinker, OV2640)**, built on
[`esp-idf-svc`](https://github.com/esp-rs/esp-idf-svc).

## Features

- Automatic integration of the `espressif/esp32-camera` ESP-IDF component and its bindings
- Typed configuration: resolution, JPEG quality, buffer count, PSRAM/DRAM, grab mode
- Ready-made pin map for the AI-Thinker board (`CameraPins::AI_THINKER`), custom pins supported
- RAII frame buffers: a `FrameBuffer` is returned to the driver when dropped
- Runtime sensor controls: vertical flip, horizontal mirror, quality, frame size
- Singleton protection: a second `Camera::new` returns an error instead of corrupting the driver

## Requirements

- An ESP32 with a supported camera and **PSRAM** (the AI-Thinker ESP32-CAM has 4 MB)
- The Rust ESP toolchain set up as described in the [esp-rs book](https://docs.esp-rs.org/book/)
  (`espup`, `ldproxy`, `espflash`), using the `std` / ESP-IDF approach
- A project generated from [`esp-idf-template`](https://github.com/esp-rs/esp-idf-template)

## Installation

```toml
[dependencies]
esp32-cam-rs = "0.1"
esp-idf-svc = "0.53"
```

Enable PSRAM in your project's `sdkconfig.defaults`:

```ini
CONFIG_SPIRAM=y
CONFIG_SPIRAM_SPEED_80M=y
CONFIG_ESP_MAIN_TASK_STACK_SIZE=8000
```

> The `esp32-camera` component is declared in this crate's
> `[package.metadata.esp-idf-sys.extra_components]`. If your build does not
> pick it up from the dependency, copy that block (with `bindings.h`) into your
> own project's `Cargo.toml`.

## Usage

```rust,ignore
use esp32_cam_rs::{Camera, Config, FrameSize};

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();

    let camera = Camera::init_ai_thinker_with(&Config {
        frame_size: FrameSize::Svga,
        jpeg_quality: 12, // 0..=63, lower = better quality
        ..Config::default()
    })?;

    camera.set_vflip(true)?;

    if let Some(frame) = camera.capture() {
        if frame.looks_like_jpeg() {
            println!("{}x{} - {} bytes", frame.width(), frame.height(), frame.len());
            // `frame` derefs to `&[u8]`: write it to a socket, SD card, ...
        }
    } // frame returned to the driver here

    Ok(())
}
```

A complete Wi-Fi + HTTP snapshot server is in
[`examples/http_snapshot.rs`](https://github.com/jorgeandrecastro/esp32-cam-rs/blob/main/examples/http_snapshot.rs).

### Other boards

```rust,ignore
use esp32_cam_rs::{Camera, CameraPins, Config};

let pins = CameraPins {
    pwdn: -1, reset: -1, xclk: 15, sda: 4, scl: 5,
    data: [11, 9, 8, 10, 12, 18, 17, 16],
    vsync: 6, href: 7, pclk: 13,
};
let camera = Camera::new(&pins, &Config::default())?;
```

## Tips and troubleshooting

- **`Camera probe failed` / `0x105`**: wrong pins, bad power supply (use a solid 5 V), or camera ribbon not seated.
- **Out of memory / `0x101`**: PSRAM not enabled, or frame size too large; try `FrameSize::Vga`.
- **Truncated or corrupted JPEGs**: lower the `xclk_freq_hz` (e.g. 10 MHz), improve the power supply, and check `frame.looks_like_jpeg()`.
- Hold a `FrameBuffer` as briefly as possible: the driver has only `fb_count` buffers.

## Documentation

docs.rs cannot compile ESP-IDF projects, so build the API docs locally:

```sh
source ~/export-esp.sh
cargo clean
rm -rf .embuild
cargo doc --no-deps --open
```

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](https://github.com/jorgeandrecastro/esp32-cam-rs/blob/main/LICENSE-APACHE))
- MIT license ([LICENSE-MIT](https://github.com/jorgeandrecastro/esp32-cam-rs/blob/main/LICENSE-MIT))

at your option.

Third-party components (notably Espressif's `esp32-camera` and ESP-IDF, both
Apache-2.0) are listed in [NOTICE](https://github.com/jorgeandrecastro/esp32-cam-rs/blob/main/NOTICE).

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.

---

Author: [Jorge Andre Castro](https://github.com/jorgeandrecastro)