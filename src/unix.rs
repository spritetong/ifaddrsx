use ipnetwork::IpNetwork;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use nix::{
    ifaddrs::*,
    net::if_::InterfaceFlags,
    sys::socket::{LinkAddr, SockaddrIn, SockaddrIn6, SockaddrLike},
};

use crate::{IfFlags, Interface};

/// Get all network interfaces.
///
/// `only_up` controls whether only up interfaces are returned.
pub fn get_interfaces(only_up: bool) -> std::io::Result<Vec<Interface>> {
    let mut interfaces = vec![];

    fn get_if_mut(
        interfaces: &mut Vec<Interface>,
        if_name: String,
        if_flags: InterfaceFlags,
    ) -> &mut Interface {
        let pos = match interfaces.iter().rev().position(|x| x.name == if_name) {
            Some(n) => interfaces.len() - n - 1,
            _ => {
                let mut flags = IfFlags::empty();
                if if_flags.contains(InterfaceFlags::IFF_UP) {
                    flags.insert(IfFlags::UP);
                }
                if if_flags.contains(InterfaceFlags::IFF_BROADCAST) {
                    flags.insert(IfFlags::BROADCAST);
                }
                if if_flags.contains(InterfaceFlags::IFF_MULTICAST) {
                    flags.insert(IfFlags::MULTICAST);
                }
                flags.insert(IfFlags::IPV4);
                flags.insert(IfFlags::IPV6);
                if if_flags.contains(InterfaceFlags::IFF_LOOPBACK) {
                    flags.insert(IfFlags::LOOPBACK);
                }
                if if_flags.contains(InterfaceFlags::IFF_POINTOPOINT) {
                    flags.insert(IfFlags::PPP);
                }
                // TAP or TUN
                #[cfg(any(target_os = "linux", target_os = "android", target_os = "fuchsia"))]
                if if_flags.contains(InterfaceFlags::IFF_TUN) {
                    flags.insert(IfFlags::TUN);
                }
                #[cfg(any(target_os = "linux", target_os = "android", target_os = "fuchsia"))]
                if if_flags.contains(InterfaceFlags::IFF_TAP) {
                    flags.insert(IfFlags::TAP);
                }
                if if_flags.contains(InterfaceFlags::IFF_POINTOPOINT | InterfaceFlags::IFF_NOARP) {
                    if flags.contains(IfFlags::TAP) {
                        flags.remove(IfFlags::TAP);
                    } else {
                        flags.insert(IfFlags::TUN);
                    }
                }

                interfaces.push(Interface {
                    name: if_name,
                    friendly_name: None,
                    index: 0,
                    ipv6_index: 0,
                    flags,
                    ips: Vec::new(),
                    mac_addr: None,
                });
                interfaces.len() - 1
            }
        };
        &mut interfaces[pos]
    }

    for if_addr in getifaddrs()? {
        let InterfaceAddress {
            interface_name: nif_name,
            flags: nif_flags,
            address: nif_address,
            netmask: nif_netmask,
            ..
        } = if_addr;

        if only_up && !nif_flags.contains(InterfaceFlags::IFF_UP) {
            continue;
        }

        let Some(addr) = nif_address else {
            continue;
        };

        if let Some(link) = unsafe { LinkAddr::from_raw(addr.as_ptr(), None) } {
            let nif = get_if_mut(&mut interfaces, nif_name, nif_flags);
            nif.index = link.ifindex();
            nif.ipv6_index = nif.index;
            if let Some(mac) = link.addr() {
                if !mac.iter().all(|&x| x == 0) {
                    nif.mac_addr = Some(mac);
                }
            }
            continue;
        } else if let Some(addr) = unsafe { SockaddrIn::from_raw(addr.as_ptr(), None) } {
            let netmask =
                match nif_netmask.and_then(|x| unsafe { SockaddrIn::from_raw(x.as_ptr(), None) }) {
                    Some(v) => v.ip().into(),
                    _ => Ipv4Addr::BROADCAST,
                };
            if let Ok(ip) =
                IpNetwork::with_netmask(IpAddr::V4(addr.ip().into()), IpAddr::V4(netmask))
            {
                let nif = get_if_mut(&mut interfaces, nif_name, nif_flags);
                nif.ips.push(ip);
                continue;
            }
        } else if let Some(addr) = unsafe { SockaddrIn6::from_raw(addr.as_ptr(), None) } {
            let netmask = match nif_netmask
                .and_then(|x| unsafe { SockaddrIn6::from_raw(x.as_ptr(), None) })
            {
                Some(v) => v.ip(),
                _ => Ipv6Addr::new(
                    0xffff, 0xffff, 0xffff, 0xffff, 0xffff, 0xffff, 0xffff, 0xffff,
                ),
            };
            if let Ok(ip) = IpNetwork::with_netmask(IpAddr::V6(addr.ip()), IpAddr::V6(netmask)) {
                let nif = get_if_mut(&mut interfaces, nif_name, nif_flags);
                nif.ips.push(ip);
                continue;
            }
        }

        let _ = get_if_mut(&mut interfaces, nif_name, nif_flags);
    }

    interfaces.iter_mut().for_each(|nif| {
        if nif.mac_addr.is_none() {
            nif.flags.remove(IfFlags::TAP);
        } else {
            nif.flags.remove(IfFlags::TUN);
        }
    });

    Ok(interfaces)
}

/// Get all network interfaces' IP addresses.
///
/// `only_up` controls whether only up interfaces' IP addresses are returned.
pub fn get_ifaddrs(only_up: bool) -> std::io::Result<Vec<IpNetwork>> {
    let mut addrs = vec![];

    for nif in getifaddrs()? {
        let InterfaceAddress {
            address: nif_address,
            flags: nif_flags,
            netmask: nif_netmask,
            ..
        } = nif;

        if only_up && !nif_flags.contains(InterfaceFlags::IFF_UP) {
            continue;
        }

        if let Some(addr) = nif_address {
            if let Some(addr) = unsafe { SockaddrIn::from_raw(addr.as_ptr(), None) } {
                let netmask = match nif_netmask
                    .and_then(|x| unsafe { SockaddrIn::from_raw(x.as_ptr(), None) })
                {
                    Some(v) => v.ip().into(),
                    _ => Ipv4Addr::BROADCAST,
                };
                if let Ok(ip) =
                    IpNetwork::with_netmask(IpAddr::V4(addr.ip().into()), IpAddr::V4(netmask))
                {
                    addrs.push(ip);
                }
                continue;
            }

            if let Some(addr) = unsafe { SockaddrIn6::from_raw(addr.as_ptr(), None) } {
                let netmask = match nif_netmask
                    .and_then(|x| unsafe { SockaddrIn6::from_raw(x.as_ptr(), None) })
                {
                    Some(v) => v.ip(),
                    _ => Ipv6Addr::new(
                        0xffff, 0xffff, 0xffff, 0xffff, 0xffff, 0xffff, 0xffff, 0xffff,
                    ),
                };
                if let Ok(ip) = IpNetwork::with_netmask(IpAddr::V6(addr.ip()), IpAddr::V6(netmask))
                {
                    addrs.push(ip);
                }
                continue;
            }
        }
    }

    Ok(addrs)
}
