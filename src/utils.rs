use csv::Writer;
use std::fs::File;

pub fn csv_writer<T: ToString>(records: Vec<Vec<T>>, output_path: String) {
    let output_file = File::create(output_path).unwrap();

    let mut wtr = Writer::from_writer(output_file);

    for record in records {
        wtr.write_record(record.into_iter().map(|x| x.to_string()))
            .expect("Could not write record");
    }

    wtr.flush().expect("Error flushing csv");
}
