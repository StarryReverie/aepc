use anyhow::Error as AnyhowError;
use snafu::prelude::*;

use crate::application::query::item::ItemQueryServiceImpl;
use crate::domain::item::model::Item;
use crate::domain::item::outbound::ItemRepository;

#[derive(Debug, Snafu)]
#[non_exhaustive]
pub enum QueryAllItemsError {
    #[snafu(display("infrastructure error when querying all items"))]
    Infrastructure { source: AnyhowError },
}

impl ItemQueryServiceImpl {
    pub(super) async fn query_all_items_impl(&self) -> Result<Vec<Item>, QueryAllItemsError> {
        let items = self
            .item_repository
            .get_all()
            .await
            .context(InfrastructureSnafu)?;
        Ok(items)
    }
}

#[cfg(test)]
mod tests {
    use unimock::*;

    use crate::domain::item::model::{Item, test_helper::make_item};
    use crate::domain::item::outbound::test_helper::ItemRepositoryMock;
    use crate::domain::item::outbound::DynItemRepository;

    use super::*;

    #[tokio::test]
    async fn test_query_all_items() {
        let service = setup_query_all_items();

        let items = service.query_all_items_impl().await.unwrap();

        assert_eq!(items.len(), 3);
        assert_eq!(items[0], make_item("i1", "Item 1"));
        assert_eq!(items[1], make_item("i2", "Item 2"));
        assert_eq!(items[2], make_item("i3", "Item 3"));
    }

    fn setup_query_all_items() -> ItemQueryServiceImpl {
        fn i1() -> Item {
            make_item("i1", "Item 1")
        }
        fn i2() -> Item {
            make_item("i2", "Item 2")
        }
        fn i3() -> Item {
            make_item("i3", "Item 3")
        }

        let item_repo =
            DynItemRepository::new_arc(Unimock::new(ItemRepositoryMock::get_all.stub(|each| {
                each.call(matching!())
                    .answers(&|_| Ok(vec![i1(), i2(), i3()]));
            })));

        ItemQueryServiceImpl::new(item_repo)
    }
}
