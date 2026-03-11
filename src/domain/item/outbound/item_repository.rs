use anyhow::Result as AnyhowResult;

use crate::domain::item::model::{Item, ItemId};

#[unimock::unimock(api = ItemRepositoryMock)]
#[dynosaur::dynosaur(pub DynItemRepository = dyn(box) ItemRepository)]
pub trait ItemRepository: Send + Sync {
    fn get(&self, item_id: &ItemId) -> impl Future<Output = AnyhowResult<Option<Item>>> + Send;
}
