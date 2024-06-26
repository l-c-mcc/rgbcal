#![no_std]
#![no_main]

mod knob;
mod rgb;
mod ui;
pub use knob::*;
pub use rgb::*;
pub use ui::*;

use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

use embassy_executor::Spawner;
use embassy_futures::join;
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, mutex::Mutex};
use embassy_time::Timer;
use microbit_bsp::{
    embassy_nrf::{
        bind_interrupts,
        gpio::{AnyPin, Level, Output, OutputDrive},
        saadc,
    },
    Button, Microbit,
};
use num_traits::float::FloatCore;

/// Struct for managing LED values and their frame rate.
pub struct Rgbfps {
    rgb_levels: [u32; 3],
    frame_rate: u64,
}

impl Rgbfps {
    /// Constructs Rgbfps with lowest reasonable values.
    const fn new() -> Self {
        Self {
            rgb_levels: [0; 3],
            frame_rate: 10,
        }
    }
}

pub static LED_VALUES: Mutex<ThreadModeRawMutex, Rgbfps> = Mutex::new(Rgbfps::new());
pub const LEVELS: u32 = 16;
pub const RED: usize = 0;
pub const GREEN: usize = 1;
pub const BLUE: usize = 2;

/// Read global RGB values with mutex lock.
async fn get_rgb_levels() -> [u32; 3] {
    let rgbfps = LED_VALUES.lock().await;
    rgbfps.rgb_levels
}

/// Set global RGB values with mutex lock.
async fn set_rgb_levels<F>(setter: F)
where
    F: FnOnce(&mut [u32; 3]),
{
    let mut rgbfps = LED_VALUES.lock().await;
    setter(&mut rgbfps.rgb_levels);
}

/// Read the global frame rate with mutex lock.
async fn get_frame_rate() -> u64 {
    let rbgfps = LED_VALUES.lock().await;
    rbgfps.frame_rate
}

/// Set the global frame rate with mutex lock.
async fn set_frame_rate<F>(setter: F)
where
    F: FnOnce(&mut u64),
{
    let mut rgbfps = LED_VALUES.lock().await;
    setter(&mut rgbfps.frame_rate);
}

/// Main program control flow; sets up awaits.
#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    rtt_init_print!();
    let board = Microbit::default();

    bind_interrupts!(struct Irqs {
        SAADC => saadc::InterruptHandler;
    });

    // closure for constructing led pins
    let led_pin = |p| Output::new(p, Level::Low, OutputDrive::Standard);
    let red = led_pin(AnyPin::from(board.p9));
    let green = led_pin(AnyPin::from(board.p8));
    let blue = led_pin(AnyPin::from(board.p16));
    let rgb: Rgb = Rgb::new([red, green, blue], 100);

    // set up adc needed to construct knob
    let mut saadc_config = saadc::Config::default();
    saadc_config.resolution = saadc::Resolution::_14BIT;
    let saadc = saadc::Saadc::new(
        board.saadc,
        Irqs,
        saadc_config,
        [saadc::ChannelConfig::single_ended(board.p2)],
    );
    let knob = Knob::new(saadc).await;
    let mut ui = Ui::new(knob, board.btn_a, board.btn_b);

    // joins the two systems together so that one cannot get ahead
    // in "steps" compared to the other
    join::join(rgb.run(), ui.run()).await;

    panic!("fell off end of main loop");
}
