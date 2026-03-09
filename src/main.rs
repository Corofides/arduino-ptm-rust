#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

use panic_halt as _;
use core::cell::RefCell;
use avr_device::{
    atmega328p::{self, PORTB, EXINT},
    interrupt::{CriticalSection, Mutex},
};

static BUTTON: Mutex<RefCell<Option<Button>>> = Mutex::new(RefCell::new(None));

pub struct Button {
    pub pin: u8,
    pub was_high: bool,
    pub can_change: bool,
    pub port_control: PortControl,
}

pub struct PortControl {
    pub port: PORTB,
    pub exint: EXINT,
}

impl Button {
    pub fn setup(&mut self) {
        self.port_control.exint.pcicr.write(|w| {
            w.pcie().bits(0b001)
        });
        self.port_control.exint.pcmsk0.write(|w| {
            w.pcint().bits(0b001)
        });
    }
    pub fn allow_change(&mut self) {
        self.can_change = true;
    }
}

#[avr_device::interrupt(atmega328p)]
fn TIMER0_OVF() {

    let cs = unsafe { CriticalSection::new() };

    let mut button = BUTTON.borrow(cs).borrow_mut();

    if let Some(mut button) = button.as_mut() {
        button.allow_change();
    }

}

#[avr_device::interrupt(atmega328p)]
fn PCINT0() {
}

#[avr_device::entry]
fn main() -> ! {

    let dp = atmega328p::Peripherals::take().unwrap();

    dp.TC0.tccr0b.write(|w| {
        w.cs0().prescale_1024()
    });

    dp.TC0.timsk0.write(|w| {
        w.toie0().set_bit()
    });
    
    dp.PORTB.ddrb.write(|w| w.pb5().set_bit());
    dp.PORTB.portb.write(|w| w.pb5().set_bit());

    let port_control = PortControl {
        port: dp.PORTB,
        exint: dp.EXINT,
    };

    let mut button = Button {
        pin: 8,
        was_high: false,
        can_change: true,
        port_control,
    };

    button.setup();

    avr_device::interrupt::free(|cs| {
        BUTTON.borrow(cs).replace(Some(button));
    });

    unsafe {
        avr_device::interrupt::enable();
    }

    loop { /* Do Nothing */ }
}
