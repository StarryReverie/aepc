use getset::{CopyGetters, Getters};

use super::{MachineId, MachineName, Power};

#[derive(Debug, Clone, PartialEq, Getters, CopyGetters)]
pub struct Machine {
    #[getset(get = "pub")]
    id: MachineId,
    #[getset(get = "pub")]
    name: MachineName,
    #[getset(get_copy = "pub")]
    power: Power,
}

impl Machine {
    pub fn new(id: MachineId, name: MachineName, power: Power) -> Self {
        Self { id, name, power }
    }
}
