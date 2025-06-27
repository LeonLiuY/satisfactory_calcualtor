use leptos::prelude::*;
use crate::Store;
use crate::AppStoreStoreFields;

#[component]
pub fn RecipesTab() -> impl IntoView {
    let store = use_context::<Store<crate::AppStore>>().expect("AppStore context");
    let recipe_output_filter = RwSignal::new(String::new());
    let set_recipe_output_filter = recipe_output_filter.write_only();
    view! {
        <div class="mb-4">
            <input
                class="input input-bordered w-full max-w-xs"
                type="text"
                placeholder="Filter by output product..."
                value=move || recipe_output_filter.get()
                on:input=move |ev| {
                    set_recipe_output_filter.set(event_target_value(&ev))
                }
            />
        </div>
        <div class="overflow-x-auto">
            <table class="table table-zebra w-full">
                <thead>
                    <tr>
                        <th class="w-20">Enable</th>
                        <th class="w-64">Recipe Name</th>
                        <th class="w-64">Outputs</th>
                        <th class="w-64">Inputs</th>
                    </tr>
                </thead>
                <tbody>
                    <For
                        each=move || {
                            let filter = recipe_output_filter.get().to_lowercase();
                            if filter.is_empty() {
                                store.recipes().into_iter().collect::<Vec<_>>()
                            } else {
                                store.recipes().into_iter()
                                    .filter(|r| r.get().outputs.iter().any(|o| o.item.to_lowercase().contains(&filter)))
                                    .collect::<Vec<_>>()
                            }
                        }
                        key=|r| r.get().name.clone()
                        children=move |r| {
                            let name = r.get().name;
                            let outputs = r.get().outputs.iter().map(|o| o.item.clone()).collect::<Vec<_>>().join(", ");
                            let inputs = r.get().inputs.iter().map(|i| i.item.clone()).collect::<Vec<_>>().join(", ");
                            let r1 = r.clone();
                            let r2 = r.clone();
                            view! {
                                <tr>
                                    <td>
                                        <input
                                            type="checkbox"
                                            class="checkbox"
                                            checked=move || r1.get().enabled
                                            on:input=move |ev| {
                                                let checked = event_target_checked(&ev);
                                                r2.write().enabled = checked;
                                            }
                                        />
                                    </td>
                                    <td>{name}</td>
                                    <td>{outputs}</td>
                                    <td>{inputs}</td>
                                </tr>
                                <tr class="bg-base-200">
                                    <td colspan="4" class="p-0">
                                        <div class="collapse collapse-arrow">
                                            <input type="checkbox" class="peer" />
                                            <div class="collapse-title text-md font-medium">
                                                Details
                                            </div>
                                            <div class="collapse-content">
                                                <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                                                    <div>
                                                        <div class="font-semibold mb-1">Outputs</div>
                                                        <table class="table table-xs w-full">
                                                            <thead>
                                                                <tr>
                                                                    <th>Product</th>
                                                                    <th>Per Craft</th>
                                                                    <th>Rate/min</th>
                                                                </tr>
                                                            </thead>
                                                            <tbody>
                                                                {r.get().outputs.iter().map(|o| {
                                                                    let rate = (o.quantity as f64) * (60_000.0 / r.get().time as f64);
                                                                    view! {
                                                                        <tr>
                                                                            <td>{o.item.clone()}</td>
                                                                            <td>{o.quantity}</td>
                                                                            <td>{format!("{:.2}", rate)}</td>
                                                                        </tr>
                                                                    }
                                                                }).collect::<Vec<_>>()}
                                                            </tbody>
                                                        </table>
                                                    </div>
                                                    <div>
                                                        <div class="font-semibold mb-1">Inputs</div>
                                                        <table class="table table-xs w-full">
                                                            <thead>
                                                                <tr>
                                                                    <th>Product</th>
                                                                    <th>Per Craft</th>
                                                                    <th>Rate/min</th>
                                                                </tr>
                                                            </thead>
                                                            <tbody>
                                                                {r.get().inputs.iter().map(|i| {
                                                                    let rate = (i.quantity as f64) * (60_000.0 / r.get().time as f64);
                                                                    view! {
                                                                        <tr>
                                                                            <td>{i.item.clone()}</td>
                                                                            <td>{i.quantity}</td>
                                                                            <td>{format!("{:.2}", rate)}</td>
                                                                        </tr>
                                                                    }
                                                                }).collect::<Vec<_>>()}
                                                            </tbody>
                                                        </table>
                                                    </div>
                                                </div>
                                                <div class="mt-4 flex flex-wrap gap-8">
                                                    <div>
                                                        <span class="font-semibold">Machine:</span>
                                                        {r.get().machine.name.clone()}
                                                    </div>
                                                    <div>
                                                        <span class="font-semibold">Craft Time:</span>
                                                        {format!("{:.2} ms", r.get().time)}
                                                    </div>
                                                </div>
                                            </div>
                                        </div>
                                    </td>
                                </tr>
                            }
                        }
                    />
                </tbody>
            </table>
        </div>
    }
}
