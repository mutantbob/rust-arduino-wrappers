#![allow(non_snake_case, non_upper_case_globals, non_camel_case_types)]
#![allow(dead_code)]
#![allow(clippy::all)]

use rust_arduino_runtime::client::Client;
use rust_arduino_runtime::ip_address::IPAddress;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
