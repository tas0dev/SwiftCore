#![no_std]
#![no_main]

extern crate alloc;

#[no_mangle]
pub extern "C" fn _start() {
    main();
}

#[no_mangle]
pub extern "C" fn main() {

}
