use crate::model::RESOURCE_AVAIL;

/// Returns true if the given product is a raw resource (should stop recursion)
pub fn is_raw_resource(product: &str) -> bool {
    RESOURCE_AVAIL.iter().any(|(name, _)| name == &product)
}
