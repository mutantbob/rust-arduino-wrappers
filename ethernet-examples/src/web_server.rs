#![no_std]
#![no_main]

use arduino_hal::{default_serial, delay_ms, pins, Adc};
use ethernet::raw::EthernetClient;
use ethernet::{ip_address_4, EthernetWrapper};
use panic_halt as _;
use rust_arduino_runtime::arduino_main_init;
use ufmt::uWrite;

pub fn report_analog_pin<W>(val: u16, pin_number: u8, stream: &mut W) -> Result<(), W::Error>
where
    W: uWrite,
{
    ufmt::uwrite!(stream, "analog input {} os {}<br />\n", pin_number, val)
}

#[arduino_hal::entry]
fn main() -> ! {
    arduino_main_init();

    let dp = arduino_hal::Peripherals::take().unwrap();

    let pins = pins!(dp);

    //let mut pins = arduino_hal::Pins::new(dp.PORTB, dp.PORTC, dp.PORTD);
    let mut serial = default_serial!(dp, pins, 115200);

    let mut adc = Adc::new(dp.ADC, Default::default());

    let a0 = pins.a0.into_analog_input(&mut adc);
    let a1 = pins.a1.into_analog_input(&mut adc);
    let a2 = pins.a2.into_analog_input(&mut adc);
    let a3 = pins.a3.into_analog_input(&mut adc);
    let a4 = pins.a4.into_analog_input(&mut adc);
    let a5 = pins.a5.into_analog_input(&mut adc);

    ufmt::uwriteln!(&mut serial, "Hello from Arduino!\r").unwrap();

    if false {
        // neopixel needs the clock divisor set to something other than no_clock().  Normally the arduino init() method does that (and a lot of other stuff)
        let tc0 = dp.TC0;
        tc0.tccr0b.write(|w| w.cs0().prescale_64())
    }

    //

    let mut mac = [0xde, 0xad, 0xbe, 0xef, 1, 2];
    let ethernet = match EthernetWrapper::builder(pins.d10.into_output())
        .static_ip(&mut mac, ip_address_4(192, 168, 8, 167))
    {
        Ok(ethernet) => ethernet,
        Err(malfunction) => {
            let _ = ufmt::uwriteln!(&mut serial, "{:?}; spin forever", malfunction);
            loop {
                delay_ms(0x7fff);
            }
        }
    };
    // XXX I really need to figure out the shape of the API I will wrap around hardwareStatus()

    let mut server = ethernet.tcp_listen(80);
    let _ = ufmt::uwriteln!(&mut serial, "server is at {}", ethernet.local_ip());

    loop {
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
                            //let _ = ufmt::uwriteln!(&mut serial, "{} byte", c);

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

                                let _ = report_analog_pin(a0.analog_read(&mut adc), 0, &mut client);
                                let _ = report_analog_pin(a1.analog_read(&mut adc), 1, &mut client);
                                let _ = report_analog_pin(a2.analog_read(&mut adc), 2, &mut client);
                                let _ = report_analog_pin(a3.analog_read(&mut adc), 3, &mut client);
                                let _ = report_analog_pin(a4.analog_read(&mut adc), 4, &mut client);
                                let _ = report_analog_pin(a5.analog_read(&mut adc), 5, &mut client);

                                //let _ = ufmt::uwriteln!(&mut client, "placeholder");
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
