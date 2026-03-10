use anyhow::Result as AnyhowResult;

use crate::domain::machine::model::{Machine, MachineId};

#[unimock::unimock(api = MachineRepositoryMock)]
#[dynosaur::dynosaur(pub DynMachineRepository = dyn(box) MachineRepository)]
pub trait MachineRepository: Send + Sync {
    async fn get(&self, machine_id: &MachineId) -> AnyhowResult<Option<Machine>>;
}
