pub mod builder;
pub mod server;

/// Adds two numbers together.  A trivial example of a public function.
///
/// # Examples
///
/// ```
/// use grafton_server::add;
/// assert_eq!(add(2, 3), 5);
/// ```
#[must_use]
pub const fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        assert_eq!(add(2, 3), 5);
    }
}
