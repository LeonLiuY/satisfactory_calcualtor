use leptos::prelude::*;
use std::collections::HashSet;
use crate::recipe::Recipe;
use crate::Store;
use crate::AppStore;

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

    fn find_recipes<'a>(product: &str, recipes: &'a [Recipe], enabled: &HashSet<String>) -> Vec<&'a Recipe> {
        recipes.iter()
            .filter(|r| r.outputs.iter().any(|o| o.item == product))
            .filter(|r| enabled.contains(&r.name))
            .collect()
    }
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
        path.push(product.to_string());
        // Find all recipes (enabled or not) that produce this product
        let all_possible_recipes: Vec<&Recipe> = recipes.iter().filter(|r| r.outputs.iter().any(|o| o.item == product)).collect();
        let node = if all_possible_recipes.is_empty() {
            // No recipe produces this product: it's a true raw resource
            BreakdownNode {
                product: product.to_string(),
                rate,
                recipe_name: None,
                machine: None,
                machines_needed: None,
                children: vec![],
            }
        } else {
            // Now, find enabled recipes for this product
            let enabled_recipes: Vec<&Recipe> = all_possible_recipes.iter().copied().filter(|r| enabled.contains(&r.name)).collect();
            if !enabled_recipes.is_empty() {
                let recipe = enabled_recipes[0];
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
                // There are recipes for this product, but none are enabled
                BreakdownNode {
                    product: product.to_string(),
                    rate,
                    recipe_name: None,
                    machine: None,
                    machines_needed: None,
                    children: vec![],
                }
            }
        };
        path.pop();
        node
    }

    let tree = Memo::new(move |_| {
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
                        for node in tree.get().iter() {
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
            <crate::components::summaries::RawResourceSummary breakdown=tree />
            <crate::components::summaries::BuildingSummary breakdown=tree />
        </div>
    }
}
