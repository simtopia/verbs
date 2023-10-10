use alloy_primitives::{Address, Bytes, U256};
use csv::Writer;
use revm::primitives::bitvec::macros::internal::funty::Fundamental;
use std::fs::File;

/// Write a time-series of agent values to a csv file.
///
/// # Arguments
///
/// * `records` - 2d vector of agent state records.
/// * `output_path` - Path to write the csv file to.
///
pub fn csv_writer<T: ToString>(records: &Vec<Vec<T>>, output_path: &str) {
    let output_file = File::create(output_path).expect("Could not open csv file");

    let mut wtr = Writer::from_writer(output_file);

    for record in records {
        wtr.write_record(record.iter().map(|x| x.to_string()))
            .expect("Could not write record");
    }

    wtr.flush().expect("Error flushing csv");
}

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
pub fn data_bytes_from_hex(hx: &str) -> Bytes {
    let data = hex::decode(hx).expect("Decoding hex failed");
    Bytes::from(data)
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

#[cfg(test)]
mod tests {

    use super::*;
    use assert_approx_eq::assert_approx_eq;

    #[test]
    fn scaling_values() {
        let x = U256::from(5) * U256::from(10).pow(U256::from(17));
        let y = scale_data_value(x, 18, 6);
        assert_approx_eq!(y, 0.5f64);

        let x = U256::from(1) * U256::from(10).pow(U256::from(15));
        let y = scale_data_value(x, 18, 6);
        assert_approx_eq!(y, 0.001f64);

        let x = U256::from(15) * U256::from(10).pow(U256::from(17));
        let y = scale_data_value(x, 18, 6);
        assert_approx_eq!(y, 1.5f64);

        let x = U256::from(1000000005) * U256::from(10).pow(U256::from(12));
        let y = scale_data_value(x, 18, 6);
        assert_approx_eq!(y, 1000.000005f64);
    }

    #[test]
    fn scaling_out_of_bounds() {
        // Test if value is too large we continue
        let x = U256::MAX;
        let y = scale_data_value(x, 4, 0);
        assert_approx_eq!(y, u64::MAX.as_f64());
    }

    #[test]
    fn dividing_u256() {
        let x = U256::from(10) * U256::from(10).pow(U256::from(15));
        let y = U256::from(5) * U256::from(10).pow(U256::from(15));

        let z = div_u256(x, y, 6);
        assert_approx_eq!(z, 2.0f64);

        let z = div_u256(y, x, 6);
        assert_approx_eq!(z, 0.5f64);

        let x = U256::from(10).pow(U256::from(20));
        let y = U256::from(10).pow(U256::from(16));

        let z = div_u256(x, y, 6);
        assert_approx_eq!(z, 10000.0f64);

        let z = div_u256(y, x, 6);
        assert_approx_eq!(z, 0.0001f64);
    }

    #[test]
    fn div_out_of_bounds() {
        let x = U256::from(10).pow(U256::from(30));
        let y = U256::from(1);
        let z = div_u256(x, y, 0);
        assert_approx_eq!(z, u64::MAX.as_f64())
    }
}
