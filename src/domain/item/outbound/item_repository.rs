use anyhow::Result as AnyhowResult;

use crate::domain::item::model::{Item, ItemId};

#[unimock::unimock(api = ItemRepositoryMock)]
#[dynosaur::dynosaur(pub DynItemRepository = dyn(box) ItemRepository)]
pub trait ItemRepository: Send + Sync {
    fn get(&self, item_id: &ItemId) -> impl Future<Output = AnyhowResult<Option<Item>>> + Send;

    fn find_all_by_name_containing_pattern(
        &self,
        pattern: &str,
    ) -> impl Future<Output = AnyhowResult<Vec<Item>>> + Send;
}
