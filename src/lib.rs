//! Deterministic semantics-preserving rewrites for Mission-time Linear Temporal Logic.

/// Placeholder entry point.
pub fn hello() -> &'static str {
    "hello from tl_rewrite"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hello_returns_greeting() {
        assert!(hello().contains("tl_rewrite"));
    }
}
