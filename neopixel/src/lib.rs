#![no_std]
#![allow(non_snake_case, non_upper_case_globals, non_camel_case_types)]

/*
use core::panic::PanicInfo;

#[panic_handler]
fn handle_panic(_arg:&PanicInfo) -> !
{
loop {}
}
 */

mod raw;

use avr_hal_generic::port::mode::Output;
use avr_hal_generic::port::Pin;
use raw::neoPixelType;
use raw::Adafruit_NeoPixel;
pub use raw::{NeoPixelColorOrder, NeoPixelFrequency};
use rust_arduino_helpers::NumberedPin;

pub struct NeoPixelWrapperBuilder<PX: NumberedPin> {
    inner: Adafruit_NeoPixel,
    pin: Pin<Output, PX>,
}

impl<PX: NumberedPin> NeoPixelWrapperBuilder<PX> {
    /// we move _pin mainly so that this object owns the pin, and to harvest the pin's type so we can figure out the pin number
    pub fn new(
        led_count: u16,
        pin: Pin<Output, PX>,
        color_order: NeoPixelColorOrder,
        frequency: NeoPixelFrequency,
    ) -> Self {
        let inner = unsafe {
            Adafruit_NeoPixel::new(
                led_count,
                <PX as NumberedPin>::pin_number(),
                color_order as neoPixelType | frequency as neoPixelType,
            )
        };

        NeoPixelWrapperBuilder { inner, pin }
    }

    pub fn begin(mut self) -> NeoPixelWrapper<PX> {
        unsafe {
            self.inner.begin();
        }
        NeoPixelWrapper {
            inner: self.inner,
            pin: self.pin,
        }
    }
}

/// You probably want to use it like so:
/// ```
/// let neo_pixel = NeoPixelWrapperBuilder::new(4, arduino_hal::pins!(dp).d6,
//         NeoPixelColorOrder::NEO_RGB,
//         NeoPixelFrequency::NEO_KHZ800,)
/// ```
pub struct NeoPixelWrapper<PX: NumberedPin> {
    inner: Adafruit_NeoPixel,
    pin: Pin<Output, PX>,
}

impl<PX: NumberedPin> NeoPixelWrapper<PX> {
    pub fn set_pixel_color_rgb(&mut self, idx: u16, r: u8, g: u8, b: u8) {
        unsafe { self.inner.setPixelColor(idx, r, g, b) }
    }

    pub fn set_pixel_color_rgbw(&mut self, idx: u16, r: u8, g: u8, b: u8, w: u8) {
        unsafe { self.inner.setPixelColor1(idx, r, g, b, w) }
    }

    pub fn set_pixel_color_packed(&mut self, idx: u16, rgb: u32) {
        unsafe { self.inner.setPixelColor2(idx, rgb) }
    }

    pub fn get_pixel_color(&self, idx: u16) -> u32 {
        unsafe { self.inner.getPixelColor(idx) }
    }

    pub fn set_brightness(&mut self, bright: u8) {
        unsafe {
            self.inner.setBrightness(bright);
        }
    }

    pub fn clear(&mut self) {
        unsafe {
            self.inner.clear();
        }
    }

    pub fn show(&mut self) {
        unsafe {
            self.inner.show();
        }
    }

    pub fn reclaim_pin(self) -> Pin<Output, PX> {
        self.pin
    }
}

pub fn color_hsv(hue: u16, saturation: u8, value: u8) -> u32 {
    unsafe { raw::Adafruit_NeoPixel::ColorHSV(hue, saturation, value) }
}

pub fn gamma32(rgb: u32) -> u32 {
    unsafe { raw::Adafruit_NeoPixel::gamma32(rgb) }
}
