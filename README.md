# Base32 Encoder/Decoder

A high-performance Base32 implementation in Rust using Crockford's Base32 alphabet.
Uses Zero Dependency.


## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
base32 = { path = "./base32" }
```


### Crockford's Base32 Alphabet

Uses Crockford's Base32 alphabet with these characters:
```
0123456789abcdefghjkmnpqrstvwxyz
```

Note: Characters 'i', 'l', 'o', 'u' are excluded to avoid confusion with numbers and to improve readability.


## Attribution

Partial fork of [andreasots/base32](https://github.com/andreasots/base32) @ commit 58909ac

This project is based on the excellent work by The base32 Developers, with significant optimizations and extensions for ARM64 architecture and performance analysis.

## License

Licensed under the MIT License - See LICENSE file for details

Original work copyright (c) 2015 The base32 Developers
