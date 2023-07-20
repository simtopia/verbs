use csv::Writer;
use ethers_core::types::{Address, Bytes, U256};
use revm::primitives::Address as RevmAddress;
use revm::primitives::U256 as RevmU256;
use std::fs::File;

pub fn csv_writer<T: ToString>(records: &Vec<Vec<T>>, output_path: &str) {
    let output_file = File::create(output_path).unwrap();

    let mut wtr = Writer::from_writer(output_file);

    for record in records {
        wtr.write_record(record.into_iter().map(|x| x.to_string()))
            .expect("Could not write record");
    }

    wtr.flush().expect("Error flushing csv");
}

pub fn address_from_hex(x: &str) -> RevmAddress {
    let address = x.strip_prefix("0x").unwrap();
    let address = hex::decode(address).expect("Decoding failed");
    RevmAddress::from_slice(address.as_slice())
}

pub fn data_bytes_from_hex(hx: &str) -> Bytes {
    let data = hex::decode(hx).expect("Decoding failed");
    Bytes::from(data)
}

pub trait Cast<Y> {
    fn cast(self) -> Y;
}

impl Cast<Address> for RevmAddress {
    fn cast(self) -> Address {
        Address::from(self.0)
    }
}

impl Cast<RevmAddress> for Address {
    fn cast(self) -> RevmAddress {
        RevmAddress::from(self.0)
    }
}

pub trait Eth {
    fn to_weth(x: u128) -> Self;
}

impl Eth for RevmU256 {
    fn to_weth(x: u128) -> Self {
        let x: u128 = x * 10u128.pow(18);
        Self::from(x)
    }
}

impl Eth for U256 {
    fn to_weth(x: u128) -> Self {
        let x: u128 = x * 10u128.pow(18);
        Self::from(x)
    }
}
