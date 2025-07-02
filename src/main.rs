use factory_planner::{adapters::satisfactory_adapter::load_satisfactory_recipes_from_json, components::factory_planner_app::FactoryPlannerApp, model::{AppStore, AppStoreStoreFields}};
use leptos::{logging::log, mount::mount_to_body, prelude::*};

use reactive_stores::Store;

#[component]
fn App() -> impl IntoView {
    let recipes = load_satisfactory_recipes_from_json(include_str!("../assets/satisfactory_en-US.json")).unwrap_or_default();
    let store = Store::new(AppStore {
        recipes,
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
