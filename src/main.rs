#![no_std]
#![no_main]

#[derive(PartialEq)]
enum UsartInstance {
    USART1,
    USART2,
}

// pick a panicking behavior
extern crate panic_halt; // you can put a breakpoint on `rust_begin_unwind` to catch panics
                         // extern crate panic_abort; // requires nightly
                         // extern crate panic_itm; // logs messages over ITM; requires ITM support
                         // extern crate panic_semihosting; // logs messages to the host stderr; requires a debugger

// use cortex_m::asm;
use cortex_m_rt::entry;
use cortex_m_semihosting::hprint;
use stm::{interrupt, Interrupt, NVIC};
use stm::{usart1, GPIOB, USART2}; // usart2 is the register block
                                  // use stm32f3::stm32f302::{interrupt, Interrupt, NVIC};
use stm32f3xx_hal::stm32 as stm;
use stm32f3xx_hal::{
    prelude::*,
    serial,
    serial::{Event, Serial},
};

struct Eon {
    usart1: Option<stm::USART1>,
    usart2: Option<stm::USART2>,
    usart3: Option<stm::USART3>,
}

// static mut PERIPHS: stm::Peripherals = stm::Peripherals::take().unwrap();

impl Eon {
    fn new() -> Eon {
        let dp = stm::Peripherals::take().unwrap();
        Eon {
            usart1: Some(dp.USART1),
            usart2: Some(dp.USART2),
            usart3: Some(dp.USART3),
        }
    }

    fn take_serial1(&self) -> Option<stm::USART1> {
        Some(self.usart1.take().unwrap())
    }
}

fn init(
    fsh: stm::FLASH,
    rcc: stm::RCC,
    gpioa: stm::GPIOA,
    gpiob: stm::GPIOB,
    ser2: stm::USART2,
) -> (
    &'static mut usart1::RegisterBlock,
    (serial::Tx<USART2>, serial::Rx<USART2>),
    stm32f3xx_hal::gpio::gpiob::PB13<stm32f3xx_hal::gpio::Output<stm32f3xx_hal::gpio::PushPull>>,
) {
    let mut flash = fsh.constrain(); // from prelude
    let mut rcc = rcc.constrain();
    let clocks = rcc
        .cfgr
        .use_hse(8.mhz())
        .sysclk(48.mhz())
        .pclk1(24.mhz())
        .pclk2(24.mhz())
        .freeze(&mut flash.acr);
    let mut gpioa = gpioa.split(&mut rcc.ahb);
    let mut gpiob = gpiob.split(&mut rcc.ahb);
    let mut led = gpiob
        .pb13
        .into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper);

    led.set_low().unwrap();
    let tx = gpioa.pa2.into_af7(&mut gpioa.moder, &mut gpioa.afrl);
    let rx = gpioa.pa3.into_af7(&mut gpioa.moder, &mut gpioa.afrl);
    let mut s = Serial::usart2(ser2, (tx, rx), 115_200.bps(), clocks, &mut rcc.apb1);
    s.listen(Event::Txe);
    let (tx, rx) = s.split();
    (unsafe { &mut *(USART2::ptr() as *mut _) }, (tx, rx), led)
}

static mut CNT: u32 = 0;
#[entry]
unsafe fn main() -> ! {
    let dp = stm::Peripherals::take().unwrap();
    let rcc = dp.RCC;
    let flash = dp.FLASH;
    let gpioa = dp.GPIOA;
    let mut uart2 = dp.USART2;
    let gpiob = dp.GPIOB;

    // asm::nop(); // To not have main optimize to abort in release mode, remove when you add code
    let (usart2, (mut tx, _), led) = init(flash, rcc, gpioa, gpiob, uart2);
    // let cp = stm::CorePeripherals::take().unwrap();
    // let msg = b"perraso";
    // let len = msg.len() - 1;
    // let mut index = 0;
    loop {
        // for c in b"habla mi perraso\n".iter() {
        // while usart2.isr.read().txe().bit_is_clear() {}
        // usart2.tdr.write(|w| unsafe { w.tdr().bits(u16::from(*c)) });
        // match tx.write(u8::from(*c)) {
        //     Ok(_) => while usart2.isr.read().txe().bit_is_clear() {},
        //     Err(_) => (),
        // };
        // if index > len {
        //     index = 0;
        // }

        // match tx.write(msg[index]) {
        //     Ok(_) => index += 1,
        //     Err(_) => (),
        // };
        // tx.flush().ok();

        usart_print("holitas!!\n", &mut tx);
        // CNT += 1;
        // hprint!("{}\n", CNT).unwrap();
        // NVIC::unmask(Interrupt::USART2_EXTI26);

        // your code goes here
    }
}

fn usart_print(s: &str, tx: &mut serial::Tx<USART2>) {
    let msg = s.as_bytes();
    let len = msg.len();
    //hprint!("{} {:?}\n", len, msg).unwrap();
    let mut index = 0;
    while index < len {
        match tx.write(msg[index]) {
            Ok(_) => index += 1,
            Err(_) => (),
        };
    }
}

#[interrupt]
unsafe fn USART2_EXTI26() {
    if CNT > 1000 {
        &(*GPIOB::ptr()).odr.write(|w| w.odr13().set_bit());
        CNT = 0;
    }
    &(*USART2::ptr()).tdr.write(|w| w.tdr().bits(15 as u16));
    // &(*USART2::ptr()).rqr.write(|w| w.txfrq().set_bit());
}

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
