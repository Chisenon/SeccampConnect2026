//! OLED Display Control Module
//!
//! This module handles the initialization and management of the SSD1306 OLED display.
//! It includes the setup of the I2C interface and a dedicated asynchronous task
//! for updating the display content based on keyboard state events (e.g., layer changes).
//!
//! Ref: [https://github.com/EthanOlpin/Lily58-Pro-RMK/blob/master/src/oled.rs](https://github.com/EthanOlpin/Lily58-Pro-RMK/blob/master/src/oled.rs)

use core::fmt::Write;
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text},
};
use esp_hal::{
    gpio::{InputPin, OutputPin},
    i2c::master::{Config, I2c, Instance},
    time::Rate,
    Async,
};
use rmk::channel::CONTROLLER_CHANNEL;
use rmk::heapless::String;
use ssd1306::{
    mode::{BasicMode, DisplayConfigAsync, BufferedGraphicsModeAsync},
    prelude::{DisplayRotation, I2CInterface},
    size::DisplaySize128x32,
    I2CDisplayInterface, Ssd1306Async,
};

/// The physical dimensions of the OLED display (128x32 pixels).
const DISPLAY_SIZE: DisplaySize128x32 = DisplaySize128x32;

/// Type alias for the I2C interface used by the SSD1306 driver.
type DisplayInterface<'a> = I2CInterface<I2c<'a, Async>>;

/// Type alias for the SSD1306 display instance in buffered graphics mode.
/// This is the main type used for drawing operations.
pub type OledDisplay<'a> =
    Ssd1306Async<DisplayInterface<'a>, DisplaySize128x32, BufferedGraphicsModeAsync<DisplaySize128x32>>;

/// Helper function to initialize the underlying I2C peripheral and the basic SSD1306 interface.
///
/// This function configures the I2C clock to 400kHz.
fn init_oled_base<'a>(
    i2c0: impl Instance + 'a,
    sda: impl InputPin + OutputPin + 'a,
    scl: impl InputPin + OutputPin + 'a,
    rotation: DisplayRotation,
) -> Ssd1306Async<DisplayInterface<'a>, DisplaySize128x32, BasicMode> {
    let config = Config::default().with_frequency(Rate::from_hz(400_000u32));
    let i2c = I2c::new(i2c0, config)
        .expect("failed to initialize I2C")
        .with_sda(sda)
        .with_scl(scl)
        .into_async();
    let interface = I2CDisplayInterface::new(i2c);
    Ssd1306Async::new(interface, DISPLAY_SIZE, rotation)
}

/// Initializes the OLED display in buffered graphics mode.
///
/// This function sets up the I2C connection, initializes the display driver,
/// clears the buffer, and flushes it to ensure a clean start.
///
/// # Panics
/// It panics when init or flushing display failed.
///
/// # Arguments
///
/// * `i2c0` - The I2C0 peripheral instance.
/// * `sda` - The pin to be used for SDA data line.
/// * `scl` - The pin to be used for SCL clock line.
/// * `rotation` - The rotation setting for the display (e.g., `DisplayRotation::Rotate0`).
///
/// # Returns
///
/// Returns a fully initialized `OledDisplay` instance ready for drawing commands.
pub async fn init_oled<'a>(
    i2c0: impl Instance + 'a,
    sda: impl InputPin + OutputPin + 'a,
    scl: impl InputPin + OutputPin + 'a,
    rotation: DisplayRotation,
) -> OledDisplay<'a> {
    let mut display = init_oled_base(i2c0, sda, scl, rotation).into_buffered_graphics_mode();
    display.init().await.unwrap();
    display.clear_buffer();
    display.flush().await.unwrap();
    display
}

/// Main asynchronous task for managing the OLED display.
///
/// This task runs an infinite loop that:
/// 1. Subscribes to the RMK controller channel to receive keyboard events.
/// 2. Updates the display with the current keyboard state (e.g., active layer).
/// 3. Refreshes the screen content.
///
/// # Arguments
///
/// * `display` - The initialized `OledDisplay` instance.
#[embassy_executor::task]
pub async fn oled_task(mut display: OledDisplay<'static>) {
    // 1. Define font style.
    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(BinaryColor::On)
        .build();

    // 2. Display the startup logo.
    let _ = Text::with_baseline("seccamp connect 2026", Point::new(0, 0), text_style, Baseline::Top)
        .draw(&mut display);
    let _ = display.flush().await;

    // Create subscriber for receiving event from RMK core.
    let Ok(mut sub) = CONTROLLER_CHANNEL.subscriber() else {
        return;
    };

    let mut current_layer = 0;

    loop {
        // 3. Clear and draw display content.
        display.clear_buffer();

        // Draw fixed text.
        let _ = Text::with_baseline("seccamp connect 2026", Point::new(0, 0), text_style, Baseline::Top)
            .draw(&mut display);

        // Format current layer information.
        let mut buf = String::<32>::new();
        let _ = write!(buf, "Layer: {current_layer}");

        // Draw layer information at y:16.
        let _ = Text::with_baseline(&buf, Point::new(0, 16), text_style, Baseline::Top)
            .draw(&mut display);

        // Flush buffer to the display.
        let _ = display.flush().await;

        // 4. Wait for the next event (async).
        // This suspends the task until a message arrives, saving power.
        let event = sub.next_message_pure().await;

        // Update the state according to the received event.
        #[allow(clippy::single_match)]
        match event {
            rmk::event::ControllerEvent::Layer(l) => {
                current_layer = l;
            }
            // TODO: Handle `Key`, `Modifier`, `Sleep` events if necessary.
            _ => {}
        }
    }
}
