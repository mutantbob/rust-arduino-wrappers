#![no_std]

pub mod raw;

use crate::EthernetInitializationMalfunction::{DhcpFailed, MissingHardware};
use avr_hal_generic::port::mode::Output;
use avr_hal_generic::port::Pin;
use core::convert::TryInto;
use embedded_nal::{nb, IpAddr, SocketAddr};
pub use raw::{Client, EthernetClient, EthernetServer, EthernetUDP, IPAddress};
use rust_arduino_helpers::NumberedPin;
use ufmt::{uDisplay, uWrite, Formatter};

#[cfg(not(feature = "board-selected"))]
compile_error!(
    "This crate requires you to specify your target Arduino board as a feature.

    Please select one of the following

    * atmega2560
    * atmega328p
    "
);

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

pub fn ip_address_4(a: u8, b: u8, c: u8, d: u8) -> IPAddress {
    unsafe { IPAddress::new1(a, b, c, d) }
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
        ip: IPAddress,
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

    pub fn new_udp(&self, port: u16) -> Option<EthernetUDP> {
        let mut rval = unsafe { raw::fabricate_EthernetUDP() };
        if rval.begin(port) != 0 {
            Some(rval)
        } else {
            None
        }
    }

    pub fn local_ip(&self) -> IPAddress {
        unsafe { raw::EthernetClass_localIP() }
    }

    pub fn dns_server_ip(&self) -> IPAddress {
        unsafe { raw::EthernetClass__dnsServerAddress } // stupid inline method
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

impl<P: NumberedPin> embedded_nal::TcpClientStack for EthernetWrapper<P> {
    type TcpSocket = EthernetClient;
    type Error = SocketError;

    fn socket(&mut self) -> Result<Self::TcpSocket, Self::Error> {
        Ok(self.make_client())
    }

    fn connect(
        &mut self,
        socket: &mut Self::TcpSocket,
        remote: SocketAddr,
    ) -> nb::Result<(), Self::Error> {
        let ip = remote.ip();
        let port = remote.port();
        let ip = match ip {
            IpAddr::V4(ipv4) => {
                let octets = ipv4.octets();
                unsafe { IPAddress::new1(octets[0], octets[1], octets[2], octets[3]) }
            }
            _ => {
                return Err(nb::Error::Other(SocketError::new(
                    "only IPv4 is supported by this module",
                )))
            }
        };
        socket.connect(ip, port).map_err(nb::Error::Other)
    }

    fn is_connected(&mut self, socket: &Self::TcpSocket) -> Result<bool, Self::Error> {
        Ok(socket.connected())
    }

    fn send(
        &mut self,
        socket: &mut Self::TcpSocket,
        buffer: &[u8],
    ) -> nb::Result<usize, Self::Error> {
        socket.write(buffer).map_err(nb::Error::Other)
    }

    fn receive(
        &mut self,
        socket: &mut Self::TcpSocket,
        buffer: &mut [u8],
    ) -> nb::Result<usize, Self::Error> {
        socket
            .read_multi(buffer)
            .map(|slice| slice.len())
            .map_err(nb::Error::Other)
    }

    fn close(&mut self, mut socket: Self::TcpSocket) -> Result<(), Self::Error> {
        socket.stop();
        Ok(())
    }
}

impl uDisplay for IPAddress {
    fn fmt<W>(&self, formatter: &mut Formatter<W>) -> Result<(), W::Error>
    where
        W: uWrite + ?Sized,
    {
        let x = unsafe { &self._address.bytes };
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
    pub fn begin(&mut self, port: u16) -> u8 {
        let this = self as *mut Self as *mut cty::c_void;
        unsafe { raw::EthernetUDP_begin(this, port) }
    }

    pub fn send_to(
        &mut self,
        destination_ip: IPAddress,
        destination_port: u16,
        payload: &[u8],
    ) -> raw::size_t {
        // use cty::*;
        if self.begin_packet(destination_ip, destination_port) {
            unsafe {
                let this = self as *mut Self as *mut cty::c_void;
                let packet_len: u16 = payload.len().try_into().unwrap();
                let n2 = raw::EthernetUDP_write1(this, payload.as_ptr(), packet_len);
                if 0 != raw::EthernetUDP_endPacket(this) {
                    n2
                } else {
                    // XXX some kind of error
                    0
                }
            }
        } else {
            // XXX some kind of error
            0
        }
    }

    pub fn parse_packet(&mut self) -> Option<UdpPacketTransaction> {
        let this = self as *mut Self as *mut cty::c_void;
        let packet_size = unsafe { raw::EthernetUDP_parsePacket(this) };
        if packet_size > 0 {
            Some(UdpPacketTransaction::new(self))
        } else {
            None
        }
    }

    pub fn remote_ip(&mut self) -> IPAddress {
        let this = self as *mut Self as *mut cty::c_void;
        unsafe { raw::EthernetClient_remoteIP(this) }
    }

    pub fn remote_port(&mut self) -> u16 {
        let this = self as *mut Self as *mut cty::c_void;
        unsafe { raw::EthernetClient_remotePort(this) }
    }

    fn begin_packet(&mut self, addr: IPAddress, port: u16) -> bool {
        let this = self as *mut Self as *mut cty::c_void;
        0 != unsafe { raw::EthernetUDP_beginPacket(this, addr, port) }
    }

    fn stop(&mut self) {
        let this = self as *mut Self as *mut cty::c_void;
        unsafe { raw::EthernetUDP_stop(this) }
    }
}

impl Drop for EthernetUDP {
    fn drop(&mut self) {
        self.stop()
    }
}

//

pub struct UdpPacketTransaction<'a> {
    socket: &'a mut EthernetUDP,
}

impl<'a> UdpPacketTransaction<'a> {
    pub fn new(socket: &'a mut EthernetUDP) -> Self {
        UdpPacketTransaction { socket }
    }

    pub fn read<'b>(&mut self, buffer: &'b mut [u8]) -> Option<&'b [u8]> {
        let this = self.socket as *mut EthernetUDP as *mut cty::c_void;
        let buffer_ptr = buffer.as_mut_ptr();
        let len = buffer.len().min(u16::MAX as usize) as u16;
        let n = unsafe { raw::EthernetUDP_read1(this, buffer_ptr, len) };
        if n > 0 {
            Some(&buffer[0..n as usize])
        } else {
            None
        }
    }
}

impl<'a> Drop for UdpPacketTransaction<'a> {
    fn drop(&mut self) {
        let this = self as *mut Self as *mut cty::c_void;
        unsafe {
            raw::EthernetUDP_flush(this);
        }
    }
}

//

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

    pub fn connect(&mut self, ip: IPAddress, port: u16) -> Result<(), SocketError> {
        let code = unsafe {
            raw::EthernetClient_connect(self as *mut EthernetClient as *mut cty::c_void, ip, port)
        };
        if code != 0 {
            Ok(())
        } else {
            Err(SocketError::new("failed to connect"))
        }
    }

    pub fn available_for_write(&mut self) -> i16 {
        unsafe { raw::virtual_EthernetClient_availableForWrite(self as *mut EthernetClient) }
    }

    pub fn connected(&self) -> bool {
        unsafe {
            raw::virtual_EthernetClient_connected(
                self as *const EthernetClient as *mut EthernetClient,
            )
        }
    }

    pub fn available(&mut self) -> i16 {
        unsafe { raw::virtual_EthernetClient_available(self as *mut EthernetClient) }
    }

    pub fn write_byte(&mut self, val: u8) -> Result<(), SocketError> {
        let n = unsafe {
            raw::virtual_EthernetClient_write(self as *mut EthernetClient, &val as *const u8, 1)
        };
        if n == 0 {
            // what is the error signaling method?  The base method returns a size_t which is unsigned
            Err(SocketError::new("failed to write to socket"))
        } else {
            Ok(())
        }
    }

    pub fn write(&mut self, buffer: &[u8]) -> Result<cty::size_t, SocketError> {
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
            Ok(n.into())
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

impl Drop for EthernetClient {
    fn drop(&mut self) {
        self.stop();
    }
}

#[derive(Debug)]
pub struct SocketError {
    pub msg: &'static str,
}

impl SocketError {
    pub fn new(msg: &'static str) -> SocketError {
        SocketError { msg }
    }
}

impl ufmt::uDebug for SocketError {
    fn fmt<W>(&self, msg: &mut Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: uWrite + ?Sized,
    {
        msg.write_str(self.msg)
    }
}

impl uWrite for EthernetClient {
    type Error = SocketError;

    fn write_str(&mut self, s: &str) -> Result<(), Self::Error> {
        let buffer = s.as_bytes();
        self.write(buffer)?;
        Ok(())
    }
}
