pub mod recipe;

use crate::model::recipe::Recipe;

/// An empty function for testing purposes.
pub fn empty_function() {}

/// Raw resource availability and WP assignment (copied from analysis_tab.rs)
pub const RESOURCE_AVAIL: [(&str, f64); 15] = [
    ("Bauxite", 12300.0),
    ("Caterium Ore", 15000.0),
    ("Coal", 42300.0),
    ("Copper Ore", 36900.0),
    ("Crude Oil", 12600.0),
    ("Iron Ore", 92100.0),
    ("Limestone", 69900.0),
    ("Nitrogen Gas", 12000.0),
    ("Raw Quartz", 13500.0),
    ("Sulfur", 10800.0),
    ("Uranium", 2100.0),
    ("SAM", 10200.0),
    ("Water", f64::INFINITY),
    ("Excited Photonic Matter", f64::INFINITY),
    ("Dark Matter Residue", f64::INFINITY),
];

pub fn resource_weight_points() -> Vec<(String, f64)> {
    let most_common = RESOURCE_AVAIL.iter()
        .filter(|(_, avail)| avail.is_finite())
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
        .unwrap().1;

    RESOURCE_AVAIL.iter()
        .map(|(name, avail)| (name.to_string(), most_common / *avail))
        .collect()
}

/// Compute minimal WP for all items using fixed-point iteration (handles cycles)
pub fn compute_item_weight_points(recipes: &[Recipe]) -> std::collections::HashMap<String, f64> {
    use std::collections::{HashMap, HashSet};
    let resource_weights = resource_weight_points();
    println!("Resource weights: {:?}", resource_weights);
    let raw_resource_names: HashSet<String> = RESOURCE_AVAIL.iter().map(|(name, _)| name.to_string()).collect();
    // Initialize WP for all raw resources
    let mut item_weights: HashMap<String, f64> = HashMap::new();
    for (name, wp) in &resource_weights {
        item_weights.insert(name.clone(), *wp);
    }
    // Also, for any item that is only an input and never an output, assign WP (raw)
    let all_outputs: HashSet<_> = recipes.iter().flat_map(|r| r.outputs.iter().map(|o| o.item.clone())).collect();
    let all_inputs: HashSet<_> = recipes.iter().flat_map(|r| r.inputs.iter().map(|i| i.item.clone())).collect();
    for item in all_inputs.difference(&all_outputs) {
        let wp = resource_weights.iter().find_map(|(name, wp)| {
            if item == name {
                Some(*wp)
            } else {
                None
            }
        }).unwrap_or(f64::INFINITY);
        item_weights.insert(item.clone(), wp);
    }
    // Fixed-point iteration: propagate values through recipes until convergence
    let mut changed = true;
    let threshold = 1e-6;
    while changed {
        changed = false;
        for recipe in recipes {
            // Do not skip any recipe, but do not update WP for raw resources
            for output in &recipe.outputs {
                // Only update WP if output is not a raw resource
                if raw_resource_names.contains(&output.item) {
                    continue;
                }
                let out_qty = output.quantity as f64;
                let mut total_weight = 0.0;
                let mut all_known = true;
                for input in &recipe.inputs {
                    if let Some(w) = item_weights.get(&input.item) {
                        total_weight += w * input.quantity as f64 / out_qty;
                    } else {
                        all_known = false;
                        break;
                    }
                }
                if all_known {
                    let entry = item_weights.entry(output.item.clone()).or_insert(f64::INFINITY);
                    if total_weight + threshold < *entry {
                        *entry = total_weight;
                        changed = true;
                    }
                }
            }
        }
    }
    // Remove all items with infinite WP
    item_weights.retain(|_, &mut wp| wp != f64::INFINITY);
    item_weights
}

#[cfg(test)]
mod tests {
    use crate::adapters::satisfactory_adapter::load_satisfactory_recipes_from_json;

    use super::*;

    #[test]
    fn test_empty_function() {
        empty_function();
        // No assertion needed for an empty function
    }

    #[test]
    fn test_print_item_weight_points() {
        let path = "assets/satisfactory_en-US.json";
        let json_str = std::fs::read_to_string(path).expect("Failed to read JSON file");
        let recipes = load_satisfactory_recipes_from_json(&json_str).expect("Failed to load recipes");
        recipes.iter().for_each(|r| {
            println!("Recipe: {}", r.name);
            for input in &r.inputs {
                println!("  Input: {} x {}", input.item, input.quantity);
            }
            for output in &r.outputs {
                println!("  Output: {} x {}", output.item, output.quantity);
            }
        });
        let item_weights = compute_item_weight_points(&recipes);
        let mut items: Vec<_> = item_weights.iter().collect();
        items.sort_by(|a, b| a.0.cmp(b.0));
        println!("{:<32} | {:>10}", "Item", "WP");
        println!("{}", "-".repeat(45));
        for (item, wp) in items {
            if *wp == f64::INFINITY {
                println!("{:<32} | {:>10}", item, "-∞");
            } else {
                println!("{:<32} | {:>10.4}", item, wp);
            }
        }
    }
}