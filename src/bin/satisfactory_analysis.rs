use factory_planner::{adapters::{satisfactory_adapter::{build_machine_power_map_from_assets, load_satisfactory_recipes_from_json}, satisfactory_asset::SatisfactoryAsset}, model::compute_item_analysis};

fn main() {
        let path = "assets/satisfactory_en-US.json";
        let json_str = std::fs::read_to_string(path).expect("Failed to read JSON file");
        let recipes = load_satisfactory_recipes_from_json(&json_str).expect("Failed to load recipes");
        let assets: Vec<SatisfactoryAsset> = serde_json::from_str(&json_str).expect("Failed to parse JSON");
        let machine_power_map = build_machine_power_map_from_assets(&assets);
        let item_analysis = compute_item_analysis(&recipes, &machine_power_map);
        let json = json5::to_string(&item_analysis).expect("Failed to serialize item analysis to JSON5");
        std::fs::write("assets/satisfactory_item_analysis.json5", json).expect("Failed to write item_analysis.json");
    }
