#![allow(non_snake_case, non_upper_case_globals, non_camel_case_types)]
#![allow(dead_code)]
#![allow(clippy::all)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
include!(concat!(env!("OUT_DIR"), "/bindings-extra.rs"));

pub mod wire {
    include!(concat!(env!("OUT_DIR"), "/bindings_wire.rs"));
}
