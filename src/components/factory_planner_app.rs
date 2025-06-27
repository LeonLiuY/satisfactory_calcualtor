//! Main app orchestrator for Factory Planner
use crate::components::calc_tab::CalcTab;
use crate::components::recipes_tab::RecipesTab;
use crate::components::analysis_tab::AnalysisTab;

use leptos::prelude::*;

#[component]
pub fn FactoryPlannerApp() -> impl IntoView {
    let (tab, set_tab) = signal("calc".to_string());
    view! {
        <div class="container mx-auto p-4">
            <h1 class="text-3xl font-bold mb-6">Factory Planner</h1>
            <div role="tablist" class="tabs tabs-boxed mb-6">
                <button
                    role="tab"
                    class=move || format!("tab{}", if tab.get() == "calc" { " tab-active" } else { "" })
                    aria-selected=move || tab.get() == "calc"
                    on:click=move |_| set_tab.set("calc".to_string())
                >
                    Calculator
                </button>
                <button
                    role="tab"
                    class=move || format!("tab{}", if tab.get() == "recipes" { " tab-active" } else { "" })
                    aria-selected=move || tab.get() == "recipes"
                    on:click=move |_| set_tab.set("recipes".to_string())
                >
                    Recipe Options
                </button>
                <button
                    role="tab"
                    class=move || format!("tab{}", if tab.get() == "analysis" { " tab-active" } else { "" })
                    aria-selected=move || tab.get() == "analysis"
                    on:click=move |_| set_tab.set("analysis".to_string())
                >
                    Recipe Analysis
                </button>
            </div>
            <div id="tab-calc-content" style=move || if tab.get() == "calc" { "" } else { "display:none;" }>
                <CalcTab />
            </div>
            <div id="tab-recipes-content" style=move || if tab.get() == "recipes" { "" } else { "display:none;" }>
                <RecipesTab />
            </div>
            <div id="tab-analysis-content" style=move || if tab.get() == "analysis" { "" } else { "display:none;" }>
                <AnalysisTab />
            </div>
        </div>
    }
}
