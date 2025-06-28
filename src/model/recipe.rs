use reactive_stores::Store;

// Data structures for factory building game recipes

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemStack {
    pub item: String,
    pub quantity: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CraftingMachine {
    pub name: String,
    // Add more fields as needed (e.g., speed, power usage)
}

#[derive(Debug, Clone, PartialEq, Eq, Store)]
pub struct Recipe {
    pub name: String,
    pub inputs: Vec<ItemStack>,
    pub outputs: Vec<ItemStack>,
    pub machine: CraftingMachine,
    pub time: u32, // crafting time in milliseconds
    pub enabled: bool, // true if recipe is enabled by default
}
