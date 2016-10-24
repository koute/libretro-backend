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
[dependencies]
libretro-backend = "0.1"
```

and this to your crate root:

```rust
#[macro_use]
extern crate libretro_backend;
```

then just implement the [Backend trait]:

```rust
struct Emulator {
    // ...
}

impl libretro_backend::Backend for Emulator {
    // ...
}
```

and use a macro:

```rust
libretro_backend!({
    Box::new( Emulator {
        // ...
    })
});
```

For a full example you can check out [this file], which is part of my NES
emulator [Pinky].

[Backend trait]: https://docs.rs/libretro-backend/*/libretro_backend/trait.Backend.html
[this file]: https://github.com/koute/pinky/blob/master/pinky-libretro/src/lib.rs
[Pinky]: https://github.com/koute/pinky
