pub mod counter;
pub mod dial;

/// Ensure action modules (and their `inventory` registrations) are linked.
pub fn force_link() {
    let _ = std::any::type_name::<counter::CounterAction>();
    let _ = std::any::type_name::<dial::DialAction>();
}
