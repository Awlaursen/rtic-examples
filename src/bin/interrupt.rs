#![no_main]
#![no_std]
#![deny(warnings)]
#![feature(type_alias_impl_trait)]

use rtic::app;
use rtic_examples as _; // global logger + panicking-behavior + memory layout

#[app(device = stm32f4xx_hal::pac, dispatchers = [SPI1])]
mod app {
    use core::sync::atomic::{AtomicUsize, Ordering};
    use rtic_monotonics::{systick::Systick, Monotonic};
    use stm32f4xx_hal::{
        gpio::{Edge, Input, Output, Pin, PushPull},
        prelude::*,
    };

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
    fn init(ctx: init::Context) -> (Shared, Local) {
        defmt::info!("init");

        // Cortex-M peripherals
        let mut _core: cortex_m::Peripherals = ctx.core;

        // Device specific peripherals
        let mut _device: stm32f4xx_hal::pac::Peripherals = ctx.device;

        rtic_examples::configure_clock!(_device, _core, 180.MHz());

        // Set up the LED. On the Nucleo-F446RE it's connected to pin PA5.
        let gpioa = _device.GPIOA.split();
        let led = gpioa.pa5.into_push_pull_output();

        // Set up the button. On the Nucleo-F446RE it's connected to pin PC13.
        let gpioc = _device.GPIOC.split();
        let mut button = gpioc.pc13.into_floating_input();

        // Configure Button Pin for Interrupts
        // 1) Promote SYSCFG structure to HAL to be able to configure interrupts
        let mut sys_cfg = _device.SYSCFG.constrain();
        // 2) Make button an interrupt source
        button.make_interrupt_source(&mut sys_cfg);
        // 3) Configure the interruption to be triggered on a rising edge
        button.trigger_on_edge(&mut _device.EXTI, Edge::Falling);
        // 4) Enable gpio interrupt for button
        button.enable_interrupt(&mut _device.EXTI);

        // Get the external interrupt controller so we can check which interrupt fired later
        let exti = _device.EXTI;

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
            let t = Systick::now();
            // reset the counter and get the old value.
            let count = COUNTER.swap(0, Ordering::SeqCst);
            defmt::info!("{}", count);
            ctx.local.led.toggle();
            Systick::delay_until(t + 1.secs()).await;
        }
    }

    // This is the interrupt handler for the button, it is bound to the EXTI15_10 interrupt
    // as the the button is connected to pin PC13 and 13 is in the range 10-15.
    #[task(binds = EXTI15_10, local = [button], shared = [exti])]
    fn on_exti(ctx: on_exti::Context) {
        // Lock the mutex to get access to the EXTI peripheral
        let mut exti = ctx.shared.exti;
        let is_button = exti.lock(|exti| exti.pr.read().pr13().bit_is_set());

        // If it's not from the button, return
        if !is_button {
            defmt::info!("not button");
            ctx.local.button.clear_interrupt_pending_bit();
            return;
        }

        // Clear the interrupt pending bit as rtic does not do this automatically.
        ctx.local.button.clear_interrupt_pending_bit();
        defmt::info!("incrementing");
        COUNTER.fetch_add(1, Ordering::SeqCst);
    }
}
