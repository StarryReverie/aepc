use getset::Getters;
use snafu::prelude::*;

use crate::domain::item::model::ItemId;
use crate::domain::machine::model::MachineId;
use crate::domain::recipe::model::{Flow, Rate, Replica};

use super::{Period, Quantity, RecipeId};

#[derive(Debug, Clone, PartialEq, Getters)]
#[getset(get = "pub")]
pub struct Recipe {
    id: RecipeId,
    machine: MachineId,
    period: Period,
    materials: Vec<(ItemId, Quantity)>,
    products: Vec<(ItemId, Quantity)>,
}

impl Recipe {
    pub fn new(
        id: RecipeId,
        machine: MachineId,
        period: Period,
        materials: Vec<(ItemId, Quantity)>,
        products: Vec<(ItemId, Quantity)>,
    ) -> Result<Self, NewRecipeError> {
        ensure!(!products.is_empty(), NoProductsSnafu);

        for (i, (mi, _)) in materials.iter().enumerate() {
            for (mj, _) in materials.iter().skip(i + 1) {
                ensure!(mi != mj, DuplicateMaterialSnafu);
            }
        }

        for (i, (pi, _)) in products.iter().enumerate() {
            for (pj, _) in products.iter().skip(i + 1) {
                ensure!(pi != pj, DuplicateProductSnafu);
            }
        }

        Ok(Self {
            id,
            machine,
            period,
            materials,
            products,
        })
    }

    pub fn get_product_rate(&self, product_id: &ItemId) -> Option<Rate> {
        self.products
            .iter()
            .find(|(target, _)| target == product_id)
            .map(|(_, quantity)| *quantity / self.period / Replica::one())
    }

    pub fn get_materials_flow(&self, replica: Replica) -> Vec<(&ItemId, Flow)> {
        self.materials
            .iter()
            .map(|(material, quantity)| {
                (material, *quantity / self.period / Replica::one() * replica)
            })
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Snafu)]
pub enum NewRecipeError {
    #[snafu(display("recipe must have at least one product"))]
    NoProducts,
    #[snafu(display("recipe has duplicate materials"))]
    DuplicateMaterial,
    #[snafu(display("recipe has duplicate products"))]
    DuplicateProduct,
}

#[cfg(test)]
mod tests {
    use anyhow::Result as AnyhowResult;

    use super::*;

    #[test]
    fn test_empty_products_returns_error() -> AnyhowResult<()> {
        let id = RecipeId::new("1")?;
        let machine = MachineId::new("1")?;
        let period = Period::new(30.0)?;
        let materials = vec![];
        let products = vec![];

        assert!(matches!(
            Recipe::new(id, machine, period, materials, products),
            Err(NewRecipeError::NoProducts),
        ));
        Ok(())
    }

    #[test]
    fn test_duplicate_materials_returns_error() -> AnyhowResult<()> {
        let id = RecipeId::new("1")?;
        let machine = MachineId::new("1")?;
        let period = Period::new(30.0)?;
        let materials = vec![
            (ItemId::new("item1")?, Quantity::new(10.0)?),
            (ItemId::new("item1")?, Quantity::new(20.0)?),
        ];
        let products = vec![(ItemId::new("item2")?, Quantity::new(5.0)?)];

        assert!(matches!(
            Recipe::new(id, machine, period, materials, products),
            Err(NewRecipeError::DuplicateMaterial),
        ));
        Ok(())
    }

    #[test]
    fn test_duplicate_products_returns_error() -> AnyhowResult<()> {
        let id = RecipeId::new("1")?;
        let machine = MachineId::new("1")?;
        let period = Period::new(30.0)?;
        let materials = vec![(ItemId::new("item1")?, Quantity::new(10.0)?)];
        let products = vec![
            (ItemId::new("item2")?, Quantity::new(5.0)?),
            (ItemId::new("item2")?, Quantity::new(3.0)?),
        ];

        assert!(matches!(
            Recipe::new(id, machine, period, materials, products),
            Err(NewRecipeError::DuplicateProduct),
        ));
        Ok(())
    }

    #[test]
    fn test_get_product_rate() -> AnyhowResult<()> {
        let id = RecipeId::new("1")?;
        let machine = MachineId::new("1")?;
        let period = Period::new(30.0)?;
        let materials = vec![(ItemId::new("item1")?, Quantity::new(10.0)?)];
        let products = vec![(ItemId::new("item2")?, Quantity::new(60.0)?)];

        let recipe = Recipe::new(id, machine, period, materials, products)?;

        let rate = recipe.get_product_rate(&ItemId::new("item2")?).unwrap();
        assert_eq!(rate, Rate::new(120.0)?);

        assert!(
            recipe
                .get_product_rate(&ItemId::new("nonexistent")?)
                .is_none()
        );
        Ok(())
    }

    #[test]
    fn test_get_materials_flow() -> AnyhowResult<()> {
        let id = RecipeId::new("1")?;
        let machine = MachineId::new("1")?;
        let period = Period::new(30.0)?;
        let materials = vec![(ItemId::new("item1")?, Quantity::new(10.0)?)];
        let products = vec![(ItemId::new("item2")?, Quantity::new(60.0)?)];

        let recipe = Recipe::new(id, machine, period, materials, products)?;

        let replica = Replica::new(1.0)?;
        let flows = recipe.get_materials_flow(replica);
        assert_eq!(flows.len(), 1);
        assert_eq!(flows[0].1, Flow::new(20.0)?);

        let replica = Replica::new(3.0)?;
        let flows = recipe.get_materials_flow(replica);
        assert_eq!(flows.len(), 1);
        assert_eq!(flows[0].1, Flow::new(60.0)?);
        Ok(())
    }
}
