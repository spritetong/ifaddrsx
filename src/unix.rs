use get_if_addrs::{get_if_addrs, IfAddr};
use ipnetwork::IpNetwork;
use std::net::IpAddr;

#[cfg(feature = "mac")]
use nix::{ifaddrs::*, sys::socket::SockAddr};

use crate::Interface;

/// Get all network interfaces.
pub fn get_interfaces() -> std::io::Result<Vec<Interface>> {
    
    #[cfg(feature = "mac")]
    let mut mac_map = std::collections::BTreeMap::<String, [u8; 6]>::new();
    #[cfg(feature = "mac")]
    for interface in getifaddrs()? {
        if let Some(SockAddr::Link(link)) = interface.address {
            mac_map.insert(interface.interface_name.clone(), link.addr());
        }
    }

    let mut rst = vec![];
    for nif in get_if_addrs()? {
        let addr = match &nif.addr {
            IfAddr::V4(v) => IpNetwork::with_netmask(IpAddr::V4(v.ip), IpAddr::V4(v.netmask)),
            IfAddr::V6(v) => IpNetwork::with_netmask(IpAddr::V6(v.ip), IpAddr::V6(v.netmask)),
        };
        if let Ok(ip) = addr {
            #[cfg(feature = "mac")]
            let mac_addr = match mac_map.get(&nif.name) {
                Some(v) => *v,
                _ => [0u8; 6],
            };

            rst.push(Interface {
                name: nif.name.clone(),
                #[cfg(feature = "friendly")]
                friendly_name: nif.name.clone(),
                ip,
                #[cfg(feature = "mac")]
                mac_addr,
            });
        }
    }

    Ok(rst)
}

/// Get all network interfaces' IP addresses.
pub fn get_ifaddrs() -> std::io::Result<Vec<IpNetwork>> {
    let mut rst = vec![];
    for nif in get_if_addrs()? {
        let addr = match &nif.addr {
            IfAddr::V4(v) => IpNetwork::with_netmask(IpAddr::V4(v.ip), IpAddr::V4(v.netmask)),
            IfAddr::V6(v) => IpNetwork::with_netmask(IpAddr::V6(v.ip), IpAddr::V6(v.netmask)),
        };
        if let Ok(ip) = addr {
            rst.push(ip);
        }
    }

    Ok(rst)
}
