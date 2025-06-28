pub mod recipe;

/// An empty function for testing purposes.
pub fn empty_function() {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_function() {
        empty_function();
        // No assertion needed for an empty function
    }
}