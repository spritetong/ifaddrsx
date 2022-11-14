use ipnetwork::IpNetwork;
use libc::c_char;
use std::ffi::CStr;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use winapi::shared::ifdef::IfOperStatusUp;
use winapi::shared::ntdef::{PVOID, ULONG};
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

/// Get all network interfaces.
pub fn get_interfaces() -> std::io::Result<Vec<Interface>> {
    unsafe {
        let mut size: ULONG = 1024;
        let mut buffer = Vec::with_capacity(size as usize);

        let sizeptr: *mut ULONG = &mut size;
        let mut res = GetAdaptersAddresses(0, 0, 0 as PVOID, buffer.as_mut_ptr(), sizeptr);

        // Since we are providing the buffer, it might be too small. Check for overflow
        // and try again with the required buffer size. There is a chance for a race
        // condition here if an interface is added between the two calls - however
        // looping potentially forever seems more dangerous.
        if res == ERROR_BUFFER_OVERFLOW {
            buffer.reserve(size as usize - buffer.len());
            res = GetAdaptersAddresses(0, 0, 0 as PVOID, buffer.as_mut_ptr(), sizeptr);
        }

        if res != ERROR_SUCCESS {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "GetAdaptersAddresses failed",
            ));
        }

        let mut res = Vec::new();
        let mut adapterptr = buffer.as_ptr() as PIP_ADAPTER_ADDRESSES;
        while !adapterptr.is_null() {
            let a = *adapterptr;

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

                    assert_ne!(current, addr.Next);
                    current = addr.Next;
                }
            }

            assert_ne!(adapterptr, a.Next);
            adapterptr = a.Next;
        }

        Ok(res)
    }
}

/// Get all network interfaces' IP addresses.
pub fn get_ifaddrs() -> std::io::Result<Vec<IpNetwork>> {
    unsafe {
        let mut size: ULONG = 1024;
        let mut buffer = Vec::with_capacity(size as usize);

        let sizeptr: *mut ULONG = &mut size;
        let mut res = GetAdaptersAddresses(0, 0, 0 as PVOID, buffer.as_mut_ptr(), sizeptr);

        // Since we are providing the buffer, it might be too small. Check for overflow
        // and try again with the required buffer size. There is a chance for a race
        // condition here if an interface is added between the two calls - however
        // looping potentially forever seems more dangerous.
        if res == ERROR_BUFFER_OVERFLOW {
            buffer.reserve(size as usize - buffer.len());
            res = GetAdaptersAddresses(0, 0, 0 as PVOID, buffer.as_mut_ptr(), sizeptr);
        }

        if res != ERROR_SUCCESS {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "GetAdaptersAddresses failed",
            ));
        }

        let mut res = Vec::new();
        let mut adapterptr = buffer.as_ptr() as PIP_ADAPTER_ADDRESSES;
        while !adapterptr.is_null() {
            let a = *adapterptr;

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

                    assert_ne!(current, addr.Next);
                    current = addr.Next;
                }
            }

            assert_ne!(adapterptr, a.Next);
            adapterptr = a.Next;
        }

        Ok(res)
    }
}
