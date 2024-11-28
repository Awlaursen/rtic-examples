#![no_main]
#![no_std]

use defmt_rtt as _; // global logger
use fugit as _; // time units
use panic_probe as _; // panic handler
use stm32f4xx_hal as _; // memory layout // time abstractions

const SYSTICK_FREQ: u32 = 1_000;
const CLOCK_FREQ: u32 = 60_000_000;

use rtic_monotonics::{stm32::prelude::*, systick_monotonic};
systick_monotonic!(Mono, SYSTICK_FREQ);

#[rtic::app(device = stm32f4xx_hal::pac, dispatchers = [USART1, USART2, USART6])]
mod app {
    use stm32f4xx_hal::{
        gpio::{Output, PushPull, PA5},
        prelude::*,
    }
    ;
    use defmt::{debug, info};
    use heapless::Vec;
    use super::*;

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

        // Configure the clock
        let clocks = ctx.device.RCC.constrain().cfgr.sysclk(CLOCK_FREQ.Hz()).freeze();
        debug!("Clocks : {}", clocks);
        Mono::start(ctx.core.SYST, CLOCK_FREQ);

        // Set up the LED. On the Nucleo-F446RE it's connected to pin PA5.
        let gpioa = ctx.device.GPIOA.split();
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
            let t = Mono::now();
            ctx.local.led.toggle();
            defmt::info!("Blink!");
            Mono::delay_until(t + 500_u64.millis()).await;
        }
    }

    // Producer task that pushes a value to the buffer
    #[task(priority = 2, shared = [buffer])]
    async fn producer(ctx: producer::Context) {
        Mono::delay(1_u64.secs()).await;

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
        Mono::delay(1_u64.secs()).await;

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
