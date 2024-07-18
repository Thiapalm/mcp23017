#![no_std]

/////// Imports

#[cfg(feature = "chipmode")]
pub mod chipmode;
#[cfg(feature = "pinmode")]
pub mod pinmode;
#[cfg(feature = "portmode")]
pub mod portmode;

mod interface;
pub mod registers;

use registers::*;

/////// Support functions

/**
 * Function that converts physical pin address connection to respective hexadecimal value
 */
#[inline]
pub fn convert_slave_address(a0: SlaveAddressing, a1: SlaveAddressing, a2: SlaveAddressing) -> u8 {
    match (a0, a1, a2) {
        (SlaveAddressing::Low, SlaveAddressing::Low, SlaveAddressing::Low) => 0x20,
        (SlaveAddressing::Low, SlaveAddressing::Low, SlaveAddressing::High) => 0x21,
        (SlaveAddressing::Low, SlaveAddressing::High, SlaveAddressing::Low) => 0x22,
        (SlaveAddressing::Low, SlaveAddressing::High, SlaveAddressing::High) => 0x23,
        (SlaveAddressing::High, SlaveAddressing::Low, SlaveAddressing::Low) => 0x24,
        (SlaveAddressing::High, SlaveAddressing::Low, SlaveAddressing::High) => 0x25,
        (SlaveAddressing::High, SlaveAddressing::High, SlaveAddressing::Low) => 0x26,
        (SlaveAddressing::High, SlaveAddressing::High, SlaveAddressing::High) => 0x27,
    }
}

/////// Tests

#[cfg(test)]
mod tests {
    use super::*;
    extern crate std;

    #[test]
    fn test_convert_slave_address() {
        assert_eq!(
            0x20,
            convert_slave_address(
                SlaveAddressing::Low,
                SlaveAddressing::Low,
                SlaveAddressing::Low
            )
        );
        assert_eq!(
            0x21,
            convert_slave_address(
                SlaveAddressing::Low,
                SlaveAddressing::Low,
                SlaveAddressing::High
            )
        );
        assert_eq!(
            0x22,
            convert_slave_address(
                SlaveAddressing::Low,
                SlaveAddressing::High,
                SlaveAddressing::Low
            )
        );
        assert_eq!(
            0x23,
            convert_slave_address(
                SlaveAddressing::Low,
                SlaveAddressing::High,
                SlaveAddressing::High
            )
        );
        assert_eq!(
            0x24,
            convert_slave_address(
                SlaveAddressing::High,
                SlaveAddressing::Low,
                SlaveAddressing::Low
            )
        );
        assert_eq!(
            0x25,
            convert_slave_address(
                SlaveAddressing::High,
                SlaveAddressing::Low,
                SlaveAddressing::High
            )
        );
        assert_eq!(
            0x26,
            convert_slave_address(
                SlaveAddressing::High,
                SlaveAddressing::High,
                SlaveAddressing::Low
            )
        );
        assert_eq!(
            0x27,
            convert_slave_address(
                SlaveAddressing::High,
                SlaveAddressing::High,
                SlaveAddressing::High
            )
        );
    }
}
