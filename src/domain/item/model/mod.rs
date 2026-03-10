mod item;
mod item_id;
mod item_name;

pub use item::Item;
pub use item_id::{ItemId, NewItemIdError};
pub use item_name::{ItemName, NewItemNameError};

#[cfg(test)]
pub mod test_helper {
    use super::*;

    pub fn make_item(id: &str, name: &str) -> Item {
        Item::new(ItemId::new(id).unwrap(), ItemName::new(name).unwrap())
    }
}
