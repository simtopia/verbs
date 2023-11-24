use rust_sim::contract::Event;

pub type PyAddress = [u8; 20];

pub type PyLog = (PyAddress, Vec<u8>);

pub type PyEvent = (String, Vec<PyLog>, usize, usize);

pub fn convert_event(event: &Event) -> PyEvent {
    (
        event.function_name.to_string(),
        event
            .logs
            .iter()
            .map(|x| (x.address.0 .0, x.data.to_vec()))
            .collect(),
        event.step,
        event.sequence,
    )
}
