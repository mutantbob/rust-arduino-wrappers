#![no_std]
#![no_main]

use arduino_hal::{default_serial, delay_ms, pins};
use ethernet::raw::{EthernetClient, EthernetServer};
use ethernet::{ip_address_4, new_udp};
use panic_halt as _;
use rust_arduino_runtime::arduino_main_init;
use ufmt::uWrite;

fn debug_udp() {
    let mut udp = new_udp(4200);

    udp.send_to(ip_address_4(192, 168, 8, 107), 3500, &mut [4, 2, 0]);
}

#[arduino_hal::entry]
fn main() -> ! {
    arduino_main_init();

    let dp = arduino_hal::Peripherals::take().unwrap();

    let pins = pins!(dp);

    //let mut pins = arduino_hal::Pins::new(dp.PORTB, dp.PORTC, dp.PORTD);
    let mut serial = default_serial!(dp, pins, 115200);

    ufmt::uwriteln!(&mut serial, "Hello from Arduino!\r").unwrap();

    if false {
        // neopixel needs the clock divisor set to something other than no_clock().  Normally the arduino init() method does that (and a lot of other stuff)
        let tc0 = dp.TC0;
        tc0.tccr0b.write(|w| w.cs0().prescale_64())
    }

    //

    let mut mac = [0xde, 0xad, 0xbe, 0xef, 1, 2];
    ethernet::begin1(&mut mac, ip_address_4(192, 168, 8, 167));

    let server = if unsafe { ethernet::raw::EthernetClass_hardwareStatus() }
        == ethernet::raw::EthernetHardwareStatus_EthernetNoHardware
    {
        let _ = ufmt::uwriteln!(&mut serial, "unable to find ethernet hardware");
        // oof
        None
    } else {
        debug_udp();
        let mut ethernet_server = EthernetServer::new(80);
        ethernet_server.begin();
        let _ = ufmt::uwriteln!(&mut serial, "server is at {}", ethernet::local_ip());
        Some(ethernet_server)
    };

    loop {
        if let Some(mut server) = &server {
            let client: Option<EthernetClient> = server.available_safe();
            if let Some(mut client) = client {
                let _ = ufmt::uwriteln!(&mut serial, "new client");

                let mut current_line_is_blank = false;
                while client.connected() {
                    if client.available() > 0 {
                        let c = client.read();
                        match c {
                            None => break,
                            Some(c) => {
                                let _ = ufmt::uwriteln!(&mut serial, "{} byte", c);
                                // if you've gotten to the end of the line (received a newline
                                // character) and the line is blank, the HTTP request has ended,
                                // so you can send a reply
                                if c == b'\n' && current_line_is_blank {
                                    // send a standard HTTP response header
                                    let _ = ufmt::uwriteln!(&mut client, "HTTP/1.1 200 OK");
                                    let _ = ufmt::uwriteln!(&mut client, "Content-Type: text/html");
                                    let _ = ufmt::uwriteln!(&mut client, "Connection: close"); // the connection will be closed after completion of the response
                                    let _ = ufmt::uwriteln!(&mut client, "Refresh: 5"); // refresh the page automatically every 5 sec
                                    let _ = ufmt::uwriteln!(&mut client, "");
                                    let _ = ufmt::uwriteln!(&mut client, "<!DOCTYPE HTML>");
                                    let _ = ufmt::uwriteln!(&mut client, "<html>");

                                    let _ = ufmt::uwriteln!(&mut client, "placeholder");
                                    let _ = ufmt::uwriteln!(&mut client, "</html>");
                                    break;
                                }

                                if c == b'\n' {
                                    current_line_is_blank = true;
                                } else if c != b'\r' {
                                    current_line_is_blank = false;
                                }
                            }
                        }
                    }
                }
                client.flush();
                client.stop();
                let _ = ufmt::uwriteln!(&mut serial, "client disconnected");
            }
        }
    }
}
