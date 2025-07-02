use leptos::prelude::*;
use once_cell::sync::Lazy;
use reactive_stores::Store;
use std::collections::HashMap;
use std::sync::Arc;
use crate::model::{recipe::Recipe, AppStore, AppStoreStoreFields, ItemAnalysis};
use leptos::prelude::RwSignal;
use json5;

static FALLBACK_ITEM_ANALYSIS: Lazy<ItemAnalysis> = Lazy::new(|| ItemAnalysis {
    wp: f64::INFINITY,
    power: f64::INFINITY,
    recipes_analysis: vec![],
});

#[derive(Clone, Copy, PartialEq, Eq)]
enum SortColumn {
    Item,
    WeightPoint,
    Power,
}

#[component]
pub fn AnalysisTab() -> impl IntoView {
    let store = use_context::<Store<AppStore>>().expect("AppStore context");
    let recipes = store.recipes().get_untracked();
    // Map from output item to Vec<Recipe>
    let mut recipes_by_output: HashMap<String, Vec<Recipe>> = HashMap::new();
    for recipe in recipes.iter() {
        for output in &recipe.outputs {
            recipes_by_output.entry(output.item.clone()).or_default().push(recipe.clone());
        }
    }
    // Sorting state
    let sort_column = RwSignal::new(SortColumn::Item);
    let sort_desc = RwSignal::new(false);

    // Load item analysis from JSON file at compile time using include_str!
    let item_analysis: Arc<HashMap<String, ItemAnalysis>> = Arc::new({
        let json_str = include_str!("../../assets/satisfactory_item_analysis.json5");
        json5::from_str(json_str).expect("Failed to parse satisfactory_item_analysis.json5")
    });
    let sort_column_memo = sort_column.clone();
    let sort_desc_memo = sort_desc.clone();
    let item_analysis_memo = item_analysis.clone();
    let sorted_items = Memo::new(move |_| {
        let sort_column = sort_column_memo.clone();
        let sort_desc = sort_desc_memo.clone();
        let item_analysis = item_analysis_memo.clone();
        let mut items: Vec<_> = item_analysis.keys().cloned().collect();
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
                    let wa = item_analysis.get(a).map(|ia| ia.wp).unwrap_or(f64::INFINITY);
                    let wb = item_analysis.get(b).map(|ia| ia.wp).unwrap_or(f64::INFINITY);
                    if wa == wb {
                        a.cmp(b)
                    } else if sort_desc.get() {
                        wb.partial_cmp(&wa).unwrap_or(std::cmp::Ordering::Equal)
                    } else {
                        wa.partial_cmp(&wb).unwrap_or(std::cmp::Ordering::Equal)
                    }
                });
            }
            SortColumn::Power => {
                items.sort_by(|a, b| {
                    let pa = item_analysis.get(a).map(|ia| ia.power).unwrap_or(f64::INFINITY);
                    let pb = item_analysis.get(b).map(|ia| ia.power).unwrap_or(f64::INFINITY);
                    if pa == pb {
                        a.cmp(b)
                    } else if sort_desc.get() {
                        pb.partial_cmp(&pa).unwrap_or(std::cmp::Ordering::Equal)
                    } else {
                        pa.partial_cmp(&pb).unwrap_or(std::cmp::Ordering::Equal)
                    }
                });
            }
        }
        items
    });

    let on_sort = move |col: SortColumn| {
        if sort_column.get() == col {
            sort_desc.set(!sort_desc.get());
        } else {
            sort_column.set(col);
            sort_desc.set(false);
        }
    };

    // Dialog state for details
    let dialog_open = RwSignal::new(false);
    let dialog_inputs = RwSignal::new(Vec::new());

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
                        <th style="cursor:pointer" on:click=move |_| on_sort(SortColumn::Power)>
                            Power (MJ) {move || if sort_column.get() == SortColumn::Power { if sort_desc.get() { "▼" } else { "▲" } } else { "" }}
                        </th>
                        <th>Recipes</th>
                    </tr>
                </thead>
                <tbody>
                    <For
                        each=move || sorted_items.get()
                        key=|item| item.clone()
                        children={
                            let item_analysis = item_analysis.clone();
                            let dialog_open = dialog_open.clone();
                            let dialog_inputs = dialog_inputs.clone();
                            move |item| {
                                let analysis = item_analysis.get(&item).unwrap_or(&FALLBACK_ITEM_ANALYSIS);
                                let recipes_analysis = analysis.recipes_analysis.clone();
                                let item = item.clone();
                                Some(view! {
                                    <tr>
                                        <td>{item}</td>
                                        <td>{if analysis.wp == f64::INFINITY { "-".to_string() } else { format!("{:.2}", analysis.wp) }}</td>
                                        <td>{if analysis.power == f64::INFINITY { "-".to_string() } else { format!("{:.2}", analysis.power) }}</td>
                                        <td>
                                            <table class="table table-compact w-full border">
                                                <thead>
                                                    <tr>
                                                        <th>Recipe</th>
                                                        <th>WP</th>
                                                        <th>Power (MJ)</th>
                                                        <th>Rate (/min)</th>
                                                        <th>WP Rate</th>
                                                        <th>Details</th>
                                                    </tr>
                                                </thead>
                                                <tbody>
                                                    {recipes_analysis.into_iter().map({
                                                        let dialog_open = dialog_open.clone();
                                                        let dialog_inputs = dialog_inputs.clone();
                                                        move |recipe| {
                                                            let recipe_name = recipe.recipe_name.clone();
                                                            let wp = recipe.wp;
                                                            let power = recipe.power;
                                                            let rate = recipe.rate;
                                                            let wp_flow = recipe.wp_flow;
                                                            let inputs = recipe.inputs.clone();
                                                            view! {
                                                                <tr>
                                                                    <td>{recipe_name}</td>
                                                                    <td>{format!("{:.2}", wp)}</td>
                                                                    <td>{format!("{:.2}", power)}</td>
                                                                    <td>{format!("{:.4}", rate)}</td>
                                                                    <td>{format!("{:.4}", wp_flow)}</td>
                                                                    <td>
                                                                        <button class="btn btn-xs" on:click=move |_| {
                                                                            dialog_inputs.set(inputs.clone());
                                                                            dialog_open.set(true);
                                                                        }>
                                                                            Details
                                                                        </button>
                                                                    </td>
                                                                </tr>
                                                            }
                                                        }
                                                    }).collect::<Vec<_>>()}
                                                </tbody>
                                            </table>
                                        </td>
                                    </tr>
                                })
                            }
                        }
                    />
                </tbody>
            </table>
            // DaisyUI dialog
            {move || if dialog_open.get() {
                view! {
                    <div class="modal modal-open">
                        <div class="modal-box">
                            <h3 class="font-bold text-lg">Recipe Details</h3>
                            <table class="table table-compact w-full border">
                                <thead>
                                    <tr>
                                        <th>Qty</th>
                                        <th>Item</th>
                                        <th>WP</th>
                                        <th>Power</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {dialog_inputs.get().into_iter().map(|input| view! {
                                        <tr>
                                            <td>{format!("{:.2}", input.quantity)}</td>
                                            <td>{input.item.clone()}</td>
                                            <td>{format!("{:.2}", input.wp_per_item)}</td>
                                            <td>{format!("{:.2}", input.power_per_item)}</td>
                                        </tr>
                                    }).collect::<Vec<_>>()}
                                </tbody>
                            </table>
                            <div class="modal-action">
                                <button class="btn" on:click=move |_| dialog_open.set(false)>Close</button>
                            </div>
                        </div>
                    </div>
                }.into_any()
            } else {
                view! { <div class="modal"></div> }.into_any()
            }}
        </div>
    }
}
