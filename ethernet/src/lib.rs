#![no_std]

pub mod raw;

use core::convert::TryInto;
use ufmt::{uDisplay,Formatter, uWrite};
pub use raw::{IPAddress,EthernetServer, EthernetUDP, EthernetClient};

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

pub fn local_ip() -> IPAddress {
    unsafe {
        raw::EthernetClass_localIP()
    }
}

impl uDisplay for IPAddress
{
    fn fmt<W>(&self, formatter:&mut Formatter<W>) -> Result<(), W::Error>
        where
            W: uWrite + ?Sized
    {
        let x = unsafe { & self._address.bytes};
        x[0].fmt(formatter)?;
        '.'.fmt(formatter)?;
        x[1].fmt(formatter)?;
        '.'.fmt(formatter)?;
        x[2].fmt(formatter)?;
        '.'.fmt(formatter)?;
        x[3].fmt(formatter)?;
        Ok(())
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


impl EthernetServer
{
    pub fn new(port: u16) -> EthernetServer
    {
        unsafe { raw::fabricate_EthernetServer(port) }
    }

    pub fn begin(&mut self)
    {
        unsafe {
            raw::virtual_EthernetServer_begin(self as *mut EthernetServer)
        }
    }

    pub fn available_safe(&mut self) -> Option<EthernetClient>
    {
        let rval = unsafe { self.available() };
        if rval.valid() {
            Some(rval)
        } else {
            None
        }
    }

}

impl EthernetClient
{
    pub fn available_for_write(&mut self) -> i16
    {
        unsafe {
            raw::virtual_EthernetClient_availableForWrite(self as *mut EthernetClient)
        }
    }

    pub fn connected(&mut self) -> bool
    {
        unsafe {
            raw::virtual_EthernetClient_connected(self as *mut EthernetClient)
        }
    }

    pub fn available(&mut self) -> i16
    {
        unsafe {
            raw::virtual_EthernetClient_available(self as *mut EthernetClient)
        }
    }

    pub fn read(&mut self) -> Option<u8>
    {
        unsafe {
            let rval = raw::virtual_EthernetClient_read(self as *mut EthernetClient);
            if rval&0xff == rval {
                Some(rval as u8)
            } else {
                None
            }
        }
    }

    pub fn println(&mut self, msg: &[u8]) -> u16
    {
        unsafe {
            raw::virtual_EthernetClient_println(self as *mut EthernetClient, msg.as_ptr())
        }
    }

    pub fn flush(&mut self)
    {
        unsafe {
            raw::virtual_EthernetClient_flush(self as *mut EthernetClient)
        }
    }

    pub fn stop(&mut self)
    {
        unsafe {
            raw::virtual_EthernetClient_stop(self as *mut EthernetClient)
        }
    }

    pub fn valid(&self) ->bool
    {
        unsafe {
            raw::EthernetClient_valid(self as *const EthernetClient)
        }
    }
}
