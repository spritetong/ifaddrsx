use ipnetwork::IpNetwork;
use libc::c_char;
use std::ffi::CStr;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use winapi::shared::ifdef::IfOperStatusUp;
use winapi::shared::ntdef::ULONG;
use winapi::shared::winerror::*;
use winapi::shared::ws2def::{AF_INET, AF_INET6, SOCKADDR_IN};
use winapi::shared::ws2ipdef::SOCKADDR_IN6_LH;
use winapi::um::iphlpapi::*;
use winapi::um::iptypes::*;

#[cfg(feature = "friendly")]
use libc::wchar_t;

use crate::Interface;

fn cstr_to_string(cstr: *const c_char) -> String {
    unsafe { CStr::from_ptr(cstr).to_string_lossy().into() }
}

#[cfg(feature = "friendly")]
fn wcs_to_string(wstr: *const wchar_t) -> String {
    unsafe { String::from_utf16_lossy(std::slice::from_raw_parts(wstr, libc::wcslen(wstr))) }
}

fn get_adapter_addresses() -> std::io::Result<Vec<u8>> {
    let mut size: ULONG = 1000;
    let mut buffer = Vec::<u8>::with_capacity(size as usize);

    for _ in 0..10 {
        match unsafe {
            GetAdaptersAddresses(
                0,
                0,
                std::ptr::null_mut(),
                buffer.as_ptr() as PIP_ADAPTER_ADDRESSES,
                &mut size,
            )
        } {
            ERROR_SUCCESS => return Ok(buffer),
            ERROR_BUFFER_OVERFLOW if size > 0 => {
                // Enlarge the buffer and try again.
                buffer.reserve(size as usize);
            }
            _ => break,
        }
    }

    Err(std::io::Error::new(
        std::io::ErrorKind::Other,
        "GetAdaptersAddresses failed",
    ))
}

/// Get all network interfaces.
pub fn get_interfaces() -> std::io::Result<Vec<Interface>> {
    unsafe {
        let buffer = get_adapter_addresses()?;

        let mut res = Vec::new();
        let mut adapter = buffer.as_ptr() as PIP_ADAPTER_ADDRESSES;
        while !adapter.is_null() {
            let a = &*adapter;

            if a.OperStatus == IfOperStatusUp {
                let mut current = a.FirstUnicastAddress;

                while !current.is_null() {
                    let addr = &*current;
                    match (*addr.Address.lpSockaddr).sa_family as i32 {
                        AF_INET => {
                            let sin = *(addr.Address.lpSockaddr as *const SOCKADDR_IN);
                            let ip = IpAddr::V4(Ipv4Addr::from(u32::from_be(
                                *sin.sin_addr.S_un.S_addr(),
                            )));
                            if let Ok(ip) = IpNetwork::new(ip, addr.OnLinkPrefixLength) {
                                res.push(Interface {
                                    name: cstr_to_string(a.AdapterName),
                                    #[cfg(feature = "friendly")]
                                    friendly_name: wcs_to_string(a.FriendlyName),
                                    index: a.u.s().IfIndex as usize,
                                    ip,
                                    mac_addr: a.PhysicalAddress[..6].try_into().unwrap(),
                                });
                            }
                        }
                        AF_INET6 => {
                            let sin6 = *(addr.Address.lpSockaddr as *const SOCKADDR_IN6_LH);
                            let ip = IpAddr::V6(Ipv6Addr::from(*sin6.sin6_addr.u.Byte()));
                            if let Ok(ip) = IpNetwork::new(ip, addr.OnLinkPrefixLength) {
                                res.push(Interface {
                                    name: cstr_to_string(a.AdapterName),
                                    #[cfg(feature = "friendly")]
                                    friendly_name: wcs_to_string(a.FriendlyName),
                                    index: a.u.s().IfIndex as usize,
                                    ip,
                                    mac_addr: a.PhysicalAddress[..6].try_into().unwrap(),
                                });
                            }
                        }
                        _ => (),
                    }

                    current = addr.Next;
                }
            }

            adapter = a.Next;
        }

        Ok(res)
    }
}

/// Get all network interfaces' IP addresses.
pub fn get_ifaddrs() -> std::io::Result<Vec<IpNetwork>> {
    unsafe {
        let buffer = get_adapter_addresses()?;

        let mut res = Vec::new();
        let mut adapter = buffer.as_ptr() as PIP_ADAPTER_ADDRESSES;
        while !adapter.is_null() {
            let a = *adapter;

            if a.OperStatus == IfOperStatusUp {
                let mut current = a.FirstUnicastAddress;

                while !current.is_null() {
                    let addr = &*current;
                    match (*addr.Address.lpSockaddr).sa_family as i32 {
                        AF_INET => {
                            let sin = *(addr.Address.lpSockaddr as *const SOCKADDR_IN);
                            let ip = IpAddr::V4(Ipv4Addr::from(u32::from_be(
                                *sin.sin_addr.S_un.S_addr(),
                            )));
                            if let Ok(addr) = IpNetwork::new(ip, addr.OnLinkPrefixLength) {
                                res.push(addr);
                            }
                        }
                        AF_INET6 => {
                            let sin6 = *(addr.Address.lpSockaddr as *const SOCKADDR_IN6_LH);
                            let ip = IpAddr::V6(Ipv6Addr::from(*sin6.sin6_addr.u.Byte()));
                            if let Ok(addr) = IpNetwork::new(ip, addr.OnLinkPrefixLength) {
                                res.push(addr);
                            }
                        }
                        _ => (),
                    }

                    current = addr.Next;
                }
            }

            adapter = a.Next;
        }

        Ok(res)
    }
}
