use std::collections::BTreeMap;

use anyhow::Result as AnyhowResult;

use crate::domain::item::model::ItemId;
use crate::domain::machine::model::MachineId;
use crate::domain::recipe::model::{Period, Quantity, Recipe, RecipeId};
use crate::domain::recipe::outbound::RecipeRepository;

pub struct RecipeConstantRepository {
    recipes: BTreeMap<RecipeId, Recipe>,
}

impl RecipeConstantRepository {
    pub fn new() -> Self {
        Self {
            recipes: get_recipes()
                .into_iter()
                .map(|recipe| (recipe.id().clone(), recipe))
                .collect(),
        }
    }
}

impl RecipeRepository for RecipeConstantRepository {
    async fn get(&self, recipe_id: &RecipeId) -> AnyhowResult<Option<Recipe>> {
        Ok(self.recipes.get(recipe_id).cloned())
    }

    async fn find_all_by_products_containing_target(
        &self,
        target_id: &ItemId,
    ) -> AnyhowResult<Vec<Recipe>> {
        let recipes = self
            .recipes
            .values()
            .filter(|recipe| {
                recipe
                    .products()
                    .iter()
                    .any(|(item_id, _)| item_id == target_id)
            })
            .cloned()
            .collect::<Vec<_>>();
        Ok(recipes)
    }
}

fn get_recipes() -> Vec<Recipe> {
    try_get_recipes().expect("the recipe repository should be built from valid data")
}

fn try_get_recipes() -> AnyhowResult<Vec<Recipe>> {
    Ok(vec![
        // Natural Resources
        Recipe::new(
            RecipeId::new("originium-ore")?,
            MachineId::new("electric-mining-rig")?,
            Period::new(3.0)?,
            vec![],
            vec![(ItemId::new("originium-ore")?, Quantity::new(1.0)?)],
        )?,
        Recipe::new(
            RecipeId::new("amethyst-ore")?,
            MachineId::new("electric-mining-rig")?,
            Period::new(3.0)?,
            vec![],
            vec![(ItemId::new("amethyst-ore")?, Quantity::new(1.0)?)],
        )?,
        Recipe::new(
            RecipeId::new("ferrium-ore")?,
            MachineId::new("electric-mining-rig-mk-ii")?,
            Period::new(3.0)?,
            vec![],
            vec![(ItemId::new("ferrium-ore")?, Quantity::new(1.0)?)],
        )?,
        Recipe::new(
            RecipeId::new("cuprium-ore")?,
            MachineId::new("hydro-mining-rig")?,
            Period::new(3.0)?,
            vec![],
            vec![(ItemId::new("cuprium-ore")?, Quantity::new(1.0)?)],
        )?,
        Recipe::new(
            RecipeId::new("buckflower")?,
            MachineId::new("planting-unit")?,
            Period::new(2.0)?,
            vec![(ItemId::new("buckflower-seed")?, Quantity::new(1.0)?)],
            vec![(ItemId::new("buckflower")?, Quantity::new(1.0)?)],
        )?,
        Recipe::new(
            RecipeId::new("buckflower-seed")?,
            MachineId::new("seed-picking-unit")?,
            Period::new(2.0)?,
            vec![(ItemId::new("buckflower")?, Quantity::new(1.0)?)],
            vec![(ItemId::new("buckflower-seed")?, Quantity::new(2.0)?)],
        )?,
        Recipe::new(
            RecipeId::new("citrome")?,
            MachineId::new("planting-unit")?,
            Period::new(2.0)?,
            vec![(ItemId::new("citrome-seed")?, Quantity::new(1.0)?)],
            vec![(ItemId::new("citrome")?, Quantity::new(1.0)?)],
        )?,
        Recipe::new(
            RecipeId::new("citrome-seed")?,
            MachineId::new("seed-picking-unit")?,
            Period::new(2.0)?,
            vec![(ItemId::new("citrome")?, Quantity::new(1.0)?)],
            vec![(ItemId::new("citrome-seed")?, Quantity::new(2.0)?)],
        )?,
        Recipe::new(
            RecipeId::new("sandleaf")?,
            MachineId::new("planting-unit")?,
            Period::new(2.0)?,
            vec![(ItemId::new("sandleaf-seed")?, Quantity::new(1.0)?)],
            vec![(ItemId::new("sandleaf")?, Quantity::new(1.0)?)],
        )?,
        Recipe::new(
            RecipeId::new("sandleaf-seed")?,
            MachineId::new("seed-picking-unit")?,
            Period::new(2.0)?,
            vec![(ItemId::new("sandleaf")?, Quantity::new(1.0)?)],
            vec![(ItemId::new("sandleaf-seed")?, Quantity::new(2.0)?)],
        )?,
        Recipe::new(
            RecipeId::new("aketine")?,
            MachineId::new("planting-unit")?,
            Period::new(2.0)?,
            vec![(ItemId::new("aketine-seed")?, Quantity::new(1.0)?)],
            vec![(ItemId::new("aketine")?, Quantity::new(1.0)?)],
        )?,
        Recipe::new(
            RecipeId::new("aketine-seed")?,
            MachineId::new("seed-picking-unit")?,
            Period::new(2.0)?,
            vec![(ItemId::new("aketine")?, Quantity::new(1.0)?)],
            vec![(ItemId::new("aketine-seed")?, Quantity::new(2.0)?)],
        )?,
        Recipe::new(
            RecipeId::new("jincao")?,
            MachineId::new("planting-unit")?,
            Period::new(2.0)?,
            vec![
                (ItemId::new("jincao-seed")?, Quantity::new(1.0)?),
                (ItemId::new("clean-water")?, Quantity::new(1.0)?),
            ],
            vec![(ItemId::new("jincao")?, Quantity::new(2.0)?)],
        )?,
        Recipe::new(
            RecipeId::new("jincao-seed")?,
            MachineId::new("seed-picking-unit")?,
            Period::new(2.0)?,
            vec![(ItemId::new("jincao")?, Quantity::new(1.0)?)],
            vec![(ItemId::new("jincao-seed")?, Quantity::new(1.0)?)],
        )?,
        Recipe::new(
            RecipeId::new("yazhen")?,
            MachineId::new("planting-unit")?,
            Period::new(2.0)?,
            vec![
                (ItemId::new("yazhen-seed")?, Quantity::new(1.0)?),
                (ItemId::new("clean-water")?, Quantity::new(1.0)?),
            ],
            vec![(ItemId::new("yazhen")?, Quantity::new(2.0)?)],
        )?,
        Recipe::new(
            RecipeId::new("yazhen-seed")?,
            MachineId::new("seed-picking-unit")?,
            Period::new(2.0)?,
            vec![(ItemId::new("yazhen")?, Quantity::new(1.0)?)],
            vec![(ItemId::new("yazhen-seed")?, Quantity::new(1.0)?)],
        )?,
        Recipe::new(
            RecipeId::new("clean-water")?,
            MachineId::new("fluid-pump")?,
            Period::new(1.0)?,
            vec![],
            vec![(ItemId::new("clean-water")?, Quantity::new(1.0)?)],
        )?,
        // AIC Products
        Recipe::new(
            RecipeId::new("jincao-solutin__reactor-crucible")?,
            MachineId::new("reactor-crucible")?,
            Period::new(2.0)?,
            vec![
                (ItemId::new("jincao-powder")?, Quantity::new(1.0)?),
                (ItemId::new("clean-water")?, Quantity::new(1.0)?),
            ],
            vec![(ItemId::new("jincao-solution")?, Quantity::new(1.0)?)],
        )?,
        Recipe::new(
            RecipeId::new("yazhen-solutin__reactor-crucible")?,
            MachineId::new("reactor-crucible")?,
            Period::new(2.0)?,
            vec![
                (ItemId::new("yazhen-powder")?, Quantity::new(1.0)?),
                (ItemId::new("clean-water")?, Quantity::new(1.0)?),
            ],
            vec![(ItemId::new("yazhen-solution")?, Quantity::new(1.0)?)],
        )?,
        Recipe::new(
            RecipeId::new("liquid-xiranite__reactor-crucible")?,
            MachineId::new("reactor-crucible")?,
            Period::new(2.0)?,
            vec![
                (ItemId::new("xiranite")?, Quantity::new(1.0)?),
                (ItemId::new("clean-water")?, Quantity::new(1.0)?),
            ],
            vec![(ItemId::new("liquid-xiranite")?, Quantity::new(1.0)?)],
        )?,
        Recipe::new(
            RecipeId::new("xircon-effluent+inert-xircon-effluent")?,
            MachineId::new("reactor-crucible")?,
            Period::new(2.0)?,
            vec![
                (ItemId::new("liquid-xiranite")?, Quantity::new(1.0)?),
                (ItemId::new("sewage")?, Quantity::new(1.0)?),
            ],
            vec![
                (ItemId::new("xircon-effluent")?, Quantity::new(1.0)?),
                (ItemId::new("inert-xircon-effluent")?, Quantity::new(1.0)?),
            ],
        )?,
        // TODO: carbon__refining-unit__buckflower
        // TODO: carbon__refining-unit__citrome
        Recipe::new(
            RecipeId::new("carbon__refining-unit__jincao")?,
            MachineId::new("refining-unit")?,
            Period::new(2.0)?,
            vec![(ItemId::new("jincao")?, Quantity::new(1.0)?)],
            vec![(ItemId::new("carbon")?, Quantity::new(2.0)?)],
        )?,
        Recipe::new(
            RecipeId::new("origocrust")?,
            MachineId::new("refining-unit")?,
            Period::new(2.0)?,
            vec![(ItemId::new("originium-ore")?, Quantity::new(1.0)?)],
            vec![(ItemId::new("origocrust")?, Quantity::new(1.0)?)],
        )?,
        Recipe::new(
            RecipeId::new("amethyst-fiber")?,
            MachineId::new("refining-unit")?,
            Period::new(2.0)?,
            vec![(ItemId::new("amethyst-ore")?, Quantity::new(1.0)?)],
            vec![(ItemId::new("amethyst-fiber")?, Quantity::new(1.0)?)],
        )?,
        Recipe::new(
            RecipeId::new("ferrium")?,
            MachineId::new("refining-unit")?,
            Period::new(2.0)?,
            vec![(ItemId::new("ferrium-ore")?, Quantity::new(1.0)?)],
            vec![(ItemId::new("ferrium")?, Quantity::new(1.0)?)],
        )?,
        Recipe::new(
            RecipeId::new("cuprium+sewage")?,
            MachineId::new("refining-unit")?,
            Period::new(2.0)?,
            vec![
                (ItemId::new("cuprium-ore")?, Quantity::new(1.0)?),
                (ItemId::new("clean-water")?, Quantity::new(1.0)?),
            ],
            vec![
                (ItemId::new("cuprium")?, Quantity::new(1.0)?),
                (ItemId::new("sewage")?, Quantity::new(1.0)?),
            ],
        )?,
        Recipe::new(
            RecipeId::new("stabilized-carbon")?,
            MachineId::new("refining-unit")?,
            Period::new(2.0)?,
            vec![(ItemId::new("dense-carbon-powder")?, Quantity::new(1.0)?)],
            vec![(ItemId::new("stabilized-carbon")?, Quantity::new(1.0)?)],
        )?,
        Recipe::new(
            RecipeId::new("packed-origocrust")?,
            MachineId::new("refining-unit")?,
            Period::new(2.0)?,
            vec![(ItemId::new("dense-origocrust-powder")?, Quantity::new(1.0)?)],
            vec![(ItemId::new("packed-origocrust")?, Quantity::new(1.0)?)],
        )?,
        Recipe::new(
            RecipeId::new("cryston-fiber")?,
            MachineId::new("refining-unit")?,
            Period::new(2.0)?,
            vec![(ItemId::new("cryston-powder")?, Quantity::new(1.0)?)],
            vec![(ItemId::new("cryston-fiber")?, Quantity::new(1.0)?)],
        )?,
        Recipe::new(
            RecipeId::new("steel")?,
            MachineId::new("refining-unit")?,
            Period::new(2.0)?,
            vec![(ItemId::new("dense-ferrium-powder")?, Quantity::new(1.0)?)],
            vec![(ItemId::new("steel")?, Quantity::new(1.0)?)],
        )?,
        Recipe::new(
            RecipeId::new("xiranite")?,
            MachineId::new("forge-of-the-sky")?,
            Period::new(2.0)?,
            vec![
                (ItemId::new("stabilized-carbon")?, Quantity::new(2.0)?),
                (ItemId::new("clean-water")?, Quantity::new(1.0)?),
            ],
            vec![(ItemId::new("xiranite")?, Quantity::new(1.0)?)],
        )?,
        Recipe::new(
            RecipeId::new("xircon")?,
            MachineId::new("reactor-crucible")?,
            Period::new(2.0)?,
            vec![
                (ItemId::new("xircon-effluent")?, Quantity::new(2.0)?),
                (ItemId::new("ferrium-powder")?, Quantity::new(1.0)?),
            ],
            vec![
                (ItemId::new("xircon")?, Quantity::new(1.0)?),
                (ItemId::new("sewage")?, Quantity::new(1.0)?),
            ],
        )?,
        Recipe::new(
            RecipeId::new("carbon-powder__refining-unit")?,
            MachineId::new("refining-unit")?,
            Period::new(2.0)?,
            vec![(ItemId::new("jincao-powder")?, Quantity::new(1.0)?)],
            vec![(ItemId::new("carbon-powder")?, Quantity::new(2.0)?)],
        )?,
        Recipe::new(
            RecipeId::new("carbon-powder__shredding-unit")?,
            MachineId::new("shredding-unit")?,
            Period::new(2.0)?,
            vec![(ItemId::new("carbon")?, Quantity::new(1.0)?)],
            vec![(ItemId::new("carbon-powder")?, Quantity::new(2.0)?)],
        )?,
        Recipe::new(
            RecipeId::new("originium-powder")?,
            MachineId::new("shredding-unit")?,
            Period::new(2.0)?,
            vec![(ItemId::new("originium-ore")?, Quantity::new(1.0)?)],
            vec![(ItemId::new("originium-powder")?, Quantity::new(1.0)?)],
        )?,
        Recipe::new(
            RecipeId::new("origocrust-powder")?,
            MachineId::new("shredding-unit")?,
            Period::new(2.0)?,
            vec![(ItemId::new("origocrust")?, Quantity::new(1.0)?)],
            vec![(ItemId::new("origocrust-powder")?, Quantity::new(1.0)?)],
        )?,
        Recipe::new(
            RecipeId::new("amethyst-powder")?,
            MachineId::new("shredding-unit")?,
            Period::new(2.0)?,
            vec![(ItemId::new("amethyst-ore")?, Quantity::new(1.0)?)],
            vec![(ItemId::new("amethyst-powder")?, Quantity::new(1.0)?)],
        )?,
        Recipe::new(
            RecipeId::new("ferrium-powder")?,
            MachineId::new("shredding-unit")?,
            Period::new(2.0)?,
            vec![(ItemId::new("ferrium-ore")?, Quantity::new(1.0)?)],
            vec![(ItemId::new("ferrium-powder")?, Quantity::new(1.0)?)],
        )?,
        Recipe::new(
            RecipeId::new("cuprium-powder")?,
            MachineId::new("shredding-unit")?,
            Period::new(2.0)?,
            vec![(ItemId::new("cuprium-ore")?, Quantity::new(1.0)?)],
            vec![(ItemId::new("cuprium-powder")?, Quantity::new(1.0)?)],
        )?,
        Recipe::new(
            RecipeId::new("sandleaf-powder")?,
            MachineId::new("shredding-unit")?,
            Period::new(2.0)?,
            vec![(ItemId::new("sandleaf")?, Quantity::new(1.0)?)],
            vec![(ItemId::new("sandleaf-powder")?, Quantity::new(3.0)?)],
        )?,
        Recipe::new(
            RecipeId::new("aketine-powder")?,
            MachineId::new("shredding-unit")?,
            Period::new(2.0)?,
            vec![(ItemId::new("aketine")?, Quantity::new(1.0)?)],
            vec![(ItemId::new("aketine-powder")?, Quantity::new(2.0)?)],
        )?,
        Recipe::new(
            RecipeId::new("ground-buckflower-powder")?,
            MachineId::new("grinding-crucible")?,
            Period::new(2.0)?,
            vec![
                (ItemId::new("buckflower-powder")?, Quantity::new(2.0)?),
                (ItemId::new("sandleaf-powder")?, Quantity::new(1.0)?),
            ],
            vec![(
                ItemId::new("ground-buckflower-powder")?,
                Quantity::new(1.0)?,
            )],
        )?,
        Recipe::new(
            RecipeId::new("ground-citrome-powder")?,
            MachineId::new("grinding-crucible")?,
            Period::new(2.0)?,
            vec![
                (ItemId::new("citrome-powder")?, Quantity::new(2.0)?),
                (ItemId::new("sandleaf-powder")?, Quantity::new(1.0)?),
            ],
            vec![(ItemId::new("ground-citrome-powder")?, Quantity::new(1.0)?)],
        )?,
        Recipe::new(
            RecipeId::new("dense-carbon-powder")?,
            MachineId::new("grinding-crucible")?,
            Period::new(2.0)?,
            vec![
                (ItemId::new("carbon-powder")?, Quantity::new(2.0)?),
                (ItemId::new("sandleaf-powder")?, Quantity::new(1.0)?),
            ],
            vec![(ItemId::new("dense-carbon-powder")?, Quantity::new(1.0)?)],
        )?,
        Recipe::new(
            RecipeId::new("dense-originium-powder")?,
            MachineId::new("grinding-crucible")?,
            Period::new(2.0)?,
            vec![
                (ItemId::new("originium-powder")?, Quantity::new(2.0)?),
                (ItemId::new("sandleaf-powder")?, Quantity::new(1.0)?),
            ],
            vec![(ItemId::new("dense-originium-powder")?, Quantity::new(1.0)?)],
        )?,
        Recipe::new(
            RecipeId::new("dense-origocrust-powder__refining-unit")?,
            MachineId::new("refining-unit")?,
            Period::new(2.0)?,
            vec![(ItemId::new("dense-originium-powder")?, Quantity::new(1.0)?)],
            vec![(ItemId::new("dense-origocrust-powder")?, Quantity::new(1.0)?)],
        )?,
        Recipe::new(
            RecipeId::new("dense-origocrust-powder__grinding_crucible")?,
            MachineId::new("grinding-crucible")?,
            Period::new(2.0)?,
            vec![
                (ItemId::new("origocrust-powder")?, Quantity::new(2.0)?),
                (ItemId::new("sandleaf-powder")?, Quantity::new(1.0)?),
            ],
            vec![(ItemId::new("dense-origocrust-powder")?, Quantity::new(1.0)?)],
        )?,
        Recipe::new(
            RecipeId::new("cryston-powder")?,
            MachineId::new("grinding-crucible")?,
            Period::new(2.0)?,
            vec![
                (ItemId::new("amethyst-powder")?, Quantity::new(2.0)?),
                (ItemId::new("sandleaf-powder")?, Quantity::new(1.0)?),
            ],
            vec![(ItemId::new("cryston-powder")?, Quantity::new(1.0)?)],
        )?,
        Recipe::new(
            RecipeId::new("dense-ferrium-powder")?,
            MachineId::new("grinding-crucible")?,
            Period::new(2.0)?,
            vec![
                (ItemId::new("ferrium-powder")?, Quantity::new(2.0)?),
                (ItemId::new("sandleaf-powder")?, Quantity::new(1.0)?),
            ],
            vec![(ItemId::new("dense-ferrium-powder")?, Quantity::new(1.0)?)],
        )?,
        Recipe::new(
            RecipeId::new("amethyst-bottle")?,
            MachineId::new("moulding-unit")?,
            Period::new(2.0)?,
            vec![(ItemId::new("amethyst-fiber")?, Quantity::new(2.0)?)],
            vec![(ItemId::new("amethyst-bottle")?, Quantity::new(1.0)?)],
        )?,
        // TODO: ferrium-bottle+clean-water__separating-unit
        // TODO: ferrium-bottle+jincao-solution__separating-unit
        // TODO: ferrium-bottle+yazhen-solution__separating-unit
        // TODO: ferrium-bottle+liquid-xiranite__separating-unit
        Recipe::new(
            RecipeId::new("ferrium-bottle__moulding-unit")?,
            MachineId::new("moulding-unit")?,
            Period::new(2.0)?,
            vec![(ItemId::new("ferrium")?, Quantity::new(2.0)?)],
            vec![(ItemId::new("ferrium-bottle")?, Quantity::new(1.0)?)],
        )?,
        Recipe::new(
            RecipeId::new("cryston-bottle")?,
            MachineId::new("moulding-unit")?,
            Period::new(2.0)?,
            vec![(ItemId::new("cryston-fiber")?, Quantity::new(2.0)?)],
            vec![(ItemId::new("cryston-bottle")?, Quantity::new(1.0)?)],
        )?,
        Recipe::new(
            RecipeId::new("steel-bottle")?,
            MachineId::new("moulding-unit")?,
            Period::new(2.0)?,
            vec![(ItemId::new("steel")?, Quantity::new(2.0)?)],
            vec![(ItemId::new("steel-bottle")?, Quantity::new(1.0)?)],
        )?,
        // TODO: cuprium-bottle+jincao-solution__separating-unit
        // TODO: cuprium-bottle+yazhen-solution__separating-unit
        Recipe::new(
            RecipeId::new("cuprium-bottle__moulding-unit")?,
            MachineId::new("moulding-unit")?,
            Period::new(2.0)?,
            vec![(ItemId::new("cuprium")?, Quantity::new(2.0)?)],
            vec![(ItemId::new("cuprium-bottle")?, Quantity::new(1.0)?)],
        )?,
        Recipe::new(
            RecipeId::new("amethyst-part")?,
            MachineId::new("fitting-unit")?,
            Period::new(2.0)?,
            vec![(ItemId::new("amethyst-fiber")?, Quantity::new(1.0)?)],
            vec![(ItemId::new("amethyst-part")?, Quantity::new(1.0)?)],
        )?,
        Recipe::new(
            RecipeId::new("ferrium-part")?,
            MachineId::new("fitting-unit")?,
            Period::new(2.0)?,
            vec![(ItemId::new("ferrium")?, Quantity::new(1.0)?)],
            vec![(ItemId::new("ferrium-part")?, Quantity::new(1.0)?)],
        )?,
        Recipe::new(
            RecipeId::new("cryston-part")?,
            MachineId::new("fitting-unit")?,
            Period::new(2.0)?,
            vec![(ItemId::new("cryston-fiber")?, Quantity::new(1.0)?)],
            vec![(ItemId::new("cryston-part")?, Quantity::new(1.0)?)],
        )?,
        Recipe::new(
            RecipeId::new("steel-part")?,
            MachineId::new("fitting-unit")?,
            Period::new(2.0)?,
            vec![(ItemId::new("steel")?, Quantity::new(1.0)?)],
            vec![(ItemId::new("steel-part")?, Quantity::new(1.0)?)],
        )?,
        Recipe::new(
            RecipeId::new("cuprium-part")?,
            MachineId::new("fitting-unit")?,
            Period::new(2.0)?,
            vec![(ItemId::new("cuprium")?, Quantity::new(1.0)?)],
            vec![(ItemId::new("cuprium-part")?, Quantity::new(1.0)?)],
        )?,
        Recipe::new(
            RecipeId::new("amethyst-component")?,
            MachineId::new("gearing-unit")?,
            Period::new(10.0)?,
            vec![
                (ItemId::new("origocrust")?, Quantity::new(5.0)?),
                (ItemId::new("amethyst-fiber")?, Quantity::new(5.0)?),
            ],
            vec![(ItemId::new("amethyst-component")?, Quantity::new(1.0)?)],
        )?,
        Recipe::new(
            RecipeId::new("ferrium-component")?,
            MachineId::new("gearing-unit")?,
            Period::new(10.0)?,
            vec![
                (ItemId::new("origocrust")?, Quantity::new(10.0)?),
                (ItemId::new("ferrium")?, Quantity::new(10.0)?),
            ],
            vec![(ItemId::new("ferrium-component")?, Quantity::new(1.0)?)],
        )?,
        Recipe::new(
            RecipeId::new("cryston-component")?,
            MachineId::new("gearing-unit")?,
            Period::new(10.0)?,
            vec![
                (ItemId::new("packed-origocrust")?, Quantity::new(10.0)?),
                (ItemId::new("cryston-fiber")?, Quantity::new(10.0)?),
            ],
            vec![(ItemId::new("cryston-component")?, Quantity::new(1.0)?)],
        )?,
        Recipe::new(
            RecipeId::new("xiranite-component")?,
            MachineId::new("gearing-unit")?,
            Period::new(10.0)?,
            vec![
                (ItemId::new("packed-origocrust")?, Quantity::new(10.0)?),
                (ItemId::new("xiranite")?, Quantity::new(10.0)?),
            ],
            vec![(ItemId::new("xiranite-component")?, Quantity::new(1.0)?)],
        )?,
        Recipe::new(
            RecipeId::new("cuprium-component")?,
            MachineId::new("gearing-unit")?,
            Period::new(10.0)?,
            vec![
                (ItemId::new("cuprium-part")?, Quantity::new(10.0)?),
                (ItemId::new("xiranite")?, Quantity::new(10.0)?),
            ],
            vec![(ItemId::new("cuprium-component")?, Quantity::new(1.0)?)],
        )?,
        Recipe::new(
            RecipeId::new("lc-valley-battery")?,
            MachineId::new("packaging-unit")?,
            Period::new(10.0)?,
            vec![
                (ItemId::new("amethyst-part")?, Quantity::new(5.0)?),
                (ItemId::new("originium-powder")?, Quantity::new(10.0)?),
            ],
            vec![(ItemId::new("lc-valley-battery")?, Quantity::new(1.0)?)],
        )?,
        Recipe::new(
            RecipeId::new("sc-valley-battery")?,
            MachineId::new("packaging-unit")?,
            Period::new(10.0)?,
            vec![
                (ItemId::new("ferrium-part")?, Quantity::new(10.0)?),
                (ItemId::new("originium-powder")?, Quantity::new(15.0)?),
            ],
            vec![(ItemId::new("sc-valley-battery")?, Quantity::new(1.0)?)],
        )?,
        Recipe::new(
            RecipeId::new("hc-valley-battery")?,
            MachineId::new("packaging-unit")?,
            Period::new(10.0)?,
            vec![
                (ItemId::new("steel-part")?, Quantity::new(10.0)?),
                (ItemId::new("dense-originium-powder")?, Quantity::new(15.0)?),
            ],
            vec![(ItemId::new("hc-valley-battery")?, Quantity::new(1.0)?)],
        )?,
        Recipe::new(
            RecipeId::new("lc-wuling-battery")?,
            MachineId::new("packaging-unit")?,
            Period::new(10.0)?,
            vec![
                (ItemId::new("xiranite")?, Quantity::new(5.0)?),
                (ItemId::new("dense-originium-powder")?, Quantity::new(15.0)?),
            ],
            vec![(ItemId::new("lc-wuling-battery")?, Quantity::new(1.0)?)],
        )?,
        Recipe::new(
            RecipeId::new("sc-wuling-battery")?,
            MachineId::new("packaging-unit")?,
            Period::new(10.0)?,
            vec![
                (ItemId::new("xircon")?, Quantity::new(5.0)?),
                (ItemId::new("dense-originium-powder")?, Quantity::new(20.0)?),
            ],
            vec![(ItemId::new("sc-wuling-battery")?, Quantity::new(1.0)?)],
        )?,
        Recipe::new(
            RecipeId::new("ferrium-bottle-clean-water")?,
            MachineId::new("filling-unit")?,
            Period::new(2.0)?,
            vec![
                (ItemId::new("ferrium-bottle")?, Quantity::new(1.0)?),
                (ItemId::new("clean-water")?, Quantity::new(1.0)?),
            ],
            vec![(
                ItemId::new("ferrium-bottle-clean-water")?,
                Quantity::new(1.0)?,
            )],
        )?,
        Recipe::new(
            RecipeId::new("ferrium-bottle-jincao-solution")?,
            MachineId::new("filling-unit")?,
            Period::new(2.0)?,
            vec![
                (ItemId::new("ferrium-bottle")?, Quantity::new(1.0)?),
                (ItemId::new("jincao-solution")?, Quantity::new(1.0)?),
            ],
            vec![(
                ItemId::new("ferrium-bottle-jincao-solution")?,
                Quantity::new(1.0)?,
            )],
        )?,
        Recipe::new(
            RecipeId::new("ferrium-bottle-yazhen-solution")?,
            MachineId::new("filling-unit")?,
            Period::new(2.0)?,
            vec![
                (ItemId::new("ferrium-bottle")?, Quantity::new(1.0)?),
                (ItemId::new("yazhen-solution")?, Quantity::new(1.0)?),
            ],
            vec![(
                ItemId::new("ferrium-bottle-yazhen-solution")?,
                Quantity::new(1.0)?,
            )],
        )?,
        Recipe::new(
            RecipeId::new("ferrium-bottle-liquid-xiranite")?,
            MachineId::new("filling-unit")?,
            Period::new(2.0)?,
            vec![
                (ItemId::new("ferrium-bottle")?, Quantity::new(1.0)?),
                (ItemId::new("liquid-xiranite")?, Quantity::new(1.0)?),
            ],
            vec![(
                ItemId::new("ferrium-bottle-liquid-xiranite")?,
                Quantity::new(1.0)?,
            )],
        )?,
        Recipe::new(
            RecipeId::new("cuprium-bottle-jincao-solution")?,
            MachineId::new("filling-unit")?,
            Period::new(2.0)?,
            vec![
                (ItemId::new("cuprium-bottle")?, Quantity::new(1.0)?),
                (ItemId::new("jincao-solution")?, Quantity::new(1.0)?),
            ],
            vec![(
                ItemId::new("cuprium-bottle-jincao-solution")?,
                Quantity::new(1.0)?,
            )],
        )?,
        Recipe::new(
            RecipeId::new("cuprium-bottle-yazhen-solution")?,
            MachineId::new("filling-unit")?,
            Period::new(2.0)?,
            vec![
                (ItemId::new("cuprium-bottle")?, Quantity::new(1.0)?),
                (ItemId::new("yazhen-solution")?, Quantity::new(1.0)?),
            ],
            vec![(
                ItemId::new("cuprium-bottle-yazhen-solution")?,
                Quantity::new(1.0)?,
            )],
        )?,
        // Usable Items
        Recipe::new(
            RecipeId::new("industrial-explosive")?,
            MachineId::new("packaging-unit")?,
            Period::new(10.0)?,
            vec![
                (ItemId::new("amethyst-part")?, Quantity::new(5.0)?),
                (ItemId::new("aketine-powder")?, Quantity::new(1.0)?),
            ],
            vec![(ItemId::new("industrial-explosive")?, Quantity::new(1.0)?)],
        )?,
        Recipe::new(
            RecipeId::new("buckflower-powder")?,
            MachineId::new("shredding-unit")?,
            Period::new(2.0)?,
            vec![(ItemId::new("buckflower")?, Quantity::new(1.0)?)],
            vec![(ItemId::new("buckflower-powder")?, Quantity::new(2.0)?)],
        )?,
        Recipe::new(
            RecipeId::new("citrome-powder")?,
            MachineId::new("shredding-unit")?,
            Period::new(2.0)?,
            vec![(ItemId::new("citrome")?, Quantity::new(1.0)?)],
            vec![(ItemId::new("citrome-powder")?, Quantity::new(2.0)?)],
        )?,
        Recipe::new(
            RecipeId::new("jincao-powder")?,
            MachineId::new("shredding-unit")?,
            Period::new(2.0)?,
            vec![(ItemId::new("jincao")?, Quantity::new(1.0)?)],
            vec![(ItemId::new("jincao-powder")?, Quantity::new(2.0)?)],
        )?,
        Recipe::new(
            RecipeId::new("yazhen-powder")?,
            MachineId::new("shredding-unit")?,
            Period::new(2.0)?,
            vec![(ItemId::new("yazhen")?, Quantity::new(1.0)?)],
            vec![(ItemId::new("yazhen-powder")?, Quantity::new(2.0)?)],
        )?,
        Recipe::new(
            RecipeId::new("buck-capsule-c")?,
            MachineId::new("filling-unit")?,
            Period::new(10.0)?,
            vec![
                (ItemId::new("amethyst-bottle")?, Quantity::new(5.0)?),
                (ItemId::new("buckflower-powder")?, Quantity::new(5.0)?),
            ],
            vec![(ItemId::new("buck-capsule-c")?, Quantity::new(1.0)?)],
        )?,
        Recipe::new(
            RecipeId::new("buck-capsule-b")?,
            MachineId::new("filling-unit")?,
            Period::new(10.0)?,
            vec![
                (ItemId::new("ferrium-bottle")?, Quantity::new(10.0)?),
                (ItemId::new("buckflower-powder")?, Quantity::new(10.0)?),
            ],
            vec![(ItemId::new("buck-capsule-b")?, Quantity::new(1.0)?)],
        )?,
        Recipe::new(
            RecipeId::new("canned-citrome-c")?,
            MachineId::new("filling-unit")?,
            Period::new(10.0)?,
            vec![
                (ItemId::new("amethyst-bottle")?, Quantity::new(5.0)?),
                (ItemId::new("citrome-powder")?, Quantity::new(5.0)?),
            ],
            vec![(ItemId::new("canned-citrome-c")?, Quantity::new(1.0)?)],
        )?,
        Recipe::new(
            RecipeId::new("canned-citrome-b")?,
            MachineId::new("filling-unit")?,
            Period::new(10.0)?,
            vec![
                (ItemId::new("ferrium-bottle")?, Quantity::new(10.0)?),
                (ItemId::new("citrome-powder")?, Quantity::new(10.0)?),
            ],
            vec![(ItemId::new("canned-citrome-b")?, Quantity::new(1.0)?)],
        )?,
        Recipe::new(
            RecipeId::new("jincao-drink")?,
            MachineId::new("packaging-unit")?,
            Period::new(10.0)?,
            vec![
                (ItemId::new("ferrium-part")?, Quantity::new(10.0)?),
                (
                    ItemId::new("ferrium-bottle-jincao-solution")?,
                    Quantity::new(5.0)?,
                ),
            ],
            vec![(ItemId::new("jincao-drink")?, Quantity::new(1.0)?)],
        )?,
        Recipe::new(
            RecipeId::new("yazhen-syringe-c")?,
            MachineId::new("packaging-unit")?,
            Period::new(10.0)?,
            vec![
                (ItemId::new("ferrium-part")?, Quantity::new(10.0)?),
                (
                    ItemId::new("ferrium-bottle-yazhen-solution")?,
                    Quantity::new(5.0)?,
                ),
            ],
            vec![(ItemId::new("yazhen-syringe-c")?, Quantity::new(1.0)?)],
        )?,
        Recipe::new(
            RecipeId::new("buck-capsule-a")?,
            MachineId::new("filling-unit")?,
            Period::new(10.0)?,
            vec![
                (ItemId::new("steel-bottle")?, Quantity::new(10.0)?),
                (
                    ItemId::new("ground-buckflower-powder")?,
                    Quantity::new(10.0)?,
                ),
            ],
            vec![(ItemId::new("buck-capsule-a")?, Quantity::new(1.0)?)],
        )?,
        Recipe::new(
            RecipeId::new("canned-citrome-a")?,
            MachineId::new("filling-unit")?,
            Period::new(10.0)?,
            vec![
                (ItemId::new("steel-bottle")?, Quantity::new(10.0)?),
                (ItemId::new("ground-citrome-powder")?, Quantity::new(10.0)?),
            ],
            vec![(ItemId::new("canned-citrome-a")?, Quantity::new(1.0)?)],
        )?,
        Recipe::new(
            RecipeId::new("jincao-tea")?,
            MachineId::new("packaging-unit")?,
            Period::new(10.0)?,
            vec![
                (ItemId::new("cuprium-part")?, Quantity::new(10.0)?),
                (
                    ItemId::new("cuprium-bottle-jincao-solution")?,
                    Quantity::new(5.0)?,
                ),
            ],
            vec![(ItemId::new("jincao-tea")?, Quantity::new(1.0)?)],
        )?,
        Recipe::new(
            RecipeId::new("yazhen-syringe-a")?,
            MachineId::new("packaging-unit")?,
            Period::new(10.0)?,
            vec![
                (ItemId::new("cuprium-part")?, Quantity::new(10.0)?),
                (
                    ItemId::new("cuprium-bottle-yazhen-solution")?,
                    Quantity::new(5.0)?,
                ),
            ],
            vec![(ItemId::new("yazhen-syringe-a")?, Quantity::new(1.0)?)],
        )?,
    ])
}
