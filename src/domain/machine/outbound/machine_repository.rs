use anyhow::Result as AnyhowResult;

use crate::domain::machine::model::{Machine, MachineId};

#[unimock::unimock(api = MachineRepositoryMock)]
#[dynosaur::dynosaur(pub DynMachineRepository = dyn(box) MachineRepository)]
pub trait MachineRepository: Send + Sync {
    fn get(
        &self,
        machine_id: &MachineId,
    ) -> impl Future<Output = AnyhowResult<Option<Machine>>> + Send;
}
