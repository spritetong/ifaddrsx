#[cfg(windows)]
#[path = "windows.rs"]
mod platform;

#[cfg(not(windows))]
#[path = "unix.rs"]
mod platform;

pub use ipnetwork::IpNetwork;
pub use platform::*;

bitflags::bitflags! {
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
    pub struct IfFlags: u32 {
        /// The interface is running.
        const UP          = 1 << 0;
        /// Support broadcast.
        const BROADCAST   = 1 << 1;
        /// Support multicast.
        const MULTICAST   = 1 << 2;
        /// Support IPv4.
        const IPV4        = 1 << 3;
        /// Support IPv6.
        const IPV6        = 1 << 4;
        /// Interface is a loopback interface.
        const LOOPBACK    = 1 << 5;
        /// Interface is a point-to-point link.
        const PPP         = 1 << 6;
        /// TUN device (no Ethernet headers)
        const TUN         = 1 << 7;
        /// TAP device
        const TAP         = 1 << 8;
    }
}

#[derive(Clone, Debug, Default)]
pub struct Interface {
    /// The name of the interface.
    pub name: String,
    /// On Windows, the friendly name; on other platforms, the same as `name`.
    friendly_name: Option<String>,
    /// The interface index.
    pub index: usize,
    /// (Windows Only) The interface index of IPv6.
    pub ipv6_index: usize,
    /// The interface flags.
    pub flags: IfFlags,
    /// The network IP address of the interface.
    pub ips: Vec<IpNetwork>,
    /// The MAC address.
    pub mac_addr: Option<[u8; 6]>,
}

impl Interface {
    /// On Windows, returns the friendly name; on other platforms, returns `name`.
    pub fn friendly_name(&self) -> &str {
        self.friendly_name.as_ref().unwrap_or(&self.name)
    }

    /// Check if this interface is up / running.
    #[inline]
    pub fn is_up(&self) -> bool {
        self.flags.contains(IfFlags::UP)
    }

    /// Check if this is a loopback interface.
    #[inline]
    pub fn is_loopback(&self) -> bool {
        self.flags.contains(IfFlags::LOOPBACK)
    }
}

////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_interfaces() {
        println!("{:#?}", get_interfaces(false));
        println!("{:#?}", get_ifaddrs(true));
    }
}
