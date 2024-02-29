#![no_main]
#![no_std]
#![deny(warnings)]
#![feature(type_alias_impl_trait)]

use heapless::Vec;
use rtic_examples as _; // global logger + panicking-behavior
use rtic_monotonics::{systick::Systick, Monotonic};
use stm32f4xx_hal::{
    gpio::{gpioa::PA5, Output, PushPull},
    prelude::*,
};

#[rtic::app(device = stm32f4xx_hal::pac, dispatchers = [USART1, USART2, USART6])]
mod app {
    use super::*;
    use defmt::info;

    // Holds the shared resources (used by multiple tasks)
    // Needed even if we don't use it
    #[shared]
    struct Shared {
        buffer: Vec<u8, 6>,
    }

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

        // Create a buffer
        let buffer = Vec::new();

        defmt::info!("Init done!");
        blink::spawn().ok();
        producer::spawn().ok();
        (Shared { buffer }, Local { led })
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

    // Producer task that pushes a value to the buffer
    #[task(priority = 2, shared = [buffer])]
    async fn producer(ctx: producer::Context) {
        Systick::delay(1.secs().into()).await;

        // Access the shared resources
        let mut buffer = ctx.shared.buffer;

        match buffer.lock(|b| b.push(1)) {
            Ok(()) => {
                defmt::info!("PRODUCER: Pushed 1");
            }
            Err(_) => {
                defmt::info!("PRODUCER: Buffer full");
            }
        };

        consumer::spawn().ok();
    }

    // Consumer task that pops all values from the buffer
    #[task(priority = 3, shared = [buffer])]
    async fn consumer(ctx: consumer::Context) {
        defmt::info!("Consumer");
        Systick::delay(1.secs().into()).await;

        // Access the shared resources
        let mut buffer = ctx.shared.buffer;

        buffer.lock(|b| {
            if b.is_full() {
                defmt::info!("CONSUMER: Buffer full, popping all values");
                while let Some(value) = b.pop() {
                    defmt::info!("CONSUMER: Popped {}", value);
                }
            } else {
                defmt::info!("CONSUMER: Buffer is: {}", b.as_slice());
            }
        });

        producer::spawn().ok();
    }
}
