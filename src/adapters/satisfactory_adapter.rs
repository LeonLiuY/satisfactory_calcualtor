// Satisfactory adapter module
// (No need to pub use crate::satisfactory_adapter;)

// Satisfactory adapter logic moved from adapters.rs
use crate::model::recipe::{Recipe, ItemStack, CraftingMachine};
use serde::Deserialize;

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
        inputs: json.ingredients.iter().map(|i| ItemStack {
            item: clean_name(&i.item),
            quantity: i.amount.round() as u32
        }).collect(),
        outputs: json.products.iter().map(|i| ItemStack {
            item: clean_name(&i.item),
            quantity: i.amount.round() as u32
        }).collect(),
        machine: CraftingMachine {
            name: json.produced_in.get(0).map(|s| clean_name(s)).unwrap_or_default()
        },
        time: (json.duration * 1000.0) as u32,
        enabled: !json.alternate.unwrap_or(false), // Default: alternate recipes are disabled
    }
}

pub fn load_satisfactory_recipes_from_json(json_str: &str) -> Result<Vec<Recipe>, Box<dyn std::error::Error>> {
    let json_recipes_map: std::collections::HashMap<String, Vec<SatisfactoryJsonRecipe>> = serde_json::from_str(json_str)?;
    let recipes = json_recipes_map
        .values()
        .flat_map(|v| v.iter()
            .filter(|r| !r.produced_in.is_empty())
            .filter(|r| !r.produced_in.iter().any(|m| m == "Desc_Converter_C"))
            .map(satisfactory_json_to_recipe))
        .collect();
    Ok(recipes)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_print_recipe_table() {
        let path = "assets/satisfactory_recipes.json";
        let json_str = std::fs::read_to_string(path).expect("Failed to read JSON file");
        let mut recipes = load_satisfactory_recipes_from_json(&json_str).expect("Failed to load recipes");
        recipes.sort_by(|a, b| {
            let a_name = a.outputs.get(0).map(|o| o.item.as_str()).unwrap_or("");
            let b_name = b.outputs.get(0).map(|o| o.item.as_str()).unwrap_or("");
            a_name.cmp(b_name)
        });
        println!("{:<30} | {:<30} | {:<30} | {:<10} | {:<10}", "Recipe Name", "Inputs", "Outputs", "Machine", "Time(ms)");
        println!("{}", "-".repeat(120));
        for recipe in &recipes {
            let inputs = recipe.inputs.iter().map(|i| format!("{} x{}", i.item, i.quantity)).collect::<Vec<_>>().join(", ");
            let outputs = recipe.outputs.iter().map(|o| format!("{} x{}", o.item, o.quantity)).collect::<Vec<_>>().join(", ");
            println!("{:<30} | {:<30} | {:<30} | {:<10} | {:<10}",
                recipe.name,
                inputs,
                outputs,
                recipe.machine.name,
                recipe.time
            );
        }
    }
}
