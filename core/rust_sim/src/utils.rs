use alloy_primitives::{Address, U256};
use revm::primitives::bitvec::macros::internal::funty::Fundamental;

/// Create a revm address from a hex string.
///
/// # Arguments
///
/// * `hx` - Hex-string, should be prefixed with `0x`.
///
pub fn address_from_hex(hx: &str) -> Address {
    let address = hx
        .strip_prefix("0x")
        .expect("Addresses require '0x' prefix");
    let address = hex::decode(address).expect("Decoding hex string failed");
    Address::from_slice(address.as_slice())
}

/// Create a Bytes object from a hex string.
///
/// # Arguments
///
/// * `hx` - Hex string.
///
pub fn data_bytes_from_hex(hx: &str) -> Vec<u8> {
    hex::decode(hx).expect("Decoding hex failed")
}

/// Convert values in in eth to weth
pub trait Eth {
    fn to_weth(x: u128) -> Self;
}

/// Convert revm u256 from eth to weth value
impl Eth for U256 {
    fn to_weth(x: u128) -> Self {
        let x: u128 = x * 10u128.pow(18);
        Self::from(x)
    }
}

/// Scale a U256 value representing a fixed number
/// of decimals into a value that fits a f64 (discarding some precision)
///
/// # Arguments
///
/// * `x` - U256 value
/// * `decimals` - Number of decimals represented by `x`
/// * `precision` - Desired precision of the output
///
pub fn scale_data_value(x: U256, decimals: usize, precision: usize) -> f64 {
    let s1 = U256::from(decimals - precision);
    let x = x / U256::from(10).pow(s1);
    // This will check for overflow
    //let x = x.as_u64();
    let x: u64 = x
        .clamp(U256::ZERO, U256::from(u64::MAX))
        .try_into()
        .unwrap();
    x.as_f64() / 10f64.powi(precision.as_i32())
}

/// Clamp a u256 to the u128 range and cast
///
/// # Arguments
///
/// * `x` - u256 value
///
pub fn clamp_u256_to_u128(x: U256) -> u128 {
    x.clamp(U256::ZERO, U256::from(u128::MAX))
        .try_into()
        .unwrap()
}

/// Divide two u256 values returning a f64
///
/// # Arguments
///
/// * `x` - u256 value
/// * `y` - u256 value
/// * `precision` - decimal precision of the result
///
pub fn div_u256(x: U256, y: U256, precision: i32) -> f64 {
    let z = x * U256::from(10).pow(U256::from(precision)) / y;
    let z: u64 = z
        .clamp(U256::ZERO, U256::from(u64::MAX))
        .try_into()
        .unwrap();
    z.as_f64() / 10f64.powi(precision)
}

/// Append contract bytecode and encoded arguments for use in deployment
///
/// # Arguments
///
/// * `bytecode_hex` - &str Contract bytecode hex
/// * `args` - Option<Vec<u8>> ABI encoded constructor arguments
///
pub fn constructor_data(bytecode_hex: &str, mut args: Option<Vec<u8>>) -> Vec<u8> {
    let mut bytecode: Vec<u8> = data_bytes_from_hex(bytecode_hex);
    match &mut args {
        Some(a) => bytecode.append(a),
        None => {}
    }
    bytecode
}

#[cfg(test)]
mod tests {

    use super::*;
    use assert_approx_eq::assert_approx_eq;
    use rstest::rstest;

    #[rstest]
    #[case(5, 17, 0.5)]
    #[case(1, 15, 0.001)]
    #[case(15, 17, 1.5)]
    #[case(1000000005, 12, 1000.000005)]
    fn scaling_values(#[case] a: u128, #[case] exp: u128, #[case] expected: f64) {
        let x = U256::from(a) * U256::from(10).pow(U256::from(exp));
        let y = scale_data_value(x, 18, 6);
        assert_approx_eq!(y, expected);
    }

    #[test]
    fn scaling_out_of_bounds() {
        // Test if value is too large we continue
        let x = U256::MAX;
        let y = scale_data_value(x, 4, 0);
        assert_approx_eq!(y, u64::MAX.as_f64());
    }

    #[rstest]
    #[case(10, 5, 15, 15, 2.0, 0.5)]
    #[case(1, 1, 20, 16, 10000.0, 0.0001)]
    fn dividing_u256(
        #[case] a: u128,
        #[case] b: u128,
        #[case] exp_a: u128,
        #[case] exp_b: u128,
        #[case] expected_a: f64,
        #[case] expected_b: f64,
    ) {
        let x = U256::from(a) * U256::from(10).pow(U256::from(exp_a));
        let y = U256::from(b) * U256::from(10).pow(U256::from(exp_b));

        let z = div_u256(x, y, 6);
        assert_approx_eq!(z, expected_a);

        let z = div_u256(y, x, 6);
        assert_approx_eq!(z, expected_b);
    }

    #[test]
    fn div_out_of_bounds() {
        let x = U256::from(10).pow(U256::from(30));
        let y = U256::from(1);
        let z = div_u256(x, y, 0);
        assert_approx_eq!(z, u64::MAX.as_f64())
    }
}
