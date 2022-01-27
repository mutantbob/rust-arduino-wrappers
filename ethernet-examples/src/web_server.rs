#![no_std]
#![no_main]

use arduino_hal::hal::port::PD6;
use arduino_hal::port::mode::Output;
use arduino_hal::port::Pin;
use arduino_hal::{default_serial, pins};
use ethernet::raw::{EthernetClient, EthernetServer};
use ethernet::{ip_address_4, new_udp};
use neopixel::{
    color_hsv, NeoPixelColorOrder, NeoPixelFrequency, NeoPixelWrapper, NeoPixelWrapperBuilder,
};
use panic_halt as _;
use rust_arduino_runtime::arduino_main_init;

fn debug_udp() {
    let mut udp = new_udp(4200);

    udp.send_to(ip_address_4(192, 168, 8, 107), 3500, &mut [4, 2, 0]);
}

enum DebugSignal {
    PreSetup,
    PostSetup,
    NoHardware,
    HardwareFound,
    BeforeAccept,
    AcceptedSomething,
    WroteORLY,
    ReadSomething(Option<u8>),
    Disconnected,
}

struct DebugGob {
    neopixels: NeoPixelWrapper<PD6>,
}

static BLACK: [u8; 3] = [0, 0, 0];
static RED: [u8; 3] = [255, 0, 0];
static YELLOW: [u8; 3] = [255, 255, 0];
static GREEN: [u8; 3] = [0, 255, 0];
static GREY: [u8; 3] = [80, 80, 80];
static BLUE: [u8; 3] = [0, 0, 255];

impl DebugGob {
    fn new(p6: Pin<Output, PD6>) -> Self {
        DebugGob {
            neopixels: NeoPixelWrapperBuilder::new(
                4,
                p6,
                NeoPixelColorOrder::NEO_RGB,
                NeoPixelFrequency::NEO_KHZ800,
            )
            .begin(),
        }
    }

    fn signal(&mut self, signal: DebugSignal) {
        let colors: [[u8; 3]; 4] = match signal {
            DebugSignal::PreSetup => [RED, BLACK, BLACK, BLACK],
            DebugSignal::PostSetup => [YELLOW, BLACK, BLACK, BLACK],
            DebugSignal::NoHardware => [RED, RED, RED, RED],
            DebugSignal::HardwareFound => [GREEN, BLACK, BLACK, BLACK],
            DebugSignal::BeforeAccept => [GREEN, GREY, BLACK, BLACK],
            DebugSignal::AcceptedSomething => [GREEN, BLUE, BLACK, BLACK],
            DebugSignal::WroteORLY => [GREEN, GREEN, BLACK, BLACK],
            DebugSignal::Disconnected => [GREEN, RED, BLACK, BLACK],
            DebugSignal::ReadSomething(maybe_byte) => match maybe_byte {
                None => [GREEN, BLACK, BLACK, RED],
                Some(byte) => {
                    let packed = color_hsv(((byte + 128) as u16) << 8, 0xff, 0xff);
                    [
                        GREEN,
                        BLACK,
                        GREEN,
                        [(packed >> 16) as u8, (packed >> 8) as u8, packed as u8],
                    ]
                }
            },
        };

        self.signal_raw(&colors)
    }

    fn signal_raw(&mut self, colors: &[[u8; 3]; 4]) {
        for (idx, rgb) in colors.iter().enumerate() {
            self.neopixels
                .set_pixel_color_rgb(idx as u16, rgb[0], rgb[1], rgb[2])
        }
        self.neopixels.show()
    }
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

    let p6 = pins.d6.into_output();

    let mut debug_gob = DebugGob::new(p6);
    debug_gob.signal(DebugSignal::PreSetup);

    //

    let mut mac = [0xde, 0xad, 0xbe, 0xef, 1, 2];
    ethernet::begin1(&mut mac, ip_address_4(192, 168, 8, 167));

    debug_gob.signal(DebugSignal::PreSetup);

    let server = if unsafe { ethernet::raw::EthernetClass_hardwareStatus() }
        == ethernet::raw::EthernetHardwareStatus_EthernetNoHardware
    {
        debug_gob.signal(DebugSignal::NoHardware);
        // oof
        None
    } else {
        debug_udp();
        let mut ethernet_server = EthernetServer::new(80);
        ethernet_server.begin();
        debug_gob.signal(DebugSignal::HardwareFound);
        Some(ethernet_server)
    };

    loop {
        if let Some(mut server) = &server {
            debug_gob.signal(DebugSignal::BeforeAccept);
            let client: Option<EthernetClient> = server.available_safe();
            if let Some(mut client) = client {
                debug_gob.signal(DebugSignal::AcceptedSomething);
                let mut current_line_is_blank = false;
                loop {
                    if client.available_for_write() > 0 {
                        client.println(b"orly\0");
                        break;
                    }
                }
                debug_gob.signal(DebugSignal::WroteORLY);
                if !client.connected() {
                    debug_gob.signal_raw(&[GREEN, BLACK, YELLOW, BLACK])
                }

                {
                    let count = client.available();
                    ufmt::uwriteln!(&mut serial, "{} bytes available", count);
                }

                while client.connected() {
                    if client.available() > 0 {
                        debug_gob.signal_raw(&[GREEN, GREEN, BLACK, BLACK]);
                        let c = client.read();
                        // Serial.write(c);
                        debug_gob.signal(DebugSignal::ReadSomething(c));
                        match c {
                            None => break,
                            Some(c) => {
                                ufmt::uwriteln!(&mut serial, "{} byte", c);
                                // if you've gotten to the end of the line (received a newline
                                // character) and the line is blank, the HTTP request has ended,
                                // so you can send a reply
                                if c == b'\n' && current_line_is_blank {
                                    // send a standard HTTP response header
                                    client.println(b"HTTP/1.1 200 OK\0");
                                    client.println(b"Content-Type: text/html\0");
                                    client.println(b"Connection: close\0"); // the connection will be closed after completion of the response
                                    client.println(b"Refresh: 5\0"); // refresh the page automatically every 5 sec
                                    client.println(b"\0");
                                    client.println(b"<!DOCTYPE HTML>\0");
                                    client.println(b"<html>\0");

                                    client.println(b"</html>\0");
                                    break;
                                }

                                if c == b'\n' {
                                    current_line_is_blank = true;
                                } else if c != b'\r' {
                                    current_line_is_blank = false;
                                }
                            }
                        }
                    } else {
                        debug_gob.signal_raw(&[GREEN, YELLOW, BLACK, BLACK]);
                    }

                    //delay_ms(300);
                }
                ufmt::uwriteln!(&mut serial, "finished responding, closing");
                //delay_ms(1000);
                client.stop();
                debug_gob.signal(DebugSignal::Disconnected);
            }
        }
    }
}
