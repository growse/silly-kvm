use anyhow::{anyhow, Error};
use std::num::ParseIntError;
use std::time::Duration;

/**
 * Also trait
 *
 * This trait is used to chain method calls and perform side effects.
 *
 * Example:
 *
 * ```
 * let x = 5;
 * x.also(|x| println!("x is {}", x));
 * ```
 *
 * This will print "x is 5" and return 5.
 */
pub trait Also {
    fn also<F>(&self, f: F) -> &Self
    where
        F: FnOnce(&Self),
    {
        f(self);
        self
    }
}

impl<T> Also for T {}

/**
 * Parse a duration from a string
 *
 * This function is used to parse a duration from a string containing an integer millisecond values
 */
pub fn parse_duration(arg: &str) -> Result<Duration, ParseIntError> {
    let milliseconds = arg.parse()?;
    Ok(Duration::from_millis(milliseconds))
}

pub trait IntegerFromHexString<T> {
    fn from_hex_string(input: &str) -> Result<T, Error>;
}

impl IntegerFromHexString<u16> for u16 {
    fn from_hex_string(input: &str) -> Result<u16, Error> {
        return if input.starts_with("0x") {
            u16::from_str_radix(input.trim_start_matches("0x"), 16).map_err(|e| e.into())
        } else {
            Err(anyhow!("Invalid hex string"))
        };
    }
}
