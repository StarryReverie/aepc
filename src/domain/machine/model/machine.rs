use getset::Getters;

use super::{MachineId, MachineName, Power};

#[derive(Debug, Clone, PartialEq, Getters)]
#[getset(get = "pub")]
pub struct Machine {
    id: MachineId,
    name: MachineName,
    power: Power,
}

impl Machine {
    pub fn new(id: MachineId, name: MachineName, power: Power) -> Self {
        Self { id, name, power }
    }
}
