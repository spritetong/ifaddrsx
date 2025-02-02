# ifaddrsx

[![Crates.io](https://img.shields.io/crates/v/ifaddrsx.svg)](https://crates.io/crates/ifaddrsx)
[![Documentation](https://docs.rs/ifaddrsx/badge.svg)](https://docs.rs/ifaddrsx)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A Rust library for retrieving network interface information, including names and IP addresses of all active network interfaces. This crate provides a cross-platform solution that works on Windows, Linux, macOS, and other Unix-like systems.

## Features

- Get network interface names
- Retrieve IPv4 and IPv6 addresses for each interface
- Cross-platform support (Windows, Linux, macOS, and other Unix-like systems)
- Easy-to-use API
- No unsafe code (except for necessary FFI bindings)

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
ifaddrsx = "0.3"
```

## Usage

```rust
use ifaddrsx::get_ifaddrs;

fn main() {
    let ifaddrs = get_ifaddrs().unwrap();
    for ifaddr in ifaddrs {
        println!("Interface: {}", ifaddr.name);
        println!("  IP Addresses: {:?}", ifaddr.ips);
    }
}
```

## Platform Support


- Windows (via WinAPI)
- Linux (via netlink)
- macOS and other Unix-like systems (via getifaddrs)

## Dependencies

- `bitflags`: For flag handling
- `ipnetwork`: IP address manipulation
- `libc`: System library bindings
- Platform-specific dependencies:
  - Windows: `winapi` with network-related features
  - Unix: `nix` for network operations

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Author

Sprite Tong <spritetong@gmail.com>
