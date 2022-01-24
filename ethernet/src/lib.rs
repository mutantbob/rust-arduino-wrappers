#![no_std]

pub mod raw;

use crate::raw::EthernetUDP;
use core::convert::TryInto;
pub use raw::IPAddress;

pub fn ip_address_4(a: u8, b: u8, c: u8, d: u8) -> IPAddress {
    unsafe { IPAddress::new1(a, b, c, d) }
}

pub fn begin_mac(mac: &mut [u8; 6], timeout: u32, response_timeout: u32) -> i16 {
    unsafe {
        let mac_ptr: *mut u8 = mac.as_mut_ptr();

        raw::EthernetClass::begin(mac_ptr, timeout, response_timeout)
    }
}

pub fn begin1(mac: &mut [u8; 6], ip: IPAddress) {
    unsafe {
        let mac_ptr: *mut u8 = mac.as_mut_ptr();
        raw::EthernetClass::begin1(mac_ptr, ip)
    }
}

pub fn new_udp(port: u16) -> EthernetUDP {
    let mut rval = ::core::mem::MaybeUninit::uninit();
    unsafe {
        raw::EthernetUDP_begin(
            rval.as_mut_ptr() as *mut rust_arduino_runtime::workaround_cty::c_void,
            port,
        );

        rval.assume_init()
    }
}

impl EthernetUDP {
    pub fn send_to(
        &mut self,
        destination_ip: IPAddress,
        destination_port: u16,
        payload: &mut [u8],
    ) -> raw::size_t {
        use rust_arduino_runtime::workaround_cty::*;
        unsafe {
            let this = self as *mut Self as *mut c_void;
            let n1 = raw::EthernetUDP_beginPacket(this, destination_ip, destination_port);
            let packet_len: u16 = payload.len().try_into().unwrap();
            let n2 = raw::EthernetUDP_write1(this, payload.as_mut_ptr(), packet_len);
            let n3 = raw::EthernetUDP_endPacket(this);

            n1 as c_uint + n2 + n3 as c_uint
        }
    }
}
