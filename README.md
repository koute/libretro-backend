# Libretro API bindings for Rust

This crate exposes idiomatic Rust API bindings to the excellent [libretro] API.

The target audience of this library are emulator authors who want to turn
their emulator into a libretro core, which relieves them from the necessity of
creating a full blown frontend for their emulator and allows them to concentrate
on actual emulation.

In its current state there is still **a lot** of features missing, nevertheless
it should be useful enough to create a basic emulator.

For an example on how to use this crate you can check out [this file], which
is part of my NES emulator [Pinky].

As always, contributions are welcome!

[libretro]: http://www.libretro.com/index.php/api/
[this file]: https://github.com/koute/pinky/blob/master/pinky-libretro/src/lib.rs
[Pinky]: https://github.com/koute/pinky
