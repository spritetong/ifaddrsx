use ipnetwork::IpNetwork;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use nix::{
    ifaddrs::*,
    sys::socket::{LinkAddr, SockaddrIn, SockaddrIn6, SockaddrLike},
};

use crate::Interface;

/// Get all network interfaces.
pub fn get_interfaces() -> std::io::Result<Vec<Interface>> {
    let mut if_map = std::collections::BTreeMap::<String, (usize, [u8; 6])>::new();
    let mut interfaces = vec![];

    for nif in getifaddrs()? {
        let InterfaceAddress {
            interface_name: nif_name,
            address: nif_address,
            netmask: nif_netmask,
            ..
        } = nif;

        if let Some(addr) = nif_address {
            if let Some(link) = unsafe { LinkAddr::from_raw(addr.as_ptr(), None) } {
                match link.addr() {
                    Some(mac) => {
                        if_map.insert(nif_name, (link.ifindex(), mac));
                    }
                    _ => {
                        if_map
                            .entry(nif_name)
                            .or_insert_with(|| (link.ifindex(), [0u8; 6]));
                    }
                }
                continue;
            }

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
                    interfaces.push(Interface {
                        #[cfg(feature = "friendly")]
                        friendly_name: nif_name.clone(),
                        name: nif_name,
                        index: 0,
                        ip,
                        mac_addr: [0u8; 6],
                    });
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
                    interfaces.push(Interface {
                        #[cfg(feature = "friendly")]
                        friendly_name: nif_name.clone(),
                        name: nif_name,
                        index: 0,
                        ip,
                        mac_addr: [0u8; 6],
                    });
                }
                continue;
            }
        }
    }

    for interface in interfaces.iter_mut() {
        if let Some(&(index, mac)) = if_map.get(&interface.name) {
            interface.index = index;
            interface.mac_addr = mac;
        }
    }

    Ok(interfaces)
}

/// Get all network interfaces' IP addresses.
pub fn get_ifaddrs() -> std::io::Result<Vec<IpNetwork>> {
    let mut addrs = vec![];

    for nif in getifaddrs()? {
        let InterfaceAddress {
            address: nif_address,
            netmask: nif_netmask,
            ..
        } = nif;

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
