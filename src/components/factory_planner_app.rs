use std::collections::HashSet;

use leptos::prelude::*;
use reactive_stores::Store;

use crate::{components::breakdown::BreakdownView, AppStore, AppStoreStoreFields};

#[component]
pub fn FactoryPlannerApp() -> impl IntoView {
    let store = use_context::<Store<AppStore>>().expect("recipes context");
    let (search, set_search) = signal(String::new());
    let (outputs, set_outputs) = signal(Vec::<(String, f64)>::new());
    let (show_autocomplete, set_show_autocomplete) = signal(false);
    let (highlighted, set_highlighted) = signal(None::<usize>);
    let enabled_recipes = use_context::<Memo<HashSet<String>>>().expect("enabled_recipes context");
    let (tab, set_tab) = signal("calc".to_string());
    let (recipe_output_filter, set_recipe_output_filter) = signal(String::new());
    let search_results_memo = Memo::new(move |_| {
        let search_val = search.get();
        let current_outputs: HashSet<String> =
            outputs.get().iter().map(|(n, _)| n.clone()).collect();
        if search_val.is_empty() {
            Vec::new()
        } else {
            store
                .recipes()
                .into_iter()
                .flat_map(|r| {
                    r.get()
                        .outputs
                        .iter()
                        .map(|o| o.item.clone())
                        .collect::<Vec<_>>()
                })
                .filter(|name| name.to_lowercase().contains(&search_val.to_lowercase()))
                .filter(|name| !current_outputs.contains(name))
                .collect()
        }
    });
    let add_output = move |name: String| {
        set_outputs.update(|outs| {
            if !outs.iter().any(|(n, _)| n == &name) {
                outs.push((name, 60.0));
            }
        });
        set_search.set(String::new());
        set_show_autocomplete.set(false);
    };
    view! {
        <div class="container mx-auto p-4">
            <h1 class="text-3xl font-bold mb-6">Factory Planner</h1>
            <div role="tablist" class="tabs tabs-boxed mb-6">
                <button
                    role="tab"
                    class=move || {
                        format!("tab{}", if tab.get() == "calc" { " tab-active" } else { "" })
                    }
                    aria-selected=move || tab.get() == "calc"
                    on:click=move |_| set_tab.set("calc".to_string())
                >
                    Calculator
                </button>
                <button
                    role="tab"
                    class=move || {
                        format!("tab{}", if tab.get() == "recipes" { " tab-active" } else { "" })
                    }
                    aria-selected=move || tab.get() == "recipes"
                    on:click=move |_| set_tab.set("recipes".to_string())
                >
                    Recipe Options
                </button>
            </div>
            {move || {
                view! {
                    <>
                        <div
                            style=if tab.get() == "calc" { "" } else { "display:none;" }
                            id="tab-calc-content"
                        >
                            <div class="mb-4 relative">
                                <input
                                    class="input input-bordered w-full"
                                    type="text"
                                    placeholder="Search product..."
                                    value=move || search.get()
                                    on:input=move |ev| {
                                        set_search.set(event_target_value(&ev));
                                        set_highlighted.set(None);
                                    }
                                    on:focus=move |_| set_show_autocomplete.set(true)
                                    on:blur=move |_| set_show_autocomplete.set(false)
                                    on:keydown=move |ev| {
                                        let results = search_results_memo.get();
                                        let len = results.len();
                                        if len == 0 {
                                            return;
                                        }
                                        match ev.key().as_str() {
                                            "ArrowDown" => {
                                                set_highlighted
                                                    .update(|h| {
                                                        *h = Some(h.map_or(0, |i| (i + 1).min(len - 1)));
                                                    });
                                            }
                                            "ArrowUp" => {
                                                set_highlighted
                                                    .update(|h| {
                                                        *h = Some(h.map_or(0, |i| i.saturating_sub(1)));
                                                    });
                                            }
                                            "Enter" => {
                                                if let Some(i) = highlighted.get() {
                                                    if let Some(name) = results.get(i) {
                                                        add_output(name.clone());
                                                    }
                                                }
                                            }
                                            _ => {}
                                        }
                                    }
                                />
                                <ul class="menu bg-base-100 border border-base-300 rounded-b shadow max-h-60 overflow-auto z-10 absolute left-0 right-0">
                                    {move || {
                                        if show_autocomplete.get() && !search.get().is_empty() {
                                            let results = search_results_memo.get();
                                            (0..results.len())
                                                .map(|i| {
                                                    let name = results[i].clone();
                                                    let add_output = add_output.clone();
                                                    let set_highlighted = set_highlighted.clone();
                                                    let highlighted = highlighted.get();
                                                    view! {
                                                        <li
                                                            class=format!(
                                                                "menu-item p-2 cursor-pointer hover:bg-base-200 {}",
                                                                if Some(i) == highlighted {
                                                                    "bg-primary text-primary-content"
                                                                } else {
                                                                    ""
                                                                },
                                                            )
                                                            on:mousedown=move |_| add_output(name.clone())
                                                            on:mouseover=move |_| set_highlighted.set(Some(i))
                                                        >
                                                            {name.clone()}
                                                        </li>
                                                    }
                                                })
                                                .collect::<Vec<_>>()
                                        } else {
                                            Vec::new()
                                        }
                                    }}
                                </ul>
                            </div>
                            <h2 class="text-xl font-semibold mt-8 mb-2">Outputs</h2>
                            <div class="overflow-x-auto">
                                <table class="table table-zebra w-full">
                                    <thead>
                                        <tr>
                                            <th>Product</th>
                                            <th>Rate (items/min)</th>
                                            <th>Actions</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        {move || {
                                            outputs
                                                .get()
                                                .iter()
                                                .enumerate()
                                                .map(|(i, (name, rate))| {
                                                    let name = name.clone();
                                                    let set_outputs = set_outputs.clone();
                                                    view! {
                                                        <tr>
                                                            <td>{name.clone()}</td>
                                                            <td>
                                                                <input
                                                                    class="input input-bordered w-24"
                                                                    type="number"
                                                                    min="0.1"
                                                                    step="0.1"
                                                                    value=rate.to_string()
                                                                    on:input=move |ev| {
                                                                        let val = event_target_value(&ev).parse().unwrap_or(0.0);
                                                                        set_outputs.update(|outs| outs[i].1 = val);
                                                                    }
                                                                />
                                                            </td>
                                                            <td>
                                                                <button
                                                                    class="btn btn-error btn-sm"
                                                                    on:click=move |_| {
                                                                        set_outputs
                                                                            .update(|outs| {
                                                                                outs.remove(i);
                                                                            })
                                                                    }
                                                                >
                                                                    Remove
                                                                </button>
                                                            </td>
                                                        </tr>
                                                    }
                                                })
                                                .collect::<Vec<_>>()
                                        }}
                                    </tbody>
                                </table>
                            </div>
                            <h2 class="text-xl font-semibold mt-8 mb-2">Breakdown</h2>
                            <div class="overflow-x-auto">
                                <BreakdownView outputs=outputs />
                            </div>
                        </div>
                        <div
                            style=if tab.get() == "recipes" { "" } else { "display:none;" }
                            id="tab-recipes-content"
                        >
                            <div class="mb-4">
                                <input
                                    class="input input-bordered w-full max-w-xs"
                                    type="text"
                                    placeholder="Filter by output product..."
                                    value=move || recipe_output_filter.get()
                                    on:input=move |ev| {
                                        set_recipe_output_filter.set(event_target_value(&ev))
                                    }
                                />
                            </div>
                            <div class="overflow-x-auto">
                                <table class="table table-zebra w-full">
                                    <thead>
                                        <tr>
                                            <th class="w-20">Enable</th>
                                            <th class="w-64">Recipe Name</th>
                                            <th class="w-64">Outputs</th>
                                            <th class="w-64">Inputs</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        {store
                                            .recipes()
                                            .into_iter()
                                            .filter(|r| {
                                                let filter = recipe_output_filter.get().to_lowercase();
                                                if filter.is_empty() {
                                                    true
                                                } else {
                                                    r.get()
                                                        .outputs
                                                        .iter()
                                                        .any(|o| o.item.to_lowercase().contains(&filter))
                                                }
                                            })
                                            .map(|r| {
                                                let name = r.get().name.clone();
                                                let outputs = r
                                                    .get()
                                                    .outputs
                                                    .iter()
                                                    .map(|o| o.item.clone())
                                                    .collect::<Vec<_>>()
                                                    .join(", ");
                                                let inputs = r
                                                    .get()
                                                    .inputs
                                                    .iter()
                                                    .map(|i| i.item.clone())
                                                    .collect::<Vec<_>>()
                                                    .join(", ");
                                                view! {
                                                    <>
                                                        <tr>
                                                            <td>
                                                                <input
                                                                    type="checkbox"
                                                                    class="checkbox"
                                                                    checked=move || { enabled_recipes.get().contains(&name) }
                                                                />
                                                            </td>
                                                            <td>{r.get().name.clone()}</td>
                                                            <td>{outputs}</td>
                                                            <td>{inputs}</td>
                                                        </tr>
                                                        <tr class="bg-base-200">
                                                            <td colspan="4" class="p-0">
                                                                <div class="collapse collapse-arrow">
                                                                    <input type="checkbox" class="peer" />
                                                                    <div class="collapse-title text-md font-medium">
                                                                        Details
                                                                    </div>
                                                                    <div class="collapse-content">
                                                                        <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                                                                            <div>
                                                                                <div class="font-semibold mb-1">Outputs</div>
                                                                                <table class="table table-xs w-full">
                                                                                    <thead>
                                                                                        <tr>
                                                                                            <th>Product</th>
                                                                                            <th>Per Craft</th>
                                                                                            <th>Rate/min</th>
                                                                                        </tr>
                                                                                    </thead>
                                                                                    <tbody>
                                                                                        {r
                                                                                            .get()
                                                                                            .outputs
                                                                                            .iter()
                                                                                            .map(|o| {
                                                                                                let rate = (o.quantity as f64)
                                                                                                    * (60_000.0 / r.get().time as f64);
                                                                                                view! {
                                                                                                    <tr>
                                                                                                        <td>{o.item.clone()}</td>
                                                                                                        <td>{o.quantity}</td>
                                                                                                        <td>{format!("{:.2}", rate)}</td>
                                                                                                    </tr>
                                                                                                }
                                                                                            })
                                                                                            .collect::<Vec<_>>()}
                                                                                    </tbody>
                                                                                </table>
                                                                            </div>
                                                                            <div>
                                                                                <div class="font-semibold mb-1">Inputs</div>
                                                                                <table class="table table-xs w-full">
                                                                                    <thead>
                                                                                        <tr>
                                                                                            <th>Product</th>
                                                                                            <th>Per Craft</th>
                                                                                            <th>Rate/min</th>
                                                                                        </tr>
                                                                                    </thead>
                                                                                    <tbody>
                                                                                        {r
                                                                                            .get()
                                                                                            .inputs
                                                                                            .iter()
                                                                                            .map(|i| {
                                                                                                let rate = (i.quantity as f64)
                                                                                                    * (60_000.0 / r.get().time as f64);
                                                                                                view! {
                                                                                                    <tr>
                                                                                                        <td>{i.item.clone()}</td>
                                                                                                        <td>{i.quantity}</td>
                                                                                                        <td>{format!("{:.2}", rate)}</td>
                                                                                                    </tr>
                                                                                                }
                                                                                            })
                                                                                            .collect::<Vec<_>>()}
                                                                                    </tbody>
                                                                                </table>
                                                                            </div>
                                                                        </div>
                                                                        <div class="mt-4 flex flex-wrap gap-8">
                                                                            <div>
                                                                                <span class="font-semibold">Machine:</span>
                                                                                {r.get().machine.name.clone()}
                                                                            </div>
                                                                            <div>
                                                                                <span class="font-semibold">Craft Time:</span>
                                                                                {format!("{:.2} ms", r.get().time)}
                                                                            </div>
                                                                        </div>
                                                                    </div>
                                                                </div>
                                                            </td>
                                                        </tr>
                                                    </>
                                                }
                                            })
                                            .collect::<Vec<_>>()}
                                    </tbody>
                                </table>
                            </div>
                        </div>
                    </>
                }
            }}
        </div>
    }
}
