mod flow;
mod period;
mod quantity;
mod rate;
mod recipe;
mod recipe_id;
mod replica;

pub use flow::{Flow, NewFlowError};
pub use period::{NewPeriodError, Period};
pub use quantity::{NewQuantityError, Quantity};
pub use rate::{NewRateError, Rate};
pub use recipe::{NewRecipeError, Recipe};
pub use recipe_id::{NewRecipeIdError, RecipeId};
pub use replica::{NewReplicaError, Replica};

#[cfg(test)]
pub mod test_helper {
    use crate::domain::item::model::ItemId;
    use crate::domain::machine::model::MachineId;

    use super::*;

    pub fn make_recipe(
        recipe_id: &str,
        machine_id: &str,
        period: f64,
        materials: Vec<(&str, f64)>,
        products: Vec<(&str, f64)>,
    ) -> Recipe {
        let materials = materials
            .into_iter()
            .map(|(id, qty)| (ItemId::new(id).unwrap(), Quantity::new(qty).unwrap()))
            .collect();

        let products = products
            .into_iter()
            .map(|(id, qty)| (ItemId::new(id).unwrap(), Quantity::new(qty).unwrap()))
            .collect();

        Recipe::new(
            RecipeId::new(recipe_id).unwrap(),
            MachineId::new(machine_id).unwrap(),
            Period::new(period).unwrap(),
            materials,
            products,
        )
        .unwrap()
    }
}
