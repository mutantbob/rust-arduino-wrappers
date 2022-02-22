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

pub struct PubSubClientWrapper {
    inner: raw::PubSubClient,
}

impl PubSubClientWrapper {
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
