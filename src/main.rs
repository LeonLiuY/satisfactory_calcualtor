use leptos::{logging::log, mount::mount_to_body, prelude::*};
pub mod adapters;
mod components;
mod model;

use adapters::satisfactory_adapter::{build_machine_power_map_from_assets, load_satisfactory_recipes_from_json};
use components::factory_planner_app::FactoryPlannerApp;
use reactive_stores::Store;

use crate::model::recipe::Recipe;
use crate::adapters::satisfactory_asset::SatisfactoryAsset;

#[derive(Default, Store, Clone)]
struct AppStore {
    #[store(key: String = |recipe| recipe.name.clone())]
    recipes: Vec<Recipe>,
    #[store]
    machine_power_map: std::collections::HashMap<String, f64>,
}

#[component]
fn App() -> impl IntoView {
    let assets: Vec<SatisfactoryAsset> = serde_json::from_str(include_str!("../assets/satisfactory_en-US.json")).unwrap_or_default();
    let recipes = load_satisfactory_recipes_from_json(include_str!("../assets/satisfactory_en-US.json")).unwrap_or_default();
    let machine_power_map = build_machine_power_map_from_assets(&assets);
    let store = Store::new(AppStore {
        recipes,
        machine_power_map,
    });
    provide_context(store);
    let enabled_recipes = Memo::new(move |_| {
        log!("Calculating enabled recipes");
        store
            .recipes()
            .into_iter()
            .filter(|r| r.get().enabled)
            .map(|r| r.get().name)
            .collect::<std::collections::HashSet<_>>()
    });
    provide_context(enabled_recipes);
    view! { <FactoryPlannerApp /> }
}

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(App);
}
