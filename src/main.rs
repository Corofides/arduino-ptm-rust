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
    pub port: u8,
    pub was_high: bool,
    pub can_change: bool,
    pub port_control: PortControl,
    pub on_click_handle: Option<fn(&PortControl)>,
    pub on_press_handle: Option<fn(&PortControl)>,
    pub on_release_handle: Option<fn(&PortControl)>,
}

pub struct PortControl {
    pub port: PORTB,
    pub exint: EXINT,
}

impl Button {
    pub fn setup(&mut self) {
        self.port_control.exint.pcicr.write(|w| {
            w.pcie().bits(self.port)
        });
        self.port_control.exint.pcmsk0.write(|w| {
            w.pcint().bits(self.port)
        });
    }
    pub fn on_interrupt(&mut self) {
        if !self.can_change {
            return;
        }

        self.can_change = false;

        if !self.was_high {
            self.was_high = true;

            if let Some(on_press) = self.on_press_handle {
                (on_press)(&self.port_control)
            }
            return;
        }

        self.was_high = false;

        if let Some(on_release) = self.on_release_handle {
            (on_release)(&self.port_control);
        }

        if let Some(on_click) = self.on_click_handle {
            (on_click)(&self.port_control);
        }

    }
    pub fn allow_change(&mut self) {
        self.can_change = true;
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

    let on_click_handle = |port_control: &PortControl| {
        port_control.port.pinb.write(|w| w.pb5().set_bit());
    };

    let on_click_handle: fn(port_control: &PortControl) -> () = on_click_handle;

    let mut button = Button {
        port: 0b001,
        was_high: false,
        can_change: true,
        port_control,
        on_press_handle: None, 
        on_release_handle: None,
        on_click_handle: Some(on_click_handle)
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
