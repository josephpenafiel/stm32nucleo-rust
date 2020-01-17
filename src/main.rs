#![no_std]
#![no_main]

// pick a panicking behavior
extern crate panic_halt; // you can put a breakpoint on `rust_begin_unwind` to catch panics
                         // extern crate panic_abort; // requires nightly
                         // extern crate panic_itm; // logs messages over ITM; requires ITM support
                         // extern crate panic_semihosting; // logs messages to the host stderr; requires a debugger

// use cortex_m::asm;
use cortex_m_rt::entry;
use cortex_m_semihosting::hprint;
// use stm32f3::stm32f302 as stm;
// use stm32f3::stm32f302::{interrupt, Interrupt, NVIC};
use stm::{interrupt, Interrupt, NVIC};
use stm::{usart1, USART2}; // usart2 is the register block
use stm32f3xx_hal::stm32 as stm;
use stm32f3xx_hal::{prelude::*, serial::Serial};

fn init() -> &'static mut usart1::RegisterBlock {
    let dp = stm::Peripherals::take().unwrap();
    let mut flash = dp.FLASH.constrain(); // from prelude
    let mut rcc = dp.RCC.constrain();
    let clocks = rcc
        .cfgr
        .use_hse(8.mhz())
        .sysclk(48.mhz())
        .pclk1(24.mhz())
        .pclk2(24.mhz())
        .freeze(&mut flash.acr);
    let mut gpioa = dp.GPIOA.split(&mut rcc.ahb);
    let tx = gpioa.pa2.into_af7(&mut gpioa.moder, &mut gpioa.afrl);
    let rx = gpioa.pa3.into_af7(&mut gpioa.moder, &mut gpioa.afrl);
    Serial::usart2(dp.USART2, (tx, rx), 115_200.bps(), clocks, &mut rcc.apb1);
    unsafe { &mut *(USART2::ptr() as *mut _) }
}

#[entry]
fn main() -> ! {
    // asm::nop(); // To not have main optimize to abort in release mode, remove when you add code
    let usart2 = init();
    loop {
        for c in b"habla mi perraso\n".iter() {
            while usart2.isr.read().txe().bit_is_clear() {}
            usart2.tdr.write(|w| unsafe { w.tdr().bits(u16::from(*c)) });
        }
        // your code goes here
        // NVIC::pend(Interrupt::USART2_EXTI26);
    }
}

// #[interrupt]
// fn USART2_EXTI26() {
//     hprint!("uart interrupt").unwrap();
// }

// let dp = stm::Peripherals::take().unwrap();
// let rcc = dp.RCC;
// let usart = dp.USART2;
// rcc.apb1enr.write(|w| w.usart2en().set_bit());
// usart.cr1.write(|w| w.ue().clear_bit());
// usart.cr2.write(|w| w.stop().stop1());
// usart.cr1.write(|w| w.m().bit8());
// usart.cr1.write(|w| w.pce().clear_bit());
// usart.cr1.write(|w| w.te().enabled());
// usart.cr1.write(|w| w.re().enabled());
// usart.cr3.write(|w| w.ctse().disabled());
// usart.cr3.write(|w| w.rtse().disabled());
// usart.brr.write(|w| w.brr().bits(1232u16));
