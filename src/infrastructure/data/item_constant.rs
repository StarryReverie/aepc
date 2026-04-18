use std::collections::BTreeMap;

use anyhow::Result as AnyhowResult;

use crate::domain::item::model::{Item, ItemId, ItemName};
use crate::domain::item::outbound::ItemRepository;

pub struct ItemConstantRepository {
    items: BTreeMap<ItemId, Item>,
}

impl ItemConstantRepository {
    pub fn new() -> Self {
        Self {
            items: get_items()
                .into_iter()
                .map(|item| (item.id().clone(), item))
                .collect(),
        }
    }
}

impl ItemRepository for ItemConstantRepository {
    async fn get(&self, item_id: &ItemId) -> AnyhowResult<Option<Item>> {
        Ok(self.items.get(item_id).cloned())
    }

    async fn find_all_by_name_containing_pattern(&self, pattern: &str) -> AnyhowResult<Vec<Item>> {
        let pattern = pattern.to_lowercase();
        let items = self
            .items
            .values()
            .filter(|item| item.name().value().to_lowercase().contains(&pattern))
            .cloned()
            .collect::<Vec<_>>();
        Ok(items)
    }
}

fn get_items() -> Vec<Item> {
    try_get_items().expect("the item repository should be built from valid data")
}

#[rustfmt::skip]
fn try_get_items() -> AnyhowResult<Vec<Item>> {
    Ok(vec![
        // Natural Resources
        Item::new(
            ItemId::new("originium-ore")?,
            ItemName::new("Originium Ore")?,
        ),
        Item::new(
            ItemId::new("amethyst-ore")?,
            ItemName::new("Amethyst Ore")?,
        ),
        Item::new(
            ItemId::new("ferrium-ore")?,
            ItemName::new("Ferrium Ore")?,
        ),
        Item::new(
            ItemId::new("cuprium-ore")?,
            ItemName::new("Cuprium Ore")?,
        ),
        Item::new(
            ItemId::new("buckflower")?,
            ItemName::new("Buckflower")?,
        ),
        Item::new(
            ItemId::new("buckflower-seed")?,
            ItemName::new("Buckflower Seed")?,
        ),
        Item::new(
            ItemId::new("citrome")?,
            ItemName::new("Citrome")?,
        ),
        Item::new(
            ItemId::new("citrome-seed")?,
            ItemName::new("Citrome Seed")?,
        ),
        Item::new(
            ItemId::new("sandleaf")?,
            ItemName::new("Sandleaf")?,
        ),
        Item::new(
            ItemId::new("sandleaf-seed")?,
            ItemName::new("Sandleaf Seed")?,
        ),
        Item::new(
            ItemId::new("aketine")?,
            ItemName::new("Aketine")?,
        ),
        Item::new(
            ItemId::new("aketine-seed")?,
            ItemName::new("Aketine Seed")?,
        ),
        Item::new(
            ItemId::new("jincao")?,
            ItemName::new("Jincao")?,
        ),
        Item::new(
            ItemId::new("jincao-seed")?,
            ItemName::new("Jincao Seed")?,
        ),
        Item::new(
            ItemId::new("yazhen")?,
            ItemName::new("Yazhen")?,
        ),
        Item::new(
            ItemId::new("yazhen-seed")?,
            ItemName::new("Yazhen Seed")?,
        ),
        Item::new(
            ItemId::new("clean-water")?,
            ItemName::new("Clean Water")?,
        ),
        Item::new(
            ItemId::new("precipitation-acid")?,
            ItemName::new("Precipitation Acid")?,
        ),
        // AIC Products
        Item::new(
            ItemId::new("jincao-solution")?,
            ItemName::new("Jincao Solution")?,
        ),
        Item::new(
            ItemId::new("yazhen-solution")?,
            ItemName::new("Yazhen Solution")?,
        ),
        Item::new(
            ItemId::new("liquid-xiranite")?,
            ItemName::new("Liquid Xiranite")?,
        ),
        Item::new(
            ItemId::new("liquid-heavy-xiranite")?,
            ItemName::new("Liquid Heavy Xiranite")?,
        ),
        Item::new(
            ItemId::new("xircon-effluent")?,
            ItemName::new("Xircon Effluent")?,
        ),
        Item::new(
            ItemId::new("inert-xircon-effluent")?,
            ItemName::new("Inert Xircon Effluent")?,
        ),
        Item::new(
            ItemId::new("hetonite-solution")?,
            ItemName::new("Hetonite Solution")?,
        ),
        Item::new(
            ItemId::new("cuprium-solution")?,
            ItemName::new("Cuprium Solution")?,
        ),
        Item::new(
            ItemId::new("sewage")?,
            ItemName::new("Sewage")?,
        ),
        Item::new(
            ItemId::new("carbon")?,
            ItemName::new("Carbon")?,
        ),
        Item::new(
            ItemId::new("origocrust")?,
            ItemName::new("Origocrust")?,
        ),
        Item::new(
            ItemId::new("amethyst-fiber")?,
            ItemName::new("Amethyst Fiber")?,
        ),
        Item::new(
            ItemId::new("ferrium")?,
            ItemName::new("Ferrium")?,
        ),
        Item::new(
            ItemId::new("cuprium")?,
            ItemName::new("Cuprium")?,
        ),
        Item::new(
            ItemId::new("stabilized-carbon")?,
            ItemName::new("Stabilized Carbon")?,
        ),
        Item::new(
            ItemId::new("packed-origocrust")?,
            ItemName::new("Packed Origocrust")?,
        ),
        Item::new(
            ItemId::new("cryston-fiber")?,
            ItemName::new("Cryston Fiber")?,
        ),
        Item::new(
            ItemId::new("steel")?,
            ItemName::new("Steel")?,
        ),
        Item::new(
            ItemId::new("hetonite")?,
            ItemName::new("Hetonite")?,
        ),
        Item::new(
            ItemId::new("bumper-rich")?,
            ItemName::new("Bumper-Rich")?,
        ),
        Item::new(
            ItemId::new("xiranite")?,
            ItemName::new("Xiranite")?,
        ),
        Item::new(
            ItemId::new("heavy-xiranite")?,
            ItemName::new("Heavy Xiranite")?,
        ),
        Item::new(
            ItemId::new("xircon")?,
            ItemName::new("Xircon")?,
        ),
        Item::new(
            ItemId::new("carbon-powder")?,
            ItemName::new("Carbon Powder")?,
        ),
        Item::new(
            ItemId::new("originium-powder")?,
            ItemName::new("Originium Powder")?,
        ),
        Item::new(
            ItemId::new("origocrust-powder")?,
            ItemName::new("Origocrust Powder")?,
        ),
        Item::new(
            ItemId::new("amethyst-powder")?,
            ItemName::new("Amethyst Powder")?,
        ),
        Item::new(
            ItemId::new("ferrium-powder")?,
            ItemName::new("Ferrium Powder")?,
        ),
        Item::new(
            ItemId::new("cuprium-powder")?,
            ItemName::new("Cuprium Powder")?,
        ),
        Item::new(
            ItemId::new("sandleaf-powder")?,
            ItemName::new("Sandleaf Powder")?,
        ),
        Item::new(
            ItemId::new("aketine-powder")?,
            ItemName::new("Aketine Powder")?,
        ),
        Item::new(
            ItemId::new("ground-buckflower-powder")?,
            ItemName::new("Ground Buckflower Powder")?,
        ),
        Item::new(
            ItemId::new("ground-citrome-powder")?,
            ItemName::new("Ground Citrome Powder")?,
        ),
        Item::new(
            ItemId::new("dense-carbon-powder")?,
            ItemName::new("Dense Carbon Powder")?,
        ),
        Item::new(
            ItemId::new("dense-originium-powder")?,
            ItemName::new("Dense Originium Powder")?,
        ),
        Item::new(
            ItemId::new("dense-origocrust-powder")?,
            ItemName::new("Dense Origocrust Powder")?,
        ),
        Item::new(
            ItemId::new("cryston-powder")?,
            ItemName::new("Cryston Powder")?,
        ),
        Item::new(
            ItemId::new("dense-ferrium-powder")?,
            ItemName::new("Dense Ferrium Powder")?,
        ),
        Item::new(
            ItemId::new("amethyst-bottle")?,
            ItemName::new("Amethyst Bottle")?,
        ),
        Item::new(
            ItemId::new("ferrium-bottle")?,
            ItemName::new("Ferrium Bottle")?,
        ),
        Item::new(
            ItemId::new("cryston-bottle")?,
            ItemName::new("Cryston Bottle")?,
        ),
        Item::new(
            ItemId::new("steel-bottle")?,
            ItemName::new("Steel Bottle")?,
        ),
        Item::new(
            ItemId::new("cuprium-bottle")?,
            ItemName::new("Cuprium Bottle")?,
        ),
        Item::new(
            ItemId::new("hetonite-bottle")?,
            ItemName::new("Hetonite Bottle")?,
        ),
        Item::new(
            ItemId::new("amethyst-part")?,
            ItemName::new("Amethyst Part")?,
        ),
        Item::new(
            ItemId::new("ferrium-part")?,
            ItemName::new("Ferrium Part")?,
        ),
        Item::new(
            ItemId::new("cryston-part")?,
            ItemName::new("Cryston Part")?,
        ),
        Item::new(
            ItemId::new("steel-part")?,
            ItemName::new("Steel Part")?,
        ),
        Item::new(
            ItemId::new("cuprium-part")?,
            ItemName::new("Cuprium Part")?,
        ),
        Item::new(
            ItemId::new("hetonite-part")?,
            ItemName::new("Hetonite Part")?,
        ),
        Item::new(
            ItemId::new("amethyst-component")?,
            ItemName::new("Amethyst Component")?,
        ),
        Item::new(
            ItemId::new("ferrium-component")?,
            ItemName::new("Ferrium Component")?,
        ),
        Item::new(
            ItemId::new("cryston-component")?,
            ItemName::new("Cryston Component")?,
        ),
        Item::new(
            ItemId::new("xiranite-component")?,
            ItemName::new("Xiranite Component")?,
        ),
        Item::new(
            ItemId::new("cuprium-component")?,
            ItemName::new("Cuprium Component")?,
        ),
        Item::new(
            ItemId::new("hetonite-component")?,
            ItemName::new("Hetonite Component")?,
        ),
        Item::new(
            ItemId::new("lc-valley-battery")?,
            ItemName::new("LC Valley Battery")?,
        ),
        Item::new(
            ItemId::new("sc-valley-battery")?,
            ItemName::new("SC Valley Battery")?,
        ),
        Item::new(
            ItemId::new("hc-valley-battery")?,
            ItemName::new("HC Valley Battery")?,
        ),
        Item::new(
            ItemId::new("lc-wuling-battery")?,
            ItemName::new("LC Wuling Battery")?,
        ),
        Item::new(
            ItemId::new("sc-wuling-battery")?,
            ItemName::new("SC Wuling Battery")?,
        ),
        Item::new(
            ItemId::new("ferrium-bottle-clean-water")?,
            ItemName::new("Ferrium Bottle (Clean Water)")?,
        ),
        Item::new(
            ItemId::new("ferrium-bottle-jincao-solution")?,
            ItemName::new("Ferrium Bottle (Jincao Solution)")?,
        ),
        Item::new(
            ItemId::new("ferrium-bottle-yazhen-solution")?,
            ItemName::new("Ferrium Bottle (Yazhen Solution)")?,
        ),
        Item::new(
            ItemId::new("ferrium-bottle-liquid-xiranite")?,
            ItemName::new("Ferrium Bottle (Liquid Xiranite)")?,
        ),
        Item::new(
            ItemId::new("ferrium-bottle-liquid-heavy-xiranite")?,
            ItemName::new("Ferrium Bottle (Liquid Heavy Xiranite)")?,
        ),
        Item::new(
            ItemId::new("cuprium-bottle-jincao-solution")?,
            ItemName::new("Cuprium Bottle (Jincao Solution)")?,
        ),
        Item::new(
            ItemId::new("cuprium-bottle-yazhen-solution")?,
            ItemName::new("Cuprium Bottle (Yazhen Solution)")?,
        ),
        // Usable Items
        Item::new(
            ItemId::new("industrial-explosive")?,
            ItemName::new("Industrial Explosive")?,
        ),
        Item::new(
            ItemId::new("buckflower-powder")?,
            ItemName::new("Buckflower Powder")?,
        ),
        Item::new(
            ItemId::new("citrome-powder")?,
            ItemName::new("Citrome Powder")?,
        ),
        Item::new(
            ItemId::new("jincao-powder")?,
            ItemName::new("Jincao Powder")?,
        ),
        Item::new(
            ItemId::new("yazhen-powder")?,
            ItemName::new("Yazhen Powder")?,
        ),
        Item::new(
            ItemId::new("buck-capsule-c")?,
            ItemName::new("Buck Capsule [C]")?,
        ),
        Item::new(
            ItemId::new("buck-capsule-b")?,
            ItemName::new("Buck Capsule [B]")?,
        ),
        Item::new(
            ItemId::new("canned-citrome-c")?,
            ItemName::new("Canned Citrome [C]")?,
        ),
        Item::new(
            ItemId::new("canned-citrome-b")?,
            ItemName::new("Canned Citrome [B]")?,
        ),
        Item::new(
            ItemId::new("jincao-drink")?,
            ItemName::new("Jincao Drink")?,
        ),
        Item::new(
            ItemId::new("yazhen-syringe-c")?,
            ItemName::new("Yazhen Syringe [C]")?,
        ),
        Item::new(
            ItemId::new("buck-capsule-a")?,
            ItemName::new("Buck Capsule [A]")?,
        ),
        Item::new(
            ItemId::new("canned-citrome-a")?,
            ItemName::new("Canned Citrome [A]")?,
        ),
        Item::new(
            ItemId::new("jincao-tea")?,
            ItemName::new("Jincao Tea")?,
        ),
        Item::new(
            ItemId::new("yazhen-syringe-a")?,
            ItemName::new("Yazhen Syringe [A]")?,
        ),
    ])
}
