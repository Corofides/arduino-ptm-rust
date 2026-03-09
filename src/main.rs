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

pub struct Button
{
    pub pin: u8,
    pub was_high: bool,
    pub can_change: bool,
    pub port_control: PortControl,
    pub on_press_handle: fn(&PortControl),
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
    pub fn on_interrupt(&mut self) {
        // Early exit if we can't change.
        if !self.can_change {
            return;
        }

        self.can_change = false;

        if !self.was_high {
            self.was_high = true;
            return;
        }

        self.was_high = false;
        self.on_press();

    }
    pub fn allow_change(&mut self) {
        self.can_change = true;
    }
    pub fn on_press(&self) {
        (self.on_press_handle)(&self.port_control);
        //self.port_control.port.pinb.write(|w| w.pb5().set_bit());
    }
}

#[avr_device::interrupt(atmega328p)]
fn TIMER0_OVF() {

    let cs = unsafe { CriticalSection::new() };

    let mut button = BUTTON.borrow(cs).borrow_mut();

    if let Some(button) = button.as_mut() {
        button.allow_change();
    }

}

#[avr_device::interrupt(atmega328p)]
fn PCINT0() {
    let cs = unsafe { CriticalSection::new() };

    let mut button = BUTTON.borrow(cs).borrow_mut();

    if let Some(button) = button.as_mut() {
        button.on_interrupt();
    }
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

    let on_press_handle = |port_control: &PortControl| {
        port_control.port.pinb.write(|w| w.pb5().set_bit());
        // Do nothing
    };

    let on_press_handle: fn(port_control: &PortControl) -> () = on_press_handle;

    let mut button = Button {
        pin: 8,
        was_high: false,
        can_change: true,
        port_control,
        on_press_handle,
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
