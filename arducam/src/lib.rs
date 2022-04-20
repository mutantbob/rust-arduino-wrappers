#![no_std]

///
/// https://www.arducam.com/downloads/modules/OV5642/OV5642_camera_module_software_application_notes_1.1.pdf
///
/// https://www.arducam.com/downloads/shields/ArduCAM_Mini_5MP_Camera_Shield_Hardware_Application_Note.pdf
///
/// https://cdn.sparkfun.com/datasheets/Sensors/LightImaging/OV5640_datasheet.pdf
///
use arduino_hal::delay_ms;
use arduino_spi::{Spi, SpiOps, SpiTransaction};
use avr_hal_generic::port::mode::Output;
use avr_hal_generic::port::{Pin, PinOps};
use core::mem::MaybeUninit;
use raw::ArduCAM;
use rust_arduino_helpers::NumberedPin;

pub mod raw;

// control registers
pub const ARDUCHIP_TEST1: u8 = raw::ARDUCHIP_TEST1 as u8;

pub const ARDUCHIP_FRAMES: u8 = raw::ARDUCHIP_FRAMES as u8;
pub const ARDUCHIP_FIFO: u8 = raw::ARDUCHIP_FIFO as u8;
pub const ARDUCHIP_GPIO: u8 = raw::ARDUCHIP_GPIO as u8;
pub const OV5642_CHIPID_HIGH: u16 = raw::OV5642_CHIPID_HIGH as u16;
pub const OV5642_CHIPID_LOW: u16 = raw::OV5642_CHIPID_LOW as u16;

// values for use with various registers

// FIFO CONTROL register 4
pub const FIFO_CLEAR_MASK: u8 = raw::FIFO_CLEAR_MASK as u8;

// GPIO write register 6
pub const GPIO_PWDN_MASK: u8 = raw::GPIO_PWDN_MASK as u8;

pub struct ArduCamOV5642<P: NumberedPin + PinOps> {
    pub inner: ArduCAM,
    cs_pin: Pin<Output, P>,
}

impl<P: NumberedPin + PinOps> ArduCamOV5642<P> {
    /// In the beginning of the program you will have to initialize the I2C and SPI libraries like so:
    ///```
    /// unsafe {
    ///     arducam::raw::wire::Wire.begin();
    ///     ethernet::raw::SPIClass::begin();
    /// }
    /// ```
    pub fn new(cs_pin: Pin<Output, P>) -> ArduCamOV5642<P> {
        let model = raw::OV5642 as u8;
        ArduCamOV5642 {
            inner: unsafe { ArduCAM::new1(model, P::pin_number() as i16) },
            cs_pin,
        }
    }

    /// although I find it hard to imagine you're going to recycle the
    /// SPI CS pin since it is hard-wired to the camera in almost every gadget.
    pub fn release(self) -> Pin<Output, P> {
        self.cs_pin
    }

    // I just copied this from an example.
    // I have not yet found a data sheet that explains it,
    // and one data sheet says register 7 is read-only.
    pub fn reset_cpld(&mut self) {
        unsafe { self.inner.write_reg(7, 0x80) };
        delay_ms(100);
        unsafe { self.inner.write_reg(7, 0x00) };
        delay_ms(100);
    }

    pub fn test_register_read_write(&mut self, test_val: u8) -> u8 {
        unsafe {
            self.inner.write_reg(ARDUCHIP_TEST1, test_val);
            self.inner.read_reg(ARDUCHIP_TEST1)
        }
    }

    /// # Safety
    /// the rdSensorReg16_8() emitted by bindgen is "unsafe"
    pub unsafe fn rd_sensor_reg16_8(&mut self, register: u16) -> u8 {
        let mut val = MaybeUninit::uninit();
        self.inner.rdSensorReg16_8(register, val.as_mut_ptr());
        val.assume_init()
    }

    pub fn get_chip_id_high(&mut self) -> u8 {
        unsafe { self.rd_sensor_reg16_8(OV5642_CHIPID_HIGH) }
    }

    pub fn get_chip_id_low(&mut self) -> u8 {
        unsafe { self.rd_sensor_reg16_8(OV5642_CHIPID_LOW) }
    }

    pub fn get_chip_id(&mut self) -> u16 {
        ((self.get_chip_id_high() as u16) << 8) | (self.get_chip_id_low() as u16)
    }

    pub fn set_camera_format(&mut self, fmt: CameraFormat) {
        unsafe { self.inner.set_format(fmt as u8) }
    }

    pub fn init_cam(&mut self) {
        unsafe { self.inner.InitCAM() }
    }
    pub fn set_jpeg_size(&mut self, size: JPEGSize) {
        unsafe { self.inner.OV5642_set_JPEG_size(size as u8) }
    }

    pub fn read_fifo_length(&mut self) -> u32 {
        unsafe { self.inner.read_fifo_length() }
    }

    pub fn start_capture(&mut self) {
        unsafe { self.inner.start_capture() }
    }

    pub fn read_fifo(&mut self) -> u8 {
        unsafe { self.inner.read_fifo() }
    }

    pub fn burst_read<'a, H, SPI, SCLKPIN, MOSIPIN, MISOPIN>(
        &'a mut self,
        spi: &'a mut Spi<H, SPI, SCLKPIN, MOSIPIN, MISOPIN>,
    ) -> BurstReader<'a, H, SPI, SCLKPIN, MOSIPIN, MISOPIN, P>
    where
        SPI: SpiOps<H, SCLKPIN, MOSIPIN, MISOPIN>,
        SCLKPIN: PinOps,
        MOSIPIN: PinOps,
        MISOPIN: PinOps,
    {
        BurstReader::new(self, spi)
    }

    // register 1

    pub fn set_capture_control_register(&mut self, frame_count: u8) {
        unsafe { self.inner.write_reg(ARDUCHIP_FRAMES, frame_count) };
    }

    // register 3

    /// # Safety
    /// the write_reg() emitted by bindgen is "unsafe"
    pub unsafe fn timing_register_set_mask(&mut self, mask: TimingMask) {
        self.inner.write_reg(raw::ARDUCHIP_TIM as u8, mask as u8);
    }

    pub fn set_vsync_active_low(&mut self) {
        unsafe { self.timing_register_set_mask(TimingMask::VSyncPolarity) }
    }

    // register 4

    pub fn flush_fifo(&mut self) {
        unsafe { self.inner.flush_fifo() }
    }

    pub fn clear_fifo_write_done_flag(&mut self) {
        unsafe { self.inner.clear_fifo_flag() }
    }

    // register 6

    /// # Safety
    /// the set_bit() emitted by bindgen is "unsafe"
    pub unsafe fn gpio6_set_mask(&mut self, mask: GPIOMask) {
        self.inner.set_bit(ARDUCHIP_GPIO, mask as u8);
    }

    /// # Safety
    /// the clear_bit() emitted by bindgen is "unsafe"
    pub unsafe fn gpio6_clear_mask(&mut self, mask: GPIOMask) {
        self.inner.clear_bit(ARDUCHIP_GPIO, mask as u8);
    }

    pub fn sensor_power_down_set(&mut self) {
        unsafe { self.gpio6_set_mask(GPIOMask::SensorPowerDown) };
    }

    pub fn sensor_power_up(&mut self) {
        unsafe { self.gpio6_clear_mask(GPIOMask::SensorPowerDown) };
    }

    // register 65

    /// # Safety
    /// the get_bit() emitted by bindgen is "unsafe"
    pub unsafe fn trig_get_mask(&mut self, mask: TriggerCmd) -> bool {
        0 != self.inner.get_bit(raw::ARDUCHIP_TRIG as u8, mask as u8)
    }

    pub fn capture_complete(&mut self) -> bool {
        unsafe { self.trig_get_mask(TriggerCmd::CaptureDone) }
    }
}

//

pub struct BurstReader<'a, H, SPI, SCLKPIN, MOSIPIN, MISOPIN, CSPIN: PinOps> {
    spi: SpiTransaction<'a, H, SPI, SCLKPIN, MOSIPIN, MISOPIN, CSPIN>,
}

impl<'a, H, SPI, SCLKPIN, MOSIPIN, MISOPIN, CSPIN: PinOps + NumberedPin>
    BurstReader<'a, H, SPI, SCLKPIN, MOSIPIN, MISOPIN, CSPIN>
where
    SPI: SpiOps<H, SCLKPIN, MOSIPIN, MISOPIN>,
    SCLKPIN: PinOps,
    MOSIPIN: PinOps,
    MISOPIN: PinOps,
{
    pub fn new(
        cam: &'a mut ArduCamOV5642<CSPIN>,
        spi: &'a mut Spi<H, SPI, SCLKPIN, MOSIPIN, MISOPIN>,
    ) -> Self {
        let mut spi = spi.begin_transaction(&mut cam.cs_pin);
        spi.duplex_transfer(raw::BURST_FIFO_READ as u8); // tell the camera to start sending FIFO bytes as fast as we can read them
        BurstReader { spi }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> u8 {
        self.spi.duplex_transfer(0)
    }
}

//

pub enum CameraFormat {
    BMP = raw::BMP as isize,
    JPEG = raw::JPEG as isize,
    Raw = raw::RAW as isize,
}

pub enum JPEGSize {
    R320x240 = raw::OV5642_320x240 as isize,
    R640x480 = raw::OV5642_640x480 as isize,
    R1024x768 = raw::OV5642_1024x768 as isize,
    R1280x960 = raw::OV5642_1280x960 as isize,
    R1600x1200 = raw::OV5642_1600x1200 as isize,
    R2048x1536 = raw::OV5642_2048x1536 as isize,
    R2592x1944 = raw::OV5642_2592x1944 as isize,
    R1920x1080 = raw::OV5642_1920x1080 as isize,
}

/// for use with ARDUCHIP_TIM register
pub enum TimingMask {
    HRefPolariy = raw::HREF_LEVEL_MASK as isize,
    VSyncPolarity = raw::VSYNC_LEVEL_MASK as isize,
    /// what is this?
    LcdBken = raw::LCD_BKEN_MASK as isize,
    PClkDelay = raw::PCLK_DELAY_MASK as isize,
}

//for use with ARDUCHIP_GPIO register 6
pub enum GPIOMask {
    SensorReset = raw::GPIO_RESET_MASK as isize,
    SensorPowerDown = raw::GPIO_PWDN_MASK as isize,
    SensorPowerEnable = raw::GPIO_PWREN_MASK as isize,
}

/// for use with ARDUCHIP_TRIG register
pub enum TriggerCmd {
    VSync = raw::VSYNC_MASK as isize,
    Shutter = raw::SHUTTER_MASK as isize,
    CaptureDone = raw::CAP_DONE_MASK as isize,
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
