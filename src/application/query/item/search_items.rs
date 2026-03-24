use anyhow::Error as AnyhowError;
use snafu::prelude::*;

use crate::application::query::item::ItemQueryServiceImpl;
use crate::domain::item::model::Item;
use crate::domain::item::outbound::ItemRepository;

#[derive(Debug, Snafu)]
#[non_exhaustive]
pub enum SearchItemsError {
    #[snafu(display("infrastructure error when querying all items"))]
    Infrastructure { source: AnyhowError },
}

impl ItemQueryServiceImpl {
    pub(super) async fn search_items_impl(
        &self,
        pattern: &str,
    ) -> Result<Vec<Item>, SearchItemsError> {
        let items = self
            .item_repository
            .find_all_by_name_containing_pattern(pattern)
            .await
            .context(InfrastructureSnafu)?;
        Ok(items)
    }
}

#[cfg(test)]
mod tests {
    use unimock::*;

    use crate::domain::item::model::{Item, test_helper::make_item};
    use crate::domain::item::outbound::DynItemRepository;
    use crate::domain::item::outbound::test_helper::ItemRepositoryMock;

    use super::*;

    #[tokio::test]
    async fn test_search_items() {
        fn i1() -> Item {
            make_item("i1", "Item 1")
        }
        fn i2() -> Item {
            make_item("i2", "Item 2")
        }
        fn i3() -> Item {
            make_item("i3", "Item 3")
        }

        let item_repo = DynItemRepository::new_arc(Unimock::new(
            ItemRepositoryMock::find_all_by_name_containing_pattern.stub(|each| {
                each.call(matching!())
                    .answers(&|_, _| Ok(vec![i1(), i2(), i3()]));
            }),
        ));
        let service = ItemQueryServiceImpl::new(item_repo);

        let items = service.search_items_impl("Item").await.unwrap();

        assert_eq!(items.len(), 3);
        assert_eq!(items[0], make_item("i1", "Item 1"));
        assert_eq!(items[1], make_item("i2", "Item 2"));
        assert_eq!(items[2], make_item("i3", "Item 3"));
    }
}
