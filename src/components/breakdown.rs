use crate::{components::raw_resource::is_raw_resource, model::AppStore};
use leptos::prelude::*;
use reactive_stores::Store;
use std::collections::HashSet;
use crate::model::recipe::Recipe;

#[derive(Clone, Debug, PartialEq)]
pub struct BreakdownNode {
    pub product: String,
    pub rate: f64,
    pub recipe_name: Option<String>,
    pub machine: Option<String>,
    pub machines_needed: Option<f64>,
    pub children: Vec<BreakdownNode>,
}

pub fn flatten_tree(node: &BreakdownNode, depth: usize, out: &mut Vec<(usize, BreakdownNode)>) {
    out.push((depth, node.clone()));
    for child in &node.children {
        flatten_tree(child, depth + 1, out);
    }
}

#[component]
pub fn BreakdownView(
    outputs: ReadSignal<Vec<(String, f64)>>,
) -> impl IntoView {
    let store = use_context::<Store<AppStore>>().expect("Store<AppStore> context");
    let enabled_recipes = use_context::<Memo<HashSet<String>>>().expect("enabled_recipes context");
    fn build_tree(
        product: &str,
        rate: f64,
        recipes: &[Recipe],
        enabled: &HashSet<String>,
        path: &mut Vec<String>,
    ) -> BreakdownNode {
        if path.contains(&product.to_string()) {
            // True cycle detected (recursion in the current path)
            return BreakdownNode {
                product: product.to_string(),
                rate,
                recipe_name: Some("Cycle".to_string()),
                machine: None,
                machines_needed: None,
                children: vec![],
            };
        }
        // Stop recursion at raw resources
        if is_raw_resource(product) {
            return BreakdownNode {
                product: product.to_string(),
                rate,
                recipe_name: None,
                machine: None,
                machines_needed: None,
                children: vec![],
            };
        }
        path.push(product.to_string());
        let recipes_for_product: Vec<&Recipe> = recipes.iter()
            .filter(|r| enabled.contains(&r.name) && r.outputs.iter().any(|o| o.item == product))
            .collect();
        let node = if !recipes_for_product.is_empty() {
            let recipe = recipes_for_product[0];
            let output = recipe.outputs.iter().find(|o| o.item == product).unwrap();
            let items_per_min = output.quantity as f64 * (60_000.0 / recipe.time as f64);
            let machines_needed = rate / items_per_min;
            let children = recipe.inputs.iter().map(|input| {
                let input_rate = rate * (input.quantity as f64) / (output.quantity as f64);
                build_tree(&input.item, input_rate, recipes, enabled, path)
            }).collect();
            BreakdownNode {
                product: product.to_string(),
                rate,
                recipe_name: Some(recipe.name.clone()),
                machine: Some(recipe.machine.name.clone()),
                machines_needed: Some(machines_needed),
                children,
            }
        } else {
            // No enabled recipe produces this product: it's a raw resource
            BreakdownNode {
                product: product.to_string(),
                rate,
                recipe_name: None,
                machine: None,
                machines_needed: None,
                children: vec![],
            }
        };
        path.pop();
        node
    }

    let breakdown = Memo::new(move |_| {
        let recipes = store.with(|s| s.recipes.clone());
        let enabled = enabled_recipes.get();
        outputs.get().iter().map(|(product, rate)| {
            let mut path = Vec::new();
            build_tree(product, *rate, &recipes, &enabled, &mut path)
        }).collect::<Vec<_>>()
    });

    view! {
        <div class="overflow-x-auto">
            <table class="table table-zebra w-full mt-4">
                <thead>
                    <tr>
                        <th>Product</th>
                        <th>Total Rate (items/min)</th>
                        <th>Recipe</th>
                        <th>Machine</th>
                        <th>Machines Needed</th>
                    </tr>
                </thead>
                <tbody>
                    {move || {
                        let mut flat: Vec<(usize, BreakdownNode)> = Vec::new();
                        for node in breakdown.get().iter() {
                            flatten_tree(node, 0, &mut flat);
                        }
                        flat.into_iter().map(|(depth, node)| view! {
                            <tr>
                                <td style={format!("padding-left:{}em;", depth * 2)}>{node.product.clone()}</td>
                                <td>{format!("{:.2}", node.rate)}</td>
                                <td>{node.recipe_name.clone().unwrap_or("(Raw Resource)".to_string())}</td>
                                <td>{node.machine.clone().unwrap_or("-".to_string())}</td>
                                <td>{node.machines_needed.map(|m| format!("{:.2}", m)).unwrap_or("-".to_string())}</td>
                            </tr>
                        }).collect::<Vec<_>>()
                    }}
                </tbody>
            </table>
            <crate::components::summaries::RawResourceSummary breakdown=breakdown />
            <crate::components::summaries::BuildingSummary breakdown=breakdown />
        </div>
    }
}
