use leptos::prelude::*;
use super::breakdown::BreakdownNode;
use std::collections::HashMap;

#[component]
pub fn RawResourceSummary(
    breakdown: Memo<Vec<BreakdownNode>>,
) -> impl IntoView {
    view! {
        <div class="mt-6">
            <h3 class="text-lg font-semibold mb-2">Total Raw Resource Rate</h3>
            <table class="table table-xs w-full">
                <thead><tr><th>Resource</th><th>Total Rate (items/min)</th></tr></thead>
                <tbody>
                    {move || {
                        let mut raw_map = HashMap::new();
                        fn collect_raws(node: &BreakdownNode, map: &mut HashMap<String, f64>) {
                            if node.machine.is_none() {
                                *map.entry(node.product.clone()).or_insert(0.0) += node.rate;
                            }
                            for child in &node.children {
                                collect_raws(child, map);
                            }
                        }
                        for node in breakdown.get().iter() {
                            collect_raws(node, &mut raw_map);
                        }
                        raw_map.iter().map(|(res, rate)| view! {
                            <tr><td>{res.clone()}</td><td>{format!("{:.2}", rate)}</td></tr>
                        }).collect::<Vec<_>>()
                    }}
                </tbody>
            </table>
        </div>
    }
}

#[component]
pub fn BuildingSummary(breakdown: Memo<Vec<BreakdownNode>>) -> impl IntoView {
    view! {
        <div class="mt-6">
            <h3 class="text-lg font-semibold mb-2">Total Buildings Needed</h3>
            <table class="table table-xs w-full">
                <thead><tr><th>Building</th><th>Total Count</th></tr></thead>
                <tbody>
                    {move || {
                        let mut building_map = HashMap::new();
                        fn collect_buildings(node: &BreakdownNode, map: &mut HashMap<String, f64>) {
                            if let (Some(machine), Some(count)) = (node.machine.as_ref(), node.machines_needed) {
                                *map.entry(machine.clone()).or_insert(0.0) += count;
                            }
                            for child in &node.children {
                                collect_buildings(child, map);
                            }
                        }
                        for node in breakdown.get().iter() {
                            collect_buildings(node, &mut building_map);
                        }
                        building_map.iter().map(|(machine, count)| view! {
                            <tr><td>{machine.clone()}</td><td>{format!("{:.2}", count)}</td></tr>
                        }).collect::<Vec<_>>()
                    }}
                </tbody>
            </table>
        </div>
    }
}
