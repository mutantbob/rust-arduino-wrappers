#![no_std]
#![no_main]

use arduino_hal::{default_serial, delay_ms, pins};
use ethernet::{ip_address_4, EthernetWrapper, IPAddress};
use panic_halt as _;
use rust_arduino_runtime::arduino_main_init;
use ufmt::{uwrite, uwriteln};

fn fallback_self_ip() -> IPAddress {
    ip_address_4(192, 168, 8, 167)
}

fn fallback_dns() -> IPAddress {
    ip_address_4(192, 168, 8, 103)
}

#[arduino_hal::entry]
fn main() -> ! {
    arduino_main_init();

    let dp = arduino_hal::Peripherals::take().unwrap();

    let pins = pins!(dp);

    //let mut pins = arduino_hal::Pins::new(dp.PORTB, dp.PORTC, dp.PORTD);
    let mut serial = default_serial!(dp, pins, 115200);

    let _ = uwriteln!(&mut serial, "Arduino says hi.");

    //

    let mut mac = [0xde, 0xad, 0xbe, 0xef, 1, 2];

    let ethernet_builder = EthernetWrapper::builder(pins.d10.into_output());
    let ethernet = match 1 {
        1 => ethernet_builder.static_ip_with_dns(&mut mac, fallback_self_ip(), fallback_dns()),
        _ => {
            let ethernet = ethernet_builder.dhcp_lease(&mut mac, 60_000, 4_000);
            match ethernet {
                Err(builder) => {
                    if unsafe { ethernet::raw::EthernetClass_hardwareStatus() }
                        == ethernet::raw::EthernetHardwareStatus_EthernetNoHardware
                    {
                        let _ = uwriteln!(
                            &mut serial,
                            "Ethernet shield was not found.  Sorry, can't run without hardware. :("
                        );
                        // oof
                        spin_forever()
                    } else {
                        if let ethernet::LinkStatus::LinkOff = builder.link_status() {
                            let _ = uwriteln!(&mut serial, "Ethernet cable is not connected");
                        }
                        builder.static_ip_with_dns(&mut mac, fallback_self_ip(), fallback_dns())
                    }
                }
                Ok(wrapper) => {
                    let _ = uwriteln!(&mut serial, "DHCP assigned IP {}", wrapper.local_ip());
                    let _ = uwriteln!(&mut serial, "DNS is {}", wrapper.dns_server_ip());
                    wrapper
                }
            }
        }
    };

    delay_ms(1000);

    let server_name = "www.purplefrog.com\0"; // XXX big problem

    let _ = uwriteln!(&mut serial, "connecting to {}...", server_name);

    let client = ethernet.tcp_connect_hostname(server_name, 80);
    match client {
        Err(code) => {
            let _ = uwriteln!(&mut serial, "connection failed: {}", code);
            spin_forever();
        }
        Ok(mut client) => {
            let _ = uwriteln!(&mut serial, "connected to {}", client.remote_ip());

            let _ = uwrite!(
                &mut client,
                "GET /~thoth/art/ HTTP/1.1\r
Host: www.purplefrog.com\r
Connection: close\r
\r\n"
            );

            let mut byte_count: usize = 0;
            let mut buffer = [0u8; 80];
            while client.connected() {
                let len = client.available();
                if len > 0 {
                    match client.read_multi(&mut buffer) {
                        Ok(slice) => {
                            for byte in slice {
                                serial.write_byte(*byte);
                            }
                            serial.flush();
                            byte_count += slice.len();
                        }
                        Err(msg) => {
                            let _ = uwriteln!(&mut serial, "err {}", msg.msg);
                        }
                    }
                }
            }
            let _ = uwriteln!(&mut serial, "{} total", byte_count);

            loop {
                delay_ms(0x7fff);
                unsafe {
                    ethernet::raw::EthernetClass_maintain();
                }
            }
        }
    }
}

fn spin_forever() -> ! {
    loop {
        delay_ms(1000) // SPIN FOREVER!
    }
}
