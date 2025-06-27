use leptos::{mount::mount_to_body, prelude::*};
mod recipe;
pub mod adapters;
mod components;

use adapters::satisfactory_adapter::load_satisfactory_recipes_from_json;
use components::factory_planner_app::FactoryPlannerApp;
use reactive_stores::Store;

use crate::recipe::Recipe;

#[derive(Default, Store, Clone)]
struct AppStore {
    #[store(key: String = |recipe| recipe.name.clone())]
    recipes: Vec<Recipe>,
}

#[component]
fn App() -> impl IntoView {
    let recipes = load_satisfactory_recipes_from_json(include_str!("../assets/satisfactory_recipes.json")).unwrap_or_default();
    let store = Store::new(AppStore {
        recipes: recipes,
    });
    provide_context(store);
    let enabled_recipes = Memo::new(move |_| {
        store.recipes().into_iter()
            .filter(|r| r.get().enabled)
            .map(|r| r.get().name )
            .collect::<std::collections::HashSet<_>>()
    });
    provide_context(enabled_recipes);
    view! {
        <FactoryPlannerApp />
    }
}

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(App);
}