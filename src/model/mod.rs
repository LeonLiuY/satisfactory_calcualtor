pub mod recipe;

use std::{io::{self, Write}, vec};

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

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct ItemAnalysis {
    pub wp: f64,
    pub power: f64,
    pub recipes_analysis: Vec<ItemRecipeAnalysis>,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct ItemRecipeAnalysis {
    pub recipe_name: String, // Added recipe name
    pub inputs: Vec<ItemInputAnalysis>,
    pub wp: f64,
    pub power: f64,
    pub rate: f64,
    pub wp_flow: f64,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct ItemInputAnalysis {
    pub item: String,
    pub quantity: f64,
    pub wp_per_item: f64,
    pub power_per_item: f64,
}

/// Compute minimal WP and power (J) for all items using fixed-point iteration (handles cycles)
pub fn compute_item_analysis(
    recipes: &[Recipe],
    machine_power_map: &std::collections::HashMap<String, f64>,
) -> std::collections::HashMap<String, ItemAnalysis> {
    use std::collections::{HashMap, HashSet};
    let resource_weights = resource_weight_points();
    let raw_resource_names: HashSet<String> = RESOURCE_AVAIL.iter().map(|(name, _)| name.to_string()).collect();
    // Initialize for all raw resources
    let mut item_analysis: HashMap<String, ItemAnalysis> = HashMap::new();
    for (name, wp) in &resource_weights {
        item_analysis.insert(name.clone(), ItemAnalysis { wp: *wp, power: 0.0, recipes_analysis: vec![] });
    }
    // Also, for any item that is only an input and never an output, assign WP/power (raw)
    let all_outputs: HashSet<_> = recipes.iter().flat_map(|r| r.outputs.iter().map(|o| o.item.clone())).collect();
    let all_inputs: HashSet<_> = recipes.iter().flat_map(|r| r.inputs.iter().map(|i| i.item.clone())).collect();
    for item in all_inputs.difference(&all_outputs) {
        let wp = resource_weights.iter().find_map(|(name, wp)| {
            if item == name { Some(*wp) } else { None }
        }).unwrap_or(f64::INFINITY);
        item_analysis.insert(item.clone(), ItemAnalysis { wp, power: 0.0, recipes_analysis: vec![] });
    }
    // Fixed-point iteration: propagate values through recipes until convergence
    let mut changed = true;
    let threshold = 1e-6;
    while changed {
        changed = false;
        for recipe in recipes {
            for output in &recipe.outputs {
                if raw_resource_names.contains(&output.item) {
                    continue;
                }
                let out_qty = output.quantity as f64;
                let mut total_wp = 0.0;
                let mut total_power = 0.0;
                let mut all_known = true;
                for input in &recipe.inputs {
                    if let Some(ia) = item_analysis.get(&input.item) {
                        total_wp += ia.wp * input.quantity as f64 / out_qty;
                        total_power += ia.power * input.quantity as f64 / out_qty;
                    } else {
                        all_known = false;
                        break;
                    }
                }
                // Add direct machine power for this recipe
                let machine_power = machine_power_map.get(&recipe.machine.name).cloned().unwrap_or(0.0); // MW
                let time_s = recipe.time as f64 / 1000.0;
                let machine_mj = machine_power * time_s / out_qty; // MW * s = MJ
                if all_known {
                    let total = ItemAnalysis {
                        wp: total_wp,
                        power: total_power + machine_mj,
                        recipes_analysis: vec![], // Will fill below
                    };
                    let entry = item_analysis.entry(output.item.clone()).or_insert(ItemAnalysis { wp: f64::INFINITY, power: f64::INFINITY, recipes_analysis: vec![] });
                    if total.wp + threshold < entry.wp || total.power + threshold < entry.power {
                        if total.wp + threshold < entry.wp {
                            println!("Updating {}: old WP = {:.4}, new WP = {:.4}", output.item, entry.wp, total.wp);
                            entry.wp = total.wp;
                        }
                        if total.power + threshold < entry.power {
                            println!("Updating {}: old Power = {:.2}, new Power = {:.2}", output.item, entry.power, total.power);
                            entry.power = total.power;
                        }
                        changed = true;
                        io::stdout().flush().unwrap(); // Ensure output is flushed
                    }
                }
            }
        }
    }
    // Now, fill in recipes_analysis for each item
    for recipe in recipes {
        for output in &recipe.outputs {
            if raw_resource_names.contains(&output.item) {
                continue;
            }
            let out_qty = output.quantity as f64;
            let mut total_wp = 0.0;
            let mut total_power = 0.0;
            let mut all_known = true;
            let mut inputs_analysis = vec![];
            for input in &recipe.inputs {
                let per_output = input.quantity as f64 / out_qty;
                if let Some(ia) = item_analysis.get(&input.item) {
                    total_wp += ia.wp * per_output;
                    total_power += ia.power * per_output;
                    inputs_analysis.push(ItemInputAnalysis {
                        item: input.item.clone(),
                        quantity: per_output,
                        wp_per_item: ia.wp,
                        power_per_item: ia.power,
                    });
                } else {
                    all_known = false;
                    break;
                }
            }
            let machine_power = machine_power_map.get(&recipe.machine.name).cloned().unwrap_or(0.0); // MW
            let time_s = recipe.time as f64 / 1000.0;
            let machine_mj = machine_power * time_s / out_qty;
            if all_known {
                let recipe_analysis = ItemRecipeAnalysis {
                    recipe_name: recipe.name.clone(), // Fill in recipe name
                    inputs: inputs_analysis,
                    wp: total_wp,
                    wp_flow: item_analysis.get(&output.item).unwrap().wp * out_qty / time_s, // WP flow rate per second
                    rate: out_qty / time_s * 60.0, // Rate per minute
                    power: total_power + machine_mj,
                };
                if let Some(entry) = item_analysis.get_mut(&output.item) {
                    entry.recipes_analysis.push(recipe_analysis);
                }
            }
        }
    }
    // Remove all items with infinite WP or power
    item_analysis.retain(|_, ia| ia.wp != f64::INFINITY && ia.power != f64::INFINITY);
    item_analysis
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
    fn test_print_item_analysis() {
        use crate::adapters::satisfactory_adapter::build_machine_power_map_from_assets;
        let path = "assets/satisfactory_en-US.json";
        let json_str = std::fs::read_to_string(path).expect("Failed to read JSON file");
        let recipes = load_satisfactory_recipes_from_json(&json_str).expect("Failed to load recipes");
        let assets: Vec<crate::adapters::satisfactory_asset::SatisfactoryAsset> = serde_json::from_str(&json_str).expect("Failed to parse JSON");
        let machine_power_map = build_machine_power_map_from_assets(&assets);
        let item_analysis = compute_item_analysis(&recipes, &machine_power_map);
        let mut items: Vec<_> = item_analysis.iter().collect();
        items.sort_by(|a, b| a.0.cmp(b.0));
        println!("{:<32} | {:>10} | {:>15}", "Item", "WP", "Power (J)");
        println!("{}", "-".repeat(60));
        for (item, analysis) in items {
            let wp_str = if analysis.wp == f64::INFINITY { "-∞".to_string() } else { format!("{:.4}", analysis.wp) };
            let power_str = if analysis.power == f64::INFINITY { "-∞".to_string() } else { format!("{:.2}", analysis.power) };
            println!("{:<32} | {:>10} | {:>15}", item, wp_str, power_str);
        }
    }
    #[test]
    fn test_write_item_analysis_json() {
        use crate::adapters::satisfactory_adapter::build_machine_power_map_from_assets;
        let path = "assets/satisfactory_en-US.json";
        let json_str = std::fs::read_to_string(path).expect("Failed to read JSON file");
        let recipes = load_satisfactory_recipes_from_json(&json_str).expect("Failed to load recipes");
        let assets: Vec<crate::adapters::satisfactory_asset::SatisfactoryAsset> = serde_json::from_str(&json_str).expect("Failed to parse JSON");
        let machine_power_map = build_machine_power_map_from_assets(&assets);
        let item_analysis = compute_item_analysis(&recipes, &machine_power_map);
        let json = json5::to_string(&item_analysis).expect("Failed to serialize item analysis to JSON5");
        std::fs::write("assets/satisfactory_item_analysis.json5", json).expect("Failed to write item_analysis.json");
    }
}