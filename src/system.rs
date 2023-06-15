// System headers, of course, doesn't use standard Rust naming
// conventions.
#![allow(nonstandard_style)]
// Not all of the functions in the headers will be used so...
#![allow(dead_code)]

// A binding to system-specific functionalities.
#[cfg(target_family = "unix")]
pub mod termios;
#[cfg(target_family = "unix")]
pub mod unistd;

// Windows stuff.
#[cfg(target_family = "windows")]
pub mod conio;
