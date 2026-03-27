//! Macro utilities for pin configuration

/// initializies pin config according to each pin settings.
macro_rules! config_matrix_pins_esp {
    (peripherals: $p:ident, input: [$($in_pin:ident), *], output: [$($out_pin:ident), +]) => {
        {
            let mut output_pins = [$(Output::new($p.$out_pin, Level::Low, OutputConfig::default())), +];
            let input_pins = [$(Input::new($p.$in_pin, InputConfig::default().with_pull(Pull::Down))), +];
            for p in output_pins.iter_mut() { {
                 let _ = p.set_low();
             } }
            (input_pins, output_pins)
        }
    };
}
