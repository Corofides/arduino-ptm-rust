#![no_std]
#![no_main]

use panic_halt as _;
use avr_device::atmega328p;

#[avr_device::entry]
fn main() -> ! {
    let mut number: i32 = 0;

    let dp = atmega328p::Peripherals::take().unwrap();
    
    dp.PORTB.ddrb.write(|w| w.pb5().set_bit());
    dp.PORTB.portb.write(|w| w.pb5().set_bit());

    loop {

        number = number.wrapping_add(1);

        dp.PORTB.portb.write(|w| w.pb5().clear_bit());

        if number < (i32::MAX / 2) {
            dp.PORTB.portb.write(|w| w.pb5().set_bit());
        }
    }
}
