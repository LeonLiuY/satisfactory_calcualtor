use leptos::prelude::*;
use crate::{components::breakdown::BreakdownView, model::{AppStore, AppStoreStoreFields}};
use reactive_stores::Store;

#[component]
pub fn CalcTab() -> impl IntoView {
    let (search, set_search) = signal(String::new());
    let (outputs, set_outputs) = signal(Vec::<(String, f64)>::new());
    let (show_autocomplete, set_show_autocomplete) = signal(false);
    let (highlighted, set_highlighted) = signal(None::<usize>);
    let store = use_context::<Store<AppStore>>().expect("AppStore context");
    let search_results_memo = Memo::new(move |_| {
        let search_val = search.get();
        let current_outputs: std::collections::HashSet<String> =
            outputs.get().iter().map(|(n, _)| n.clone()).collect();
        if search_val.is_empty() {
            Vec::new()
        } else {
            let mut seen = std::collections::HashSet::new();
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
                .filter(|name| seen.insert(name.clone()))
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
    }
}
