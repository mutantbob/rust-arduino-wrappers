#![no_std]

mod raw;

use cstr_core::CStr;
use ethernet::{EthernetClient, IPAddress};
use raw::PubSubClient;

pub type PubSubClientCallbackType = unsafe extern "C" fn(
    arg1: *mut rust_arduino_runtime::workaround_cty::c_char,
    arg2: *mut u8,
    arg3: rust_arduino_runtime::workaround_cty::c_uint,
);

///
/// ```
///
///extern "C" fn mqtt_message_received(topic: *mut i8, payload: *mut u8, payload_length: u16) {
///    let payload = unsafe { core::slice::from_raw_parts(payload, payload_length as usize) };
///    let topic = unsafe { CStr::from_ptr(topic as *const i8) };
///
///    with_serial(|serial| {
///        let _ = uwrite!(serial, "got {} bytes on ", payload.len(),);
///        for x in topic.to_bytes() {
///            serial.write_byte(*x);
///        }
///        serial.write_byte(b'\n');
///        for x in payload {
///            serial.write_byte(*x);
///        }
///        serial.write_byte(b'\n');
///    });
///}
///
/// fn main() ->! {
///     use ethernet::ip_address_4;
///     let mut client = ethernet.unwrap().make_client();
///     let mqtt = PubSubClientWrapper::new(ip_address_4(192, 168,8,9), 1883, Some(mqtt_message_received), &client);
///     let val = mqtt.connect(cstr!("arduino"), None, None, None, 0, false, None, true);
///     let _ = mqtt.subscribe(cstr!("/arduino/writeln"), false);
///
///     loop {
///         if let Some(payload) = make_message() {
///             mqtt.publish(cstr!("/from/arduino"), payload, false);
///         }
///         mqtt.per_loop();
///         delay_ms(10);
///     }
/// }
///```
pub struct PubSubClientWrapper {
    inner: raw::PubSubClient,
}

impl PubSubClientWrapper {
    /// create a new MQTT connection to the broker at `host:port` using the `EthernetClient` for connectivity.
    /// The default port for MQTT is 1883
    /// Any messages we receive due to subscriptions will be passed to `callback` (yes, it is ugly, because the underlying implementation is simplistic).
    pub fn new(
        host: IPAddress,
        port: u16,
        callback: ::core::option::Option<PubSubClientCallbackType>,
        client: &mut EthernetClient,
    ) -> PubSubClientWrapper {
        PubSubClientWrapper {
            inner: unsafe {
                PubSubClient::new4(host, port, callback, client.as_client_mut_pointer())
            },
        }
    }

    /// publish `message` on `topic` to broker.  If `retained`, then the broker will use this as the retained message for the topic from this MQTT node.
    pub fn publish(
        &mut self,
        topic: &CStr,
        message: &[u8],
        retained: bool,
    ) -> Result<(), &'static str> {
        if unsafe {
            self.inner.publish3(
                topic.as_ptr(),
                message.as_ptr(),
                message.len() as u16,
                retained,
            )
        } {
            Ok(())
        } else {
            Err("packet would be too long")
        }
    }

    /// connect to the broker specified in the call to `new()`
    #[allow(clippy::too_many_arguments)]
    pub fn connect(
        &mut self,
        id: &CStr,
        user: Option<&CStr>,
        pass: Option<&CStr>,
        will_topic: Option<&CStr>,
        will_qos: u8,
        will_retain: bool,
        will_message: Option<&CStr>,
        clean_session: bool,
    ) -> bool {
        let user = pointer_for(user);
        unsafe {
            self.inner.connect4(
                id.as_ptr(),
                user,
                pointer_for(pass),
                pointer_for(will_topic),
                will_qos,
                will_retain,
                pointer_for(will_message),
                clean_session,
            )
        }
    }

    /// subscribe to the `topic` at our broker.  The callback specified in `new()` will be invoked whenever the code inside `self.per_loop()` receives a publication from another node.
    pub fn subscribe(&mut self, topic: &CStr, qos: bool) -> Result<(), &'static str> {
        let qos = if qos { 1 } else { 0 };
        if unsafe { self.inner.subscribe1(topic.as_ptr(), qos) } {
            Ok(())
        } else {
            Err("failed to subscribe")
        }
    }

    /// unsubscribe from a previously-subscribed topic
    pub fn unsubscribe(&mut self, topic: &CStr) -> Result<(), &'static str> {
        if unsafe { self.inner.unsubscribe(topic.as_ptr()) } {
            Ok(())
        } else {
            Err("failed to unsubscribe")
        }
    }

    /// call this periodically to process messages from the MQTT broker
    pub fn per_loop(&mut self) -> bool {
        unsafe { self.inner.loop_() }
    }

    pub fn connected(&mut self) -> bool {
        unsafe { self.inner.connected() }
    }

    /*pub fn state(&mut self) -> MQTTState {
        unsafe { self.inner.state() }
    }*/
}

fn pointer_for(x: Option<&CStr>) -> *const i8 {
    match x {
        Some(rval) => rval.as_ptr(),
        None => core::ptr::null(),
    }
}

impl Drop for PubSubClientWrapper {
    fn drop(&mut self) {
        unsafe { self.inner.destruct() }
    }
}
