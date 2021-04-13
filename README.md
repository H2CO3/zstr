# `zstr!`: Zero-terminated C string literals

This crate provides a minimal, safe, zero-cost function-like procedural
macro, `zstr!()`, that creates 0-terminated `&'static CStr` values from
regular Rust string (or byte string) literals.

The basic usage is:

```rust
let c_str = zstr!("Hello World!");
```

See the [documentation](https://docs.rs/zstr) for more examples.
