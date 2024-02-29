#![no_main]
#![no_std]
#![deny(warnings)]
#![feature(type_alias_impl_trait)]

use rtic_examples as _; // global logger + panicking-behavior
use rtic_monotonics::{systick::Systick, Monotonic};
use stm32f4xx_hal::{
    gpio::{gpioa::PA5, Output, PushPull},
    prelude::*,
};

#[rtic::app(device = stm32f4xx_hal::pac, dispatchers = [USART1, USART2])]
mod app {
    use super::*;
    use defmt::info;

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

        // Device specific peripherals
        let mut _device: stm32f4xx_hal::pac::Peripherals = ctx.device;
        let mut _core: cortex_m::Peripherals = ctx.core;

        rtic_examples::configure_clock!(_device, _core, 84.MHz());

        // Set up the LED. On the Nucleo-F446RE it's connected to pin PA5.
        let gpioa = _device.GPIOA.split();
        let led = gpioa.pa5.into_push_pull_output();

        defmt::info!("Init done!");
        blink::spawn().ok();
        higher_priority::spawn().ok();
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
            let t = Systick::now();
            ctx.local.led.toggle();
            defmt::info!("Blink!");
            Systick::delay_until(t + 500.millis()).await;
        }
    }

    // Higher priority tasks preempt lower priority tasks
    #[task(priority = 2)]
    async fn higher_priority(_: higher_priority::Context) {
        loop {
            Systick::delay(2.secs().into()).await;
            defmt::info!("Higher priority task");

            // simulate a long running task
            for _ in 0..2_000_000 {
                cortex_m::asm::nop();
            }

            defmt::info!("Higher priority task done");
        }
    }
}
