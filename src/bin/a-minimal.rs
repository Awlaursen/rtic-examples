#![no_main]
#![no_std]
#![deny(warnings)]
#![deny(unsafe_code)]

use defmt_rtt as _; // global logger
use fugit as _; // time units
use panic_probe as _; // panic handler
use stm32f4xx_hal as _; // memory layout // time abstractions

use rtic::app;

#[app(device = stm32f4xx_hal::pac)]
mod app {
    use defmt::info;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

    #[init]
    fn init(_: init::Context) -> (Shared, Local) {
        info!("Hello, world!");
        (Shared {}, Local {})
    }
}