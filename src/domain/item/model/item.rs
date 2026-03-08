use getset::Getters;

use super::{ItemId, ItemName};

#[derive(Debug, Clone, PartialEq, Eq, Getters)]
#[getset(get = "pub")]
pub struct Item {
    id: ItemId,
    name: ItemName,
}

impl Item {
    pub fn new(id: ItemId, name: ItemName) -> Self {
        Self { id, name }
    }
}
