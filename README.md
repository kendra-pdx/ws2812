# WS2812 Embedded Driver

This project aims to be a **simple** driver for WS2812 / NeoPixels in embedded `#![no_std]` environments.

## Features

- [x] Basic 'Color' structs
- [x] `const`s defining for the protocol definition.
- [x] ESP32 and ESP32S3 support via `esp_hal::rmt`
- [ ] Color correction
- [ ] 32-bit color (RGBW)

## Building
Note: This project does not currently build on its own. It is intended to be used as a submodule in your embedded ESP32 project with the toolchain.

## Feature Flags
- `esp`: Enables the `esp` module. Requires one of:
    - `esp32`: Enables ESP32 support
    - `esp32s3`: Enables ESP32-S3 support
- `bevy`: Enable `bevy_color` compatibility
- `defmt`: Enable defmt logging in dependencies
- `timings_spec`: Use the timings specified in the WS2812 protocol spec. Otherwise, use adjusted timings.