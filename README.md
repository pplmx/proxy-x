# proxy-x

[![Crates.io](https://img.shields.io/crates/v/proxy-x.svg)](https://crates.io/crates/proxy-x)
[![Docs.rs](https://docs.rs/proxy-x/badge.svg)](https://docs.rs/proxy-x)
[![CI](https://github.com/pplmx/proxy-x/workflows/CI/badge.svg)](https://github.com/pplmx/proxy-x/actions)

## Installation

### Cargo

- Install the rust toolchain in order to have cargo installed by following
  [this](https://www.rust-lang.org/tools/install) guide.
- run `cargo install proxy-x`

### Usage

- run `proxy-x` to see the help message.
- run `proxy-x enable http://localhost:7890` to enable the proxy.
- run `proxy-x disable` to disable the proxy.
- run `proxy-x ip` to get the current agent ip.
- run `proxy-x ping example.com` to ping the target dns name or ip.

## License

Licensed under either of

- Apache License, Version 2.0
  ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license
  ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dually licensed as above, without any additional terms or conditions.

See [CONTRIBUTING.md](CONTRIBUTING.md).
