mod machine;
mod machine_id;
mod machine_name;
mod power;

pub use machine::Machine;
pub use machine_id::{MachineId, NewMachineIdError};
pub use machine_name::{MachineName, NewMachineNameError};
pub use power::{NewPowerError, Power};

#[cfg(test)]
pub mod test_helper {
    use super::*;

    pub fn make_machine(id: &str, name: &str, power: f64) -> Machine {
        Machine::new(
            MachineId::new(id).unwrap(),
            MachineName::new(name).unwrap(),
            Power::new(power).unwrap(),
        )
    }
}
