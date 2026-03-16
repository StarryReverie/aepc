use std::collections::BTreeMap;

use anyhow::Result as AnyhowResult;

use crate::domain::machine::model::{Machine, MachineId, MachineName, Power};
use crate::domain::machine::outbound::MachineRepository;

pub struct MachineConstantRepository {
    machines: BTreeMap<MachineId, Machine>,
}

impl MachineConstantRepository {
    pub fn new() -> Self {
        Self {
            machines: get_machines()
                .into_iter()
                .map(|machine| (machine.id().clone(), machine))
                .collect(),
        }
    }
}

impl MachineRepository for MachineConstantRepository {
    async fn get(&self, machine_id: &MachineId) -> AnyhowResult<Option<Machine>> {
        Ok(self.machines.get(machine_id).cloned())
    }
}

fn get_machines() -> Vec<Machine> {
    try_get_machines().expect("the machines repository should be built from valid data")
}

fn try_get_machines() -> AnyhowResult<Vec<Machine>> {
    Ok(vec![
        // Resouring
        Machine::new(
            MachineId::new("electric-mining-rig")?,
            MachineName::new("Electric Mining Rig")?,
            Power::new(5.0)?,
        ),
        Machine::new(
            MachineId::new("electric-mining-rig-mk-ii")?,
            MachineName::new("Electric Mining Rig Mk II")?,
            Power::new(10.0)?,
        ),
        Machine::new(
            MachineId::new("hydro-mining-rig")?,
            MachineName::new("Hydro Mining Rig")?,
            Power::new(0.0)?,
        ),
        Machine::new(
            MachineId::new("fluid-pump")?,
            MachineName::new("Fluid Pump")?,
            Power::new(5.0)?,
        ),
        Machine::new(
            MachineId::new("portable-originium-rig")?,
            MachineName::new("Portable Originium Rig")?,
            Power::new(0.0)?,
        ),
        // Production I
        Machine::new(
            MachineId::new("refining-unit")?,
            MachineName::new("Refining Unit")?,
            Power::new(5.0)?,
        ),
        Machine::new(
            MachineId::new("shredding-unit")?,
            MachineName::new("Shredding Unit")?,
            Power::new(5.0)?,
        ),
        Machine::new(
            MachineId::new("fitting-unit")?,
            MachineName::new("Fitting Unit")?,
            Power::new(20.0)?,
        ),
        Machine::new(
            MachineId::new("moulding-unit")?,
            MachineName::new("Moulding Unit")?,
            Power::new(10.0)?,
        ),
        Machine::new(
            MachineId::new("seed-picking-unit")?,
            MachineName::new("Seed-Picking Unit")?,
            Power::new(10.0)?,
        ),
        Machine::new(
            MachineId::new("planting-unit")?,
            MachineName::new("Planting Unit")?,
            Power::new(20.0)?,
        ),
        Machine::new(
            MachineId::new("water-treatment-unit")?,
            MachineName::new("Water Treatment Unit")?,
            Power::new(50.0)?,
        ),
        // Production II
        Machine::new(
            MachineId::new("gearing-unit")?,
            MachineName::new("Gearing Unit")?,
            Power::new(10.0)?,
        ),
        Machine::new(
            MachineId::new("filling-unit")?,
            MachineName::new("Filling Unit")?,
            Power::new(20.0)?,
        ),
        Machine::new(
            MachineId::new("packaging-unit")?,
            MachineName::new("Packaging Unit")?,
            Power::new(20.0)?,
        ),
        Machine::new(
            MachineId::new("grinding-crucible")?,
            MachineName::new("Grinding Crucible")?,
            Power::new(50.0)?,
        ),
        Machine::new(
            MachineId::new("reactor-crucible")?,
            MachineName::new("Reactor Crucible")?,
            Power::new(50.0)?,
        ),
        Machine::new(
            MachineId::new("forge-of-the-sky")?,
            MachineName::new("Forge of the Sky")?,
            Power::new(50.0)?,
        ),
        Machine::new(
            MachineId::new("separating-unit")?,
            MachineName::new("Separating Unit")?,
            Power::new(20.0)?,
        ),
    ])
}
