use std::sync::Arc;

use crate::domain::item::model::Item;
use crate::domain::item::outbound::DynItemRepository;

use super::SearchItemsError;

#[unimock::unimock(api = ItemQueryServiceMock)]
#[dynosaur::dynosaur(pub DynItemQueryService = dyn(box) ItemQueryService)]
pub trait ItemQueryService: Send + Sync {
    fn search_items(
        &self,
        pattern: &str,
    ) -> impl Future<Output = Result<Vec<Item>, SearchItemsError>> + Send;
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
    async fn search_items(&self, pattern: &str) -> Result<Vec<Item>, SearchItemsError> {
        self.search_items_impl(pattern).await
    }
}
