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

#[rtic::app(device = stm32f4xx_hal::pac, dispatchers = [USART1, USART2])]
mod app {
    use stm32f4xx_hal::
        gpio::{Output, PushPull, PA5}
    ;
    use defmt::{debug, info};
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
        let rcc = ctx.device.RCC.constrain();
        let clocks = rcc.cfgr.sysclk(CLOCK_FREQ.Hz()).freeze();
        debug!("Clocks : {}", clocks);
        Mono::start(ctx.core.SYST, CLOCK_FREQ);

        // Set up the LED. On the Nucleo-F446RE it's connected to pin PA5.
        let gpioa = ctx.device.GPIOA.split();
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
            let t = Mono::now();
            ctx.local.led.toggle();
            defmt::info!("Blink!");
            Mono::delay_until(t + 500_u64.millis()).await;
        }
    }

    // Higher priority tasks preempt lower priority tasks
    #[task(priority = 2)]
    async fn higher_priority(_: higher_priority::Context) {
        loop {
            Mono::delay(2_u64.secs().into()).await;
            defmt::info!("Higher priority task");

            // simulate a long running task
            for _ in 0..2_000_000 {
                cortex_m::asm::nop();
            }

            defmt::info!("Higher priority task done");
        }
    }
}
