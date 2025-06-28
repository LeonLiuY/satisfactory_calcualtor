// Satisfactory adapter module
// (No need to pub use crate::satisfactory_adapter;)

// Satisfactory adapter logic moved from adapters.rs
use crate::model::recipe::{CraftingMachine, ItemStack, Recipe};
use regex;
use serde::Deserialize;
use std::collections::HashMap;

use crate::adapters::satisfactory_asset::SatisfactoryAsset;

#[derive(Debug, Deserialize)]
pub struct SatisfactoryJsonIngredient {
    pub item: String,
    pub amount: f32,
}

#[derive(Debug, Deserialize)]
pub struct SatisfactoryJsonRecipe {
    #[serde(rename = "className")]
    pub class_name: String,
    #[serde(rename = "name")]
    pub display_name: String,
    pub duration: f32,
    pub ingredients: Vec<SatisfactoryJsonIngredient>,
    pub products: Vec<SatisfactoryJsonIngredient>,
    #[serde(rename = "producedIn")]
    pub produced_in: Vec<String>,
    pub alternate: Option<bool>, // Add this field to match the JSON
}

fn de_str_or_float_opt<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;
    let val: Option<serde_json::Value> = Option::deserialize(deserializer).unwrap_or(None);
    match val {
        Some(serde_json::Value::String(s)) => s.parse::<f64>().map(Some).map_err(Error::custom),
        Some(serde_json::Value::Number(n)) => n
            .as_f64()
            .ok_or_else(|| Error::custom("Invalid number"))
            .map(Some),
        Some(serde_json::Value::Null) | None => Ok(None),
        _ => Err(Error::custom(
            "Expected string or float for mManufactoringDuration",
        )),
    }
}

#[derive(Debug, Deserialize)]
pub struct SatisfactoryClass {
    #[serde(rename = "ClassName")]
    pub class_name: String,
    #[serde(rename = "mDisplayName")]
    pub display_name: Option<String>,
    // For recipes: now as Option<String> for tuple parsing
    #[serde(rename = "mIngredients")]
    pub m_ingredients: Option<String>,
    #[serde(rename = "mProduct")]
    pub m_product: Option<String>,
    #[serde(rename = "mProducedIn")]
    pub m_produced_in: Option<String>,
    #[serde(
        rename = "mManufactoringDuration",
        default,
        deserialize_with = "de_str_or_float_opt"
    )]
    pub m_manufactoring_duration: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct SatisfactoryIngredientEntry {
    pub item_class: String,
    pub amount: f64,
}

fn clean_name(raw: &str) -> String {
    raw.trim_start_matches("Desc_")
        .trim_start_matches("Build_")
        .trim_start_matches("Recipe_")
        .trim_start_matches("TempRecipe_")
        .trim_end_matches("_C")
        .replace('_', " ")
}

pub fn satisfactory_json_to_recipe(json: &SatisfactoryJsonRecipe) -> Recipe {
    Recipe {
        name: clean_name(&json.display_name),
        inputs: json
            .ingredients
            .iter()
            .map(|i| ItemStack {
                item: clean_name(&i.item),
                quantity: i.amount.round() as u32,
            })
            .collect(),
        outputs: json
            .products
            .iter()
            .map(|i| ItemStack {
                item: clean_name(&i.item),
                quantity: i.amount.round() as u32,
            })
            .collect(),
        machine: CraftingMachine {
            name: json
                .produced_in
                .get(0)
                .map(|s| clean_name(s))
                .unwrap_or_default(),
        },
        time: (json.duration * 1000.0) as u32,
        enabled: !json.alternate.unwrap_or(false), // Default: alternate recipes are disabled
    }
}

pub fn build_display_name_map_from_assets(assets: &[SatisfactoryAsset]) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for asset in assets {
        for class in &asset.classes {
            if let Some(name) = class.display_name.clone() {
                let short = class.class_name.clone();
                map.insert(short.clone(), name.clone());
                // Add all possible suffixes ending with the class name
                // e.g. .../Desc_IronIngot.Desc_IronIngot_C'
                let class_name = class.class_name.clone();
                for prefix in [
                    "/Script/Engine.BlueprintGeneratedClass'",
                    "",
                    "/Game/FactoryGame/Resource/Parts/",
                    "/Game/FactoryGame/Resource/RawResources/",
                    "/Game/FactoryGame/Buildable/Factory/",
                    "/Script/FactoryGame.",
                ] {
                    let full = format!("{}{}", prefix, class_name);
                    map.insert(full.clone(), name.clone());
                    // Also try with trailing single quote
                    map.insert(format!("{}'", full), name.clone());
                }
                // Also map any substring ending with the class name (for robust fallback)
                // e.g. ...Desc_IronIngot_C'
                map.insert(format!("{}'", class_name), name.clone());
            }
        }
    }
    map
}

fn calc_quantity(item_name: String, amount: u32) -> u32 {
    // const FLUIDS: [&str; 15] = [
    //     "Crude Oil",
    //     "Water",
    //     "Fuel",
    //     "Heavy Oil Residue",
    //     "Nitrogen Gas",
    //     "Excited Photonic Matter",
    //     "Dark Matter Residue",
    //     "Nitric Acid",
    //     "Sulfuric Acid",
    //     "Turbofuel",
    //     "Liquid Biofuel",
    //     "Alumina Solution",
    //     "Rocket Fuel",
    //     "Ionized Fuel",
    //     "Dissolved Silica"
    // ];
    if amount >= 1000 {
        amount / 1000
    } else {
        amount // For all other items, use the given amount
    }
}

pub fn load_satisfactory_recipes_from_json(
    json_str: &str,
) -> Result<Vec<Recipe>, Box<dyn std::error::Error>> {
    let assets: Vec<SatisfactoryAsset> = serde_json::from_str(json_str)?;
    let display_name_map = build_display_name_map_from_assets(&assets);
    let mut recipes = Vec::new();
    for asset in assets {
        for class in asset.classes {
            if class.class_name.starts_with("Recipe_") {
                let machine_classes = class
                    .m_produced_in
                    .as_ref()
                    .map(|s| parse_produced_in_tuple(s))
                    .unwrap_or_default();
                let filtered_machines: Vec<_> = machine_classes
                    .iter()
                    .filter(|mc| {
                        *mc != "BP_WorkBenchComponent_C"
                            && *mc != "BP_WorkshopComponent_C"
                            && *mc != "BP_BuildGun_C"
                            && *mc != "FGBuildGun"
                    })
                    .collect();
                if filtered_machines.is_empty() {
                    continue; // skip if only workbench/workshop
                }
                let machine_name = filtered_machines
                    .get(0)
                    .and_then(|mc| display_name_map.get(*mc).cloned())
                    .or_else(|| filtered_machines.get(0).map(|mc| clean_name(mc)))
                    .unwrap_or_default();
                let recipe_display = class
                    .display_name
                    .clone()
                    .unwrap_or_else(|| class.class_name.clone());
                let inputs = class
                    .m_ingredients
                    .as_ref()
                    .map(|s| parse_ingredient_tuples(s))
                    .unwrap_or_default();
                let outputs = class
                    .m_product
                    .as_ref()
                    .map(|s| parse_ingredient_tuples(s))
                    .unwrap_or_default();
                let time = class.m_manufactoring_duration.unwrap_or(1.0);
                recipes.push(Recipe {
                    name: recipe_display,
                    inputs: inputs
                        .iter()
                        .map(|ing| {
                            let item_name = display_name_map
                                .get(&ing.item_class)
                                .cloned()
                                .unwrap_or_else(|| clean_name(&ing.item_class));
                            ItemStack {
                                item: item_name.clone(),
                                quantity: calc_quantity(item_name, ing.amount.round() as u32),
                            }
                        })
                        .collect(),
                    outputs: outputs
                        .iter()
                        .map(|prod| {
                            let item_name = display_name_map
                                .get(&prod.item_class)
                                .cloned()
                                .unwrap_or_else(|| clean_name(&prod.item_class));
                            ItemStack {
                                item: item_name.clone(),
                                quantity: calc_quantity(item_name, prod.amount.round() as u32),
                            }
                        })
                        .collect(),
                    machine: CraftingMachine { name: machine_name },
                    time: (time * 1000.0) as u32,
                    enabled: true,
                });
            }
        }
    }
    Ok(recipes)
}

fn extract_short_class_name(item_class: &str) -> String {
    // Handles both Unreal path and plain class name
    if let Some(pos) = item_class.rfind('/') {
        let after_slash = &item_class[pos + 1..];
        if let Some(dot_pos) = after_slash.rfind('.') {
            after_slash[dot_pos + 1..].to_string()
        } else {
            after_slash.to_string()
        }
    } else if let Some(dot_pos) = item_class.rfind('.') {
        item_class[dot_pos + 1..].to_string()
    } else {
        item_class.to_string()
    }
}

fn parse_ingredient_tuples(s: &str) -> Vec<SatisfactoryIngredientEntry> {
    // Example: ((ItemClass="...",Amount=3),(ItemClass="...",Amount=2))
    let mut entries = Vec::new();
    let re = regex::Regex::new(r#"ItemClass=\\?\"([^\"]+)\\?\",Amount=([0-9.]+)"#).unwrap();
    for cap in re.captures_iter(s) {
        let item_class_full = &cap[1];
        let item_class = extract_short_class_name(item_class_full);
        let amount: f64 = cap[2].parse().unwrap_or(1.0);
        entries.push(SatisfactoryIngredientEntry { item_class, amount });
    }
    entries
}

fn parse_produced_in_tuple(s: &str) -> Vec<String> {
    // Match everything after the last dot to the end of the string, for each entry inside parentheses
    let trimmed = s.trim();
    let mut result = Vec::new();
    if trimmed.starts_with('(') && trimmed.ends_with(')') {
        let inner = &trimmed[1..trimmed.len() - 1];
        for part in inner.split(',') {
            let part = part.trim().trim_matches('"');
            if let Some(dot_pos) = part.rfind('.') {
                let class_name = &part[dot_pos + 1..];
                if !class_name.is_empty() {
                    result.push(class_name.to_string());
                }
            } else if !part.is_empty() {
                result.push(part.to_string());
            }
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_print_recipe_table() {
        let path = "assets/satisfactory_en-US.json";
        let json_str = std::fs::read_to_string(path).expect("Failed to read JSON file");
        let recipes =
            load_satisfactory_recipes_from_json(&json_str).expect("Failed to load recipes");
        let assets: Vec<SatisfactoryAsset> =
            serde_json::from_str(&json_str).expect("Failed to parse JSON");
        let display_name_map = build_display_name_map_from_assets(&assets);
        let mut recipes = recipes;
        use std::collections::HashMap;
        recipes.sort_by(|a, b| {
            let a_name = a.outputs.get(0).map(|o| o.item.as_str()).unwrap_or("");
            let b_name = b.outputs.get(0).map(|o| o.item.as_str()).unwrap_or("");
            a_name.cmp(b_name)
        });
        println!(
            "{:<30} | {:<30} | {:<30} | {:<20} | {:<10}",
            "Recipe Name", "Inputs", "Outputs", "Machine", "Time(ms)"
        );
        println!("{}", "-".repeat(130));
        for recipe in &recipes {
            let inputs = recipe
                .inputs
                .iter()
                .map(|i| {
                    display_name_map
                        .get(&i.item)
                        .cloned()
                        .unwrap_or_else(|| i.item.clone())
                })
                .zip(recipe.inputs.iter().map(|i| i.quantity))
                .map(|(name, qty)| format!("{} x{}", name, qty))
                .collect::<Vec<_>>()
                .join(", ");
            let outputs = recipe
                .outputs
                .iter()
                .map(|o| {
                    display_name_map
                        .get(&o.item)
                        .cloned()
                        .unwrap_or_else(|| o.item.clone())
                })
                .zip(recipe.outputs.iter().map(|o| o.quantity))
                .map(|(name, qty)| format!("{} x{}", name, qty))
                .collect::<Vec<_>>()
                .join(", ");
            let machine = display_name_map
                .get(&recipe.machine.name)
                .cloned()
                .unwrap_or_else(|| recipe.machine.name.clone());
            println!(
                "{:<30} | {:<30} | {:<30} | {:<20} | {:<10}",
                recipe.name, inputs, outputs, machine, recipe.time
            );
        }
        // Print summary at the end
        let mut building_counts: HashMap<String, usize> = HashMap::new();
        for recipe in &recipes {
            let machine = display_name_map
                .get(&recipe.machine.name)
                .cloned()
                .unwrap_or_else(|| recipe.machine.name.clone());
            *building_counts.entry(machine).or_insert(0) += 1;
        }
        println!("\nRecipe count per building type:");
        let mut building_counts_vec: Vec<_> = building_counts.into_iter().collect();
        building_counts_vec.sort_by(|a, b| b.1.cmp(&a.1));
        for (building, count) in building_counts_vec {
            println!("{:<30} : {}", building, count);
        }
    }
}
