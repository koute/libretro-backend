# Libretro API bindings for Rust

[![Documentation](https://docs.rs/libretro-backend/badge.svg)](https://docs.rs/libretro-backend/*/libretro_backend/)

This crate exposes idiomatic Rust API bindings to the excellent [libretro] API.

The target audience of this library are emulator authors who want to turn
their emulator into a libretro core, which relieves them from the necessity of
creating a full blown frontend for their emulator and allows them to concentrate
on actual emulation.

In its current state there is still **a lot** of features missing, nevertheless
it should be useful enough to create a basic emulator.

As always, contributions are welcome!

[libretro]: http://www.libretro.com/index.php/api/

## Getting started

Add this to your `Cargo.toml`:

```toml
[lib]
crate-type = ["cdylib"]

[dependencies]
libretro-backend = "0.2"
```

and this to your crate root:

```rust
#[macro_use]
extern crate libretro_backend;
```

then just implement the [Core trait]:

```rust
struct Emulator {
    // ...
}

impl libretro_backend::Core for Emulator {
    // ...
}
```

and use a macro:

```rust
libretro_core!( Emulator );
```

For a full example you can check out [this file], which is part of my NES
emulator [Pinky].

[Core trait]: https://docs.rs/libretro-backend/*/libretro_backend/trait.Core.html
[this file]: https://github.com/koute/pinky/blob/master/pinky-libretro/src/lib.rs
[Pinky]: https://github.com/koute/pinky

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license,
shall be dual licensed as above, without any additional terms or conditions.
