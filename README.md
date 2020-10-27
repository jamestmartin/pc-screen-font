# PC Screen Font
A Rust library which can parses PSF version 2 fonts from their bytes.
It correctly handles fonts which contain extra pixels in their padding bits.

This library does not depend on the Rust standard library,
making it suitable for inclusion in bootloaders or kernels,
although it does require a working global allocator.

This library predates [the PSF crate](https://crates.io/crates/psf).
I have not investigated it at all, but you may wish to investigate using it instead
before choosing to use this library.

## Maintenance status
This library is only passively maintained, receiving updates as I need them.
I do not intend to publish this library to crates.io, but I will accept pull requests,
and if you wish to publish this library to crates.io and maintain it yourself,
you are welcome to do so.

## Limitations
Currently, this library does not support multi-codepoint spellings of characters.
Support may be added at some point, but I do not currently have any plans to do so.
