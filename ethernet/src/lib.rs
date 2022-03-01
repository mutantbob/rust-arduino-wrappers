#![no_std]

pub mod raw;

use crate::EthernetInitializationMalfunction::{DhcpFailed, MissingHardware};
use avr_hal_generic::port::mode::Output;
use avr_hal_generic::port::Pin;
use core::convert::TryInto;
pub use raw::{EthernetClient, EthernetServer, EthernetUDP};
use rust_arduino_helpers::NumberedPin;
pub use rust_arduino_runtime::client::Client;
pub use rust_arduino_runtime::ip_address::IPAddress;
use ufmt::{uWrite, Formatter};

pub enum LinkStatus {
    Unknown,
    LinkOn,
    LinkOff,
    Madness(u16),
}

impl From<u16> for LinkStatus {
    fn from(raw: u16) -> Self {
        match raw {
            raw::EthernetLinkStatus_Unknown => LinkStatus::Unknown,
            raw::EthernetLinkStatus_LinkON => LinkStatus::LinkOn,
            raw::EthernetLinkStatus_LinkOFF => LinkStatus::LinkOff,
            _ => LinkStatus::Madness(raw),
        }
    }
}

//

pub enum HardwareStatus {
    NoHardware,
    W5100,
    W5200,
    W5500,
    Madness(u16),
}

impl From<u16> for HardwareStatus {
    fn from(raw: u16) -> Self {
        match raw {
            raw::EthernetHardwareStatus_EthernetNoHardware => HardwareStatus::NoHardware,
            raw::EthernetHardwareStatus_EthernetW5100 => HardwareStatus::W5100,
            raw::EthernetHardwareStatus_EthernetW5200 => HardwareStatus::W5200,
            raw::EthernetHardwareStatus_EthernetW5500 => HardwareStatus::W5500,
            _ => HardwareStatus::Madness(raw),
        }
    }
}

//

pub enum EthernetInitializationMalfunction<P: NumberedPin> {
    DhcpFailed(EthernetBuilder<P>),
    LinkOff(EthernetBuilder<P>),
    MissingHardware(Pin<Output, P>),
}

impl<P: NumberedPin> core::fmt::Debug for EthernetInitializationMalfunction<P> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            DhcpFailed(_) => f.write_str("DHCP failed"),
            EthernetInitializationMalfunction::LinkOff(_) => {
                f.write_str("Link Off (cable unplugged?)")
            }
            MissingHardware(_) => f.write_str("Missing ethernet hardware"),
        }
    }
}

impl<P: NumberedPin> ufmt::uDebug for EthernetInitializationMalfunction<P> {
    fn fmt<W>(&self, f: &mut Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: uWrite + ?Sized,
    {
        match self {
            DhcpFailed(_) => f.write_str("DHCP failed"),
            EthernetInitializationMalfunction::LinkOff(_) => {
                f.write_str("Link Off (cable unplugged?)")
            }
            MissingHardware(_) => f.write_str("Missing ethernet hardware"),
        }
    }
}

//

pub struct EthernetBuilder<P: NumberedPin> {
    pin: Pin<Output, P>,
}

impl<P: NumberedPin> EthernetBuilder<P> {
    pub fn dhcp_lease(
        self,
        mac: &mut [u8; 6],
        timeout: u32,
        response_timeout: u32,
    ) -> Result<EthernetWrapper<P>, EthernetInitializationMalfunction<P>> {
        let code = unsafe {
            let mac_ptr: *mut u8 = mac.as_mut_ptr();
            raw::EthernetClass::begin(mac_ptr, timeout, response_timeout)
        };
        if code == 1 {
            Ok(self.with_pin())
        } else {
            match self.link_status() {
                LinkStatus::LinkOn => Err(DhcpFailed(self)),
                LinkStatus::LinkOff => Err(EthernetInitializationMalfunction::LinkOff(self)),
                LinkStatus::Unknown | LinkStatus::Madness(_) => Err(MissingHardware(self.pin)),
            }
        }
    }

    fn with_pin(self) -> EthernetWrapper<P> {
        unsafe { raw::EthernetClass_init(P::pin_number()) }
        EthernetWrapper { pin: self.pin }
    }

    pub fn static_ip(
        self,
        mac: &mut [u8; 6],
        ip: rust_arduino_runtime::ip_address::IPAddress,
    ) -> Result<EthernetWrapper<P>, EthernetInitializationMalfunction<P>> {
        unsafe {
            let mac_ptr: *mut u8 = mac.as_mut_ptr();
            raw::EthernetClass::begin1(mac_ptr, ip)
        }

        Ok(self.error_if_no_hardware()?.with_pin())
    }

    pub fn static_ip_with_dns(
        self,
        mac: &mut [u8; 6],
        ip: IPAddress,
        dns: IPAddress,
    ) -> Result<EthernetWrapper<P>, EthernetInitializationMalfunction<P>> {
        unsafe {
            let mac_ptr: *mut u8 = mac.as_mut_ptr();
            raw::EthernetClass::begin2(mac_ptr, ip, dns)
        }

        Ok(self.error_if_no_hardware()?.with_pin())
    }

    pub fn link_status(&self) -> LinkStatus {
        unsafe { raw::EthernetClass::linkStatus().into() }
    }

    pub fn hardware_status(&self) -> HardwareStatus {
        unsafe { raw::EthernetClass::hardwareStatus().into() }
    }

    pub fn error_if_no_hardware(self) -> Result<Self, EthernetInitializationMalfunction<P>> {
        match self.hardware_status() {
            HardwareStatus::NoHardware | HardwareStatus::Madness(_) => {
                Err(EthernetInitializationMalfunction::MissingHardware(self.pin))
            }
            _ => Ok(self),
        }
    }
}

//

/// ```
/// let dp = arduino_hal::Peripherals::take().unwrap();
/// let pins = pins!(dp);
/// let mut mac = [0xde, 0xad, 0xbe, 0xef, 1, 2];
/// let ethernet = EthernetWrapper::builder(pins.d10.into_output())
///         .static_ip(&mut mac, ip_address_4(192, 168, 8, 167)) ?;
/// ```
pub struct EthernetWrapper<P: NumberedPin> {
    pin: Pin<Output, P>,
}

impl<P: NumberedPin> EthernetWrapper<P> {
    pub fn tcp_listen(&self, port: u16) -> EthernetServer {
        unsafe {
            let mut rval = raw::fabricate_EthernetServer(port);
            raw::virtual_EthernetServer_begin(&mut rval as *mut EthernetServer); // would you ever NOT want to call `.begin()`?
            rval
        }
    }
}

impl<P: NumberedPin> EthernetWrapper<P> {
    /// which pin?
    ///
    /// 10: most shields
    ///
    /// 5: MKR Eth Shield
    ///
    /// 0: Teensy 2.0
    ///
    /// 20: Teensy++ 2.0
    ///
    /// 15: ESP8266 with Adafruit FeatherWing
    ///
    /// 33: ESP32 with Adafruit FeatherWing
    pub fn builder(spi_cs_pin: Pin<Output, P>) -> EthernetBuilder<P> {
        EthernetBuilder { pin: spi_cs_pin }
    }

    pub fn link_status(&self) -> LinkStatus {
        unsafe { raw::EthernetClass::linkStatus().into() }
    }

    pub fn new_udp(&self, port: u16) -> EthernetUDP {
        let mut rval = ::core::mem::MaybeUninit::uninit();
        unsafe {
            raw::EthernetUDP_begin(rval.as_mut_ptr() as *mut cty::c_void, port);

            rval.assume_init()
        }
    }

    pub fn local_ip(&self) -> IPAddress {
        unsafe { raw::EthernetClass_localIP() }
    }

    pub fn dns_server_ip(&self) -> &'static IPAddress {
        unsafe { &raw::EthernetClass__dnsServerAddress } // stupid inline method
    }

    pub fn reclaim_pin(self) -> Pin<Output, P> {
        self.pin
    }

    pub fn tcp_connect_hostname(
        &self,
        host_name: &cstr_core::CStr,
        port: u16,
    ) -> Result<EthernetClient, i16> {
        let mut rval = EthernetClient::new();

        let return_code = unsafe {
            raw::virtual_EthernetClient_connect_hostname(
                &mut rval as *mut EthernetClient,
                host_name.as_ptr() as *const i8,
                port,
            )
        };
        if return_code != 0 {
            Ok(rval)
        } else {
            Err(return_code)
        }
    }

    pub fn make_client(&self) -> EthernetClient {
        EthernetClient::new()
    }
}

impl EthernetUDP {
    pub fn send_to(
        &mut self,
        destination_ip: IPAddress,
        destination_port: u16,
        payload: &mut [u8],
    ) -> raw::size_t {
        use cty::*;
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

/// To create one of these, use [`EthernetWrapper::tcp_listen`]
impl EthernetServer {
    pub fn available_safe(&mut self) -> Option<EthernetClient> {
        let rval = unsafe { self.available() };
        if rval.valid() {
            Some(rval)
        } else {
            None
        }
    }
}

impl EthernetClient {
    fn new() -> Self {
        unsafe { raw::fabricate_EthernetClient() }
    }

    pub fn available_for_write(&mut self) -> i16 {
        unsafe { raw::virtual_EthernetClient_availableForWrite(self as *mut EthernetClient) }
    }

    pub fn connected(&mut self) -> bool {
        unsafe { raw::virtual_EthernetClient_connected(self as *mut EthernetClient) }
    }

    pub fn available(&mut self) -> i16 {
        unsafe { raw::virtual_EthernetClient_available(self as *mut EthernetClient) }
    }

    pub fn write(&mut self, buffer: &[u8]) -> Result<(), SocketError> {
        let n = unsafe {
            raw::virtual_EthernetClient_write(
                self as *mut EthernetClient,
                buffer.as_ptr(),
                buffer.len().try_into().unwrap(),
            )
        };
        if n == 0 {
            // what is the error signaling method?  The base method returns a size_t which is unsigned
            Err(SocketError::new("failed to write to socket"))
        } else {
            Ok(())
        }
    }

    pub fn read(&mut self) -> Option<u8> {
        unsafe {
            let rval = raw::virtual_EthernetClient_read(self as *mut EthernetClient);
            if rval & 0xff == rval {
                Some(rval as u8)
            } else {
                None
            }
        }
    }

    pub fn read_multi<'a>(&mut self, dest: &'a mut [u8]) -> Result<&'a [u8], SocketError> {
        let buf = dest.as_mut_ptr();
        let size = dest.len().try_into().unwrap();
        let code = unsafe {
            raw::virtual_EthernetClient_readMulti(self as *mut EthernetClient, buf, size)
        };
        if code > 0 {
            Ok(&dest[..(code as usize)])
        } else {
            Err(SocketError::new("read returns nothing"))
        }
    }

    pub fn println(&mut self, msg: &[u8]) -> u16 {
        unsafe { raw::virtual_EthernetClient_println(self as *mut EthernetClient, msg.as_ptr()) }
    }

    pub fn flush(&mut self) {
        unsafe { raw::virtual_EthernetClient_flush(self as *mut EthernetClient) }
    }

    pub fn stop(&mut self) {
        unsafe { raw::virtual_EthernetClient_stop(self as *mut EthernetClient) }
    }

    pub fn valid(&self) -> bool {
        unsafe { raw::EthernetClient_valid(self as *const EthernetClient) }
    }

    pub fn remote_ip(&self) -> IPAddress {
        unsafe { raw::virtual_EthernetClient_remoteIP(self as *const EthernetClient) }
    }

    pub fn as_client_pointer(&self) -> *const Client {
        unsafe {
            // too lazy to create a second method for const
            raw::cast_to_Client(self as *const EthernetClient as *mut EthernetClient)
                as *const Client
        }
    }

    pub fn as_client_mut_pointer(&mut self) -> *mut Client {
        unsafe { raw::cast_to_Client(self as *mut EthernetClient) }
    }
}

pub struct SocketError {
    pub msg: &'static str,
}
impl SocketError {
    pub fn new(msg: &'static str) -> SocketError {
        SocketError { msg }
    }
}

impl uWrite for EthernetClient {
    type Error = SocketError;

    fn write_str(&mut self, s: &str) -> Result<(), Self::Error> {
        let buffer = s.as_bytes();
        self.write(buffer)
    }
}
