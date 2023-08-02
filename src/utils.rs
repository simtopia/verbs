use csv::Writer;
use ethers_core::types::{Address, Bytes, U256};
use revm::primitives::Address as RevmAddress;
use revm::primitives::U256 as RevmU256;
use std::fs::File;

/// Write a vector series of agent values to a csv file
pub fn csv_writer<T: ToString>(records: &Vec<Vec<T>>, output_path: &str) {
    let output_file = File::create(output_path).expect("Could not open csv file");

    let mut wtr = Writer::from_writer(output_file);

    for record in records {
        wtr.write_record(record.into_iter().map(|x| x.to_string()))
            .expect("Could not write record");
    }

    wtr.flush().expect("Error flushing csv");
}

/// Create a revm address from a hex string
pub fn address_from_hex(x: &str) -> RevmAddress {
    let address = x.strip_prefix("0x").expect("Addresses require '0x' prefix");
    let address = hex::decode(address).expect("Decoding hex string failed");
    RevmAddress::from_slice(address.as_slice())
}

/// Create a Bytes object from a hex string
pub fn data_bytes_from_hex(hx: &str) -> Bytes {
    let data = hex::decode(hx).expect("Decoding hex failed");
    Bytes::from(data)
}

/// Casting between revm and ethers-core types
pub trait Cast<Y> {
    fn cast(self) -> Y;
}

/// Cast etheres-core address to revm address
impl Cast<Address> for RevmAddress {
    fn cast(self) -> Address {
        Address::from(self.0)
    }
}

/// Cast revm address to ethers-core address
impl Cast<RevmAddress> for Address {
    fn cast(self) -> RevmAddress {
        RevmAddress::from(self.0)
    }
}

/// Convert values in in eth to weth
pub trait Eth {
    fn to_weth(x: u128) -> Self;
}

/// Convert revm u256 from eth to weth value
impl Eth for RevmU256 {
    fn to_weth(x: u128) -> Self {
        let x: u128 = x * 10u128.pow(18);
        Self::from(x)
    }
}

/// Convert ethers-core u256 from eth to weth value
impl Eth for U256 {
    fn to_weth(x: u128) -> Self {
        let x: u128 = x * 10u128.pow(18);
        Self::from(x)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn cast_addresses() {
        let w = RevmAddress::random();
        let x = w.cast();
        let y = x.cast();
        let z = y.cast();

        assert_eq!(w, y);
        assert_eq!(x, z);
    }
}
