use anyhow::Result as AnyhowResult;

use crate::domain::item::model::{Item, ItemId};

#[dynosaur::dynosaur(pub DynItemRepository = dyn(box) ItemRepository)]
pub trait ItemRepository: Send + Sync {
    async fn get(&self, item_id: &ItemId) -> AnyhowResult<Option<Item>>;
}
