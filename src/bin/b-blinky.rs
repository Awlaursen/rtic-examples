#![no_main]
#![no_std]

use defmt_rtt as _; // global logger
use fugit as _; // time units
use panic_probe as _; // panic handler
use stm32f4xx_hal as _; // memory layout // time abstractions

use rtic_monotonics::{stm32::prelude::*, systick_monotonic};

// Create a new monotonic resource called `Mono` that uses the SysTick timer
// with a frequency of 1_000 Hz
systick_monotonic!(Mono, 1_000);


#[rtic::app(device = stm32f4xx_hal::pac, dispatchers = [USART1])]
mod app {
    use stm32f4xx_hal::{
        gpio::{gpioa::PA5, Output, PushPull}, prelude::*
    };
    use defmt::info;
    use super::*;

    // Holds the shared resources (used by multiple tasks)
    // Needed even if we don't use it
    #[shared]
    struct Shared {}

    // Holds the local resources (used by a single task)
    // Needed even if we don't use it
    #[local]
    struct Local {
        led: PA5<Output<PushPull>>,
    }

    // The init function is called in the beginning of the program
    #[init]
    fn init(ctx: init::Context) -> (Shared, Local) {
        info!("init");

        // Configure the clock
        Mono::start(ctx.core.SYST, 36_000_000);

        // Set up the LED. On the Nucleo-F446RE it's connected to pin PA5.
        let gpioa = ctx.device.GPIOA.split();
        let led = gpioa.pa5.into_push_pull_output();

        defmt::info!("Init done!");
        blink::spawn().ok();
        (Shared {}, Local { led })
    }

    // The idle function is called when there is nothing else to do
    #[idle]
    fn idle(_: idle::Context) -> ! {
        loop {
            continue;
        }
    }

    // The task functions are called by the scheduler
    #[task(local = [led], priority = 1)]
    async fn blink(ctx: blink::Context) {
        loop {
            ctx.local.led.toggle();
            defmt::info!("Blink!");
            Mono::delay(1_u64.secs()).await;
        }
    }
}
