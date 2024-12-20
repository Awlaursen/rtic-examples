#![no_main]
#![no_std]

use defmt_rtt as _; // global logger
use fugit as _; // time units
use panic_probe as _; // panic handler
use stm32f4xx_hal as _; // memory layout // time abstractions

const SYSTICK_FREQ: u32 = 1_000;
const CLOCK_FREQ: u32 = 84_000_000;

use rtic_monotonics::{stm32::prelude::*, systick_monotonic};
systick_monotonic!(Mono, SYSTICK_FREQ);

#[rtic::app(device = stm32f4xx_hal::pac, dispatchers = [SPI1])]
mod app {
    use core::sync::atomic::{AtomicUsize, Ordering};
    use stm32f4xx_hal::{
        gpio::{Edge, Input, Output, Pin, PushPull},
        prelude::*,
    };
    use defmt::info;
    use super::*;

    // AtomicUsize is a thread-safe integer type
    static COUNTER: AtomicUsize = AtomicUsize::new(0);

    #[shared]
    struct Shared {
        exti: stm32f4xx_hal::pac::EXTI,
    }

    #[local]
    struct Local {
        led: Pin<'A', 5, Output<PushPull>>,
        button: Pin<'C', 13, Input>,
    }

    #[init]
    fn init(mut ctx: init::Context) -> (Shared, Local) {
        info!("init");

        // Configure the clock
        Mono::start(ctx.core.SYST, CLOCK_FREQ);
    

        // Set up the LED. On the Nucleo-F446RE it's connected to pin PA5.
        let gpioa = ctx.device.GPIOA.split();
        let led = gpioa.pa5.into_push_pull_output();

        // Set up the button. On the Nucleo-F446RE it's connected to pin PC13.
        let gpioc = ctx.device.GPIOC.split();
        let mut button = gpioc.pc13.into_floating_input();

        // Configure Button Pin for Interrupts
        // 1) Get the system configuration
        let mut sys_cfg = ctx.device.SYSCFG.constrain();
        // 2) Make button an interrupt source
        button.make_interrupt_source(&mut sys_cfg);
        // 3) Configure the interruption to be triggered on a falling edge
        button.trigger_on_edge(&mut ctx.device.EXTI, Edge::Falling);
        // 4) Enable gpio interrupt for button
        button.enable_interrupt(&mut ctx.device.EXTI);

        // Get the external interrupt controller so we can check which interrupt fired later
        let exti = ctx.device.EXTI;

        blink::spawn().ok();

        (Shared { exti }, Local { button, led })
    }

    // The idle function is called when there is nothing else to do
    #[idle]
    fn idle(_: idle::Context) -> ! {
        loop {
            continue;
        }
    }

    #[task(local = [led], priority = 4)]
    async fn blink(ctx: blink::Context) {
        loop {
            let t = Mono::now();
            // reset the counter and get the old value.
            let count = COUNTER.swap(0, Ordering::SeqCst);
            info!("{}", count);
            ctx.local.led.toggle();
            Mono::delay_until(t + 1_u64.secs()).await;
        }
    }

    // This is the interrupt handler for the button, it is bound to the EXTI15_10 interrupt
    // as the the button is connected to pin PC13 and 13 is in the range 10-15.
    #[task(binds = EXTI15_10, local = [button], shared = [exti])]
    fn on_exti(ctx: on_exti::Context) {
        // Lock the mutex to get access to the EXTI peripheral
        let mut exti = ctx.shared.exti;
        let is_button = exti.lock(|exti| exti.pr().read().pr13().bit_is_set());

        // If it's not from the button, return
        if !is_button {
            info!("not button");
            ctx.local.button.clear_interrupt_pending_bit();
            return;
        }

        // Clear the interrupt pending bit as rtic does not do this automatically.
        ctx.local.button.clear_interrupt_pending_bit();
        info!("incrementing");
        COUNTER.fetch_add(1, Ordering::SeqCst);
    }
}
