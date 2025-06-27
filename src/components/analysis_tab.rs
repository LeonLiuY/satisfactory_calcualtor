use leptos::prelude::*;
use std::collections::{HashMap, HashSet};
use crate::recipe::Recipe;
use crate::Store;
use crate::AppStoreStoreFields;

#[component]
pub fn AnalysisTab() -> impl IntoView {
    let store = use_context::<Store<crate::AppStore>>().expect("AppStore context");
    let recipes = store.recipes().get();
    let mut recipes_by_output: HashMap<String, Vec<Recipe>> = HashMap::new();
    for recipe in &recipes {
        for output in &recipe.outputs {
            recipes_by_output.entry(output.item.clone()).or_default().push(recipe.clone());
        }
    }
    let all_outputs: HashSet<_> = recipes.iter().flat_map(|r| r.outputs.iter().map(|o| o.item.clone())).collect();
    let all_inputs: HashSet<_> = recipes.iter().flat_map(|r| r.inputs.iter().map(|i| i.item.clone())).collect();
    let mut raw_weights = HashMap::new();
    let resource_weights = resource_weight_points();
    let raw_resource_names: HashSet<String> = RESOURCE_AVAIL.iter().map(|(name, _)| name.to_string()).collect();
    for item in all_inputs.difference(&all_outputs) {
        let wp = resource_weights.iter().find_map(|(name, wp)| {
            if item == name {
                Some(*wp)
            } else {
                None
            }
        }).unwrap_or(f64::INFINITY);
        raw_weights.insert(item.clone(), wp);
    }
    // Force set WP for all raw resources, even if they are not only inputs
    for (name, wp) in &resource_weights {
        raw_weights.insert(name.clone(), *wp);
    }
    let item_weights = calculate_item_weights_force_raw(&recipes, &raw_weights, &raw_resource_names);
    // After calculating item_weights, patch in the min WP for each item from all recipes (if any recipe can be made)
    let mut patched_item_weights = item_weights.clone();
    for (item, recipes) in &recipes_by_output {
        let min_recipe_wp = recipes.iter().filter_map(|r| {
            let out_qty = r.outputs.iter().find(|o| o.item == *item).map(|o| o.quantity as f64).unwrap_or(1.0);
            let mut total_weight = 0.0;
            let mut all_known = true;
            for input in &r.inputs {
                let per_output = input.quantity as f64 / out_qty;
                let wp = item_weights.get(&input.item).cloned().unwrap_or(f64::INFINITY);
                if wp == f64::INFINITY {
                    all_known = false;
                    break;
                }
                total_weight += wp * per_output;
            }
            if all_known { Some(total_weight) } else { None }
        }).fold(f64::INFINITY, |a, b| a.min(b));
        if min_recipe_wp < f64::INFINITY {
            patched_item_weights.insert(item.clone(), min_recipe_wp);
        }
    }
    let item_weights = patched_item_weights;
    // Topological sort of items for WP calculation and display
    let mut in_degree: HashMap<String, usize> = HashMap::new();
    let mut dependents: HashMap<String, Vec<String>> = HashMap::new();
    for recipe in &recipes {
        for output in &recipe.outputs {
            let entry = in_degree.entry(output.item.clone()).or_insert(0);
            for input in &recipe.inputs {
                *entry += 1;
                dependents.entry(input.item.clone()).or_default().push(output.item.clone());
            }
        }
    }
    // Add all items that are only inputs (raw resources) with in_degree 0
    for item in all_inputs.iter() {
        in_degree.entry(item.clone()).or_insert(0);
    }
    // Kahn's algorithm for topological sort
    let mut queue: Vec<String> = in_degree.iter().filter(|(_, deg)| **deg == 0).map(|(k, _)| k.clone()).collect();
    let mut topo_order = Vec::new();
    let mut in_degree = in_degree;
    while let Some(item) = queue.pop() {
        topo_order.push(item.clone());
        if let Some(deps) = dependents.get(&item) {
            for dep in deps {
                if let Some(deg) = in_degree.get_mut(dep) {
                    *deg -= 1;
                    if *deg == 0 {
                        queue.push(dep.clone());
                    }
                }
            }
        }
    }
    // Calculate WP in topological order
    let mut item_weights = raw_weights.clone();
    for item in &topo_order {
        // Skip if already set (raw resource)
        if item_weights.contains_key(item) {
            continue;
        }
        if let Some(recipes) = recipes_by_output.get(item) {
            let mut min_wp = f64::INFINITY;
            for r in recipes {
                let out_qty = r.outputs.iter().find(|o| o.item == *item).map(|o| o.quantity as f64).unwrap_or(1.0);
                let mut total_weight = 0.0;
                let mut all_known = true;
                for input in &r.inputs {
                    let per_output = input.quantity as f64 / out_qty;
                    let wp = item_weights.get(&input.item).cloned().unwrap_or(f64::INFINITY);
                    if wp == f64::INFINITY {
                        all_known = false;
                        break;
                    }
                    total_weight += wp * per_output;
                }
                if all_known && total_weight < min_wp {
                    min_wp = total_weight;
                }
            }
            if min_wp < f64::INFINITY {
                item_weights.insert(item.clone(), min_wp);
            }
        }
    }
    // Use topological order for display
    let all_items = topo_order;
    view! {
        <div class="p-4">
            <h2 class="text-xl font-bold mb-4">Recipe Analysis</h2>
            <table class="table table-zebra w-full">
                <thead>
                    <tr>
                        <th>Item</th>
                        <th>Weight Point</th>
                        <th>Recipes</th>
                    </tr>
                </thead>
                <tbody>
                    {all_items.iter().filter_map(|item| {
                        let recipes = match recipes_by_output.get(item) {
                            Some(r) => r,
                            None => return None,
                        };
                        // Compute the minimum WP for this item across all recipes
                        let min_recipe_wp = recipes.iter().filter_map(|r| {
                            let out_qty = r.outputs.iter().find(|o| o.item == *item).map(|o| o.quantity as f64).unwrap_or(1.0);
                            if out_qty == 0.0 || r.outputs.iter().find(|o| o.item == *item).is_none() {
                                return None;
                            }
                            let mut total_weight = 0.0;
                            let mut all_known = true;
                            for input in &r.inputs {
                                let per_output = input.quantity as f64 / out_qty;
                                let wp = item_weights.get(&input.item).cloned().unwrap_or(f64::INFINITY);
                                if wp == f64::INFINITY {
                                    all_known = false;
                                    break;
                                }
                                total_weight += wp * per_output;
                            }
                            if all_known { Some(total_weight) } else { None }
                        }).fold(f64::INFINITY, |a, b| a.min(b));
                        Some(view! {
                            <tr>
                                <td>{item.clone()}</td>
                                <td>{if min_recipe_wp == f64::INFINITY { "-".to_string() } else { format!("{:.2}", min_recipe_wp) }}</td>
                                <td>
                                    <ul>
                                        {recipes.iter().filter_map(|r| {
                                            let out_qty = r.outputs.iter().find(|o| o.item == *item).map(|o| o.quantity as f64).unwrap_or(1.0);
                                            if out_qty == 0.0 || r.outputs.iter().find(|o| o.item == *item).is_none() {
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
                    }).collect::<Vec<_>>()}

                </tbody>
            </table>
        </div>
    }
}

// Utility for resource WP and item weight calculation (shared with main app)
pub const RESOURCE_AVAIL: [(&str, f64); 13] = [
    ("OreBauxite", 12300.0),
    ("OreGold", 15000.0),
    ("Coal", 42300.0),
    ("OreCopper", 36900.0),
    ("LiquidOil", 12600.0),
    ("OreIron", 92100.0),
    ("Stone", 69900.0),
    ("NitrogenGas", 12000.0),
    ("RawQuartz", 13500.0),
    ("Sulfur", 10800.0),
    ("OreUranium", 2100.0),
    ("SAM", 10200.0),
    ("Water", f64::INFINITY),
];

pub fn resource_weight_points() -> Vec<(String, f64)> {
    let iron_avail = RESOURCE_AVAIL.iter().find(|(n, _)| *n == "OreIron").map(|(_, v)| *v).unwrap_or(1.0);
    RESOURCE_AVAIL.iter().map(|(name, avail)| {
        if *name == "Water" {
            (name.to_string(), 0.0)
        } else {
            (name.to_string(), iron_avail / *avail)
        }
    }).collect()
}

pub fn calculate_item_weights(recipes: &[Recipe], raw_weights: &std::collections::HashMap<String, f64>) -> std::collections::HashMap<String, f64> {
    let mut item_weights: std::collections::HashMap<String, f64> = raw_weights.clone();
    let mut changed = true;
    while changed {
        changed = false;
        for recipe in recipes {
            for output in &recipe.outputs {
                let mut total_weight = 0.0;
                let mut all_known = true;
                for input in &recipe.inputs {
                    if let Some(w) = item_weights.get(&input.item) {
                        total_weight += w * input.quantity as f64 / output.quantity as f64;
                    } else {
                        all_known = false;
                        break;
                    }
                }
                if all_known {
                    let entry = item_weights.entry(output.item.clone()).or_insert(f64::INFINITY);
                    if total_weight < *entry {
                        *entry = total_weight;
                        changed = true;
                    }
                }
            }
        }
    }
    item_weights
}

pub fn calculate_item_weights_force_raw(
    recipes: &[Recipe],
    raw_weights: &std::collections::HashMap<String, f64>,
    raw_resource_names: &std::collections::HashSet<String>,
) -> std::collections::HashMap<String, f64> {
    let mut item_weights: std::collections::HashMap<String, f64> = raw_weights.clone();
    let mut changed = true;
    while changed {
        changed = false;
        for recipe in recipes {
            // If any output is a forced raw resource, skip this recipe for WP calculation
            if recipe.outputs.iter().any(|o| raw_resource_names.contains(&o.item)) {
                continue;
            }
            for output in &recipe.outputs {
                let mut total_weight = 0.0;
                let mut all_known = true;
                for input in &recipe.inputs {
                    if let Some(w) = item_weights.get(&input.item) {
                        total_weight += w * input.quantity as f64 / output.quantity as f64;
                    } else {
                        all_known = false;
                        break;
                    }
                }
                if all_known {
                    let entry = item_weights.entry(output.item.clone()).or_insert(f64::INFINITY);
                    if total_weight < *entry {
                        *entry = total_weight;
                        changed = true;
                    }
                }
            }
        }
    }
    item_weights
}
