#![no_std]

pub use ariel_os_log::*;

pub mod gpio {
    pub mod output {
        pub use atsamd_hal::gpio::Output;

        // type OutputPin = self::Output;
    }

    pub mod input {
        pub use atsamd_hal::gpio::Input;
    }
}

