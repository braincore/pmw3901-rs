# PMW3901 Library for Rust [![Latest Version]][crates.io] [![Documentation]][docs.rs]

[Latest Version]: https://img.shields.io/crates/v/pmw3901.svg
[crates.io]: https://crates.io/crates/pmw3901
[Documentation]: https://docs.rs/pmw3901/badge.svg
[docs.rs]: https://docs.rs/pmw3901

A library for the PMW3901 optical flow sensor.

Intended to be deployed in a linux environment with
[spidev](https://www.kernel.org/doc/Documentation/spi/spidev).

## Usage

See `examples/scan.rs`.

## Limitations

The datasheet for the PMW3901 is sparse.  The list of registers comes with
little explanation. This library does the bare minimum to read pixel velocity.

In addition, the initialization write sequence is opaque and is simply copied
from [Bitcraze](https://github.com/bitcraze/Bitcraze_PMW3901).

## Testing

Use environment variables to specify the SPI bus and chip select:

```
PMW3901_SPI_BUS=0 PMW3901_SPI_CS=0 cargo test
```

Tested on the [breakout board by Pesky Product](
https://www.tindie.com/products/onehorse/pmw3901-optical-flow-sensor/).

## Todo

- [ ] Control over the EN pin. Currently assumes it's pulled up.
- [ ] Motion detection interrupt pin.
- [ ] Control over the NRESET pin. Currently assumes it's pulled up.
