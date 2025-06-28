use leptos::prelude::*;
use std::collections::HashMap;
use crate::model::{recipe::Recipe, compute_item_weight_points};
use crate::Store;
use crate::AppStoreStoreFields;

#[derive(Clone, Copy, PartialEq, Eq)]
enum SortColumn {
    Item,
    WeightPoint,
}

#[component]
pub fn AnalysisTab() -> impl IntoView {
    let store = use_context::<Store<crate::AppStore>>().expect("AppStore context");
    let recipes = store.recipes().get_untracked();
    let item_weights = compute_item_weight_points(&recipes);
    let mut recipes_by_output: HashMap<String, Vec<Recipe>> = HashMap::new();
    for recipe in &recipes {
        for output in &recipe.outputs {
            recipes_by_output.entry(output.item.clone()).or_default().push(recipe.clone());
        }
    }
    // Sorting state
    let sort_column = RwSignal::new(SortColumn::Item);
    let sort_desc = RwSignal::new(false);

    let item_weights1 = item_weights.clone();
    // Use Arc instead of Rc for thread safety in Memo
    let sorted_items = Memo::new(
        move |_| {
            let mut items: Vec<_> = item_weights1.keys().cloned().collect();
            match sort_column.get() {
                SortColumn::Item => {
                    items.sort_by(|a, b| {
                        if sort_desc.get() {
                            b.cmp(a)
                        } else {
                            a.cmp(b)
                        }
                    });
                }
                SortColumn::WeightPoint => {
                    items.sort_by(|a, b| {
                        let wa = item_weights1.get(a).unwrap_or(&f64::INFINITY);
                        let wb = item_weights1.get(b).unwrap_or(&f64::INFINITY);
                        if wa == wb {
                            a.cmp(b)
                        } else if sort_desc.get() {
                            wb.partial_cmp(&wa).unwrap_or(std::cmp::Ordering::Equal)
                        } else {
                            wa.partial_cmp(&wb).unwrap_or(std::cmp::Ordering::Equal)
                        }
                    });
                }
            }
            items
        }
    );

    let on_sort = move |col: SortColumn| {
        if sort_column.get() == col {
            sort_desc.set(!sort_desc.get());
        } else {
            sort_column.set(col);
            sort_desc.set(false);
        }
    };

    view! {
        <div class="p-4">
            <h2 class="text-xl font-bold mb-4">Recipe Analysis</h2>
            <table class="table table-zebra w-full">
                <thead>
                    <tr>
                        <th style="cursor:pointer" on:click=move |_| on_sort(SortColumn::Item)>
                            Item {move || if sort_column.get() == SortColumn::Item { if sort_desc.get() { "▼" } else { "▲" } } else { "" }}
                        </th>
                        <th style="cursor:pointer" on:click=move |_| on_sort(SortColumn::WeightPoint)>
                            Weight Point {move || if sort_column.get() == SortColumn::WeightPoint { if sort_desc.get() { "▼" } else { "▲" } } else { "" }}
                        </th>
                        <th>Recipes</th>
                    </tr>
                </thead>
                <tbody>
                    <For
                        each=move || sorted_items.get()
                        key=|item| item.clone()
                        children={move |item| {
                            let recipes = match recipes_by_output.get(&item) {
                                Some(r) => r,
                                None => return None,
                            };
                            let min_recipe_wp = item_weights.get(&item).cloned().unwrap_or(f64::INFINITY);
                            Some(view! {
                                <tr>
                                    <td>{item.clone()}</td>
                                    <td>{if min_recipe_wp == f64::INFINITY { "-".to_string() } else { format!("{:.2}", min_recipe_wp) }}</td>
                                    <td>
                                        <ul>
                                            {recipes.iter().filter_map(|r| {
                                                let out_qty = r.outputs.iter().find(|o| o.item == item).map(|o| o.quantity as f64).unwrap_or(1.0);
                                                if out_qty == 0.0 || r.outputs.iter().find(|o| o.item == item).is_none() {
                                                    return None;
                                                }
                                                let mut total_weight = 0.0;
                                                let mut all_known = true;
                                                let input_info = r.inputs.iter().map(|input| {
                                                    let per_output = input.quantity as f64 / out_qty;
                                                    let wp = item_weights.get(&input.item).cloned().unwrap_or(f64::INFINITY);
                                                    if wp == f64::INFINITY {
                                                        all_known = false;
                                                    }
                                                    total_weight += wp * per_output;
                                                    format!("{:.2}x {} (WP={:.2})", per_output, input.item, wp)
                                                }).collect::<Vec<_>>().join(", ");
                                                Some(view! {
                                                    <li>
                                                        <span class="font-semibold">{r.name.clone()}</span>
                                                        {": "}
                                                        {if all_known { format!("{:.2}", total_weight) } else { "-".to_string() }}
                                                        {" ("}
                                                        {input_info}
                                                        {")"}
                                                    </li>
                                                })
                                            }).collect::<Vec<_>>()}
                                        </ul>
                                    </td>
                                </tr>
                            })
                        }}
                    />
                </tbody>
            </table>
        </div>
    }
}
