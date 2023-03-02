// System headers, of course, doesn't use standard Rust naming
// conventions.
#[allow(nonstandard_style)]

// A binding to system-specific functionalities.
#[cfg(target_family = "unix")]
pub mod termios;
#[cfg(target_family = "unix")]
pub mod unistd;
