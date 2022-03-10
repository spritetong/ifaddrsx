#[cfg(windows)]
#[path = "windows.rs"]
mod platform;

#[cfg(not(windows))]
#[path = "unix.rs"]
mod platform;

pub use ipnetwork::IpNetwork;
pub use platform::*;

#[derive(Clone, Debug)]
pub struct Interface {
    /// The name of the interface.
    pub name: String,
    /// The friendly name.
    #[cfg(feature = "friendly")]
    pub friendly_name: String,
    /// The network IP address of the interface.
    pub ip: IpNetwork,
    /// The MAC address.
    #[cfg(feature = "mac")]
    pub mac_addr: [u8; 6],
}

impl Interface {
    /// Check if it's an interface.
    #[inline]
    pub fn is_loopback(&self) -> bool {
        self.ip.ip().is_loopback()
    }
}

////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_interfaces() {
        println!("{:?}", get_interfaces());
        println!("{:?}", get_ifaddrs());
    }
}
