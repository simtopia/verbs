use csv::Writer;
use ethers_core::types::Address;
use revm::primitives::Address as RevmAddress;
use revm::primitives::U256;
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

pub fn convert_address(address: RevmAddress) -> Address {
    Address::from_low_u64_le(address.to_low_u64_le())
}

pub fn inverse_convert_address(address: Address) -> RevmAddress {
    RevmAddress::from_low_u64_le(address.to_low_u64_le())
}

pub fn eth_to_weth(x: u128) -> U256 {
    let x: u128 = x * 10u128.pow(18);
    U256::from(x)
}
