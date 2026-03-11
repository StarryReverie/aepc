mod machine_repository;

pub use machine_repository::{DynMachineRepository, MachineRepository};

#[cfg(test)]
pub mod test_helper {
    use super::*;

    pub use machine_repository::MachineRepositoryMock;
}
