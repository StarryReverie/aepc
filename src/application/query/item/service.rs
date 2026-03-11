use std::sync::Arc;

use crate::domain::item::model::Item;
use crate::domain::item::outbound::DynItemRepository;

use super::QueryAllItemsError;

#[dynosaur::dynosaur(pub DynItemQueryService = dyn(box) ItemQueryService)]
pub trait ItemQueryService: Send + Sync {
    fn query_all_items(&self)
    -> impl Future<Output = Result<Vec<Item>, QueryAllItemsError>> + Send;
}

pub struct ItemQueryServiceImpl {
    pub(super) item_repository: Arc<DynItemRepository<'static>>,
}

impl ItemQueryServiceImpl {
    pub fn new(item_repository: Arc<DynItemRepository<'static>>) -> Self {
        Self { item_repository }
    }
}

impl ItemQueryService for ItemQueryServiceImpl {
    async fn query_all_items(&self) -> Result<Vec<Item>, QueryAllItemsError> {
        self.query_all_items_impl().await
    }
}
