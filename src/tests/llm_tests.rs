const MAX_DIFF_CHARS: usize = 8000;

fn truncate_diff(diff: &str) -> String {
    if diff.len() > MAX_DIFF_CHARS {
        format!(
            "{}...\n[diff truncated, {} more bytes]",
            &diff[..MAX_DIFF_CHARS],
            diff.len() - MAX_DIFF_CHARS
        )
    } else {
        diff.to_string()
    }
}

#[test]
fn truncate_diff_short_input_unchanged() {
    let short = "hello world";
    let result = truncate_diff(short);
    assert_eq!(result, short);
}

#[test]
fn truncate_diff_exact_limit_unchanged() {
    let exact: String = "a".repeat(MAX_DIFF_CHARS);
    let result = truncate_diff(&exact);
    assert_eq!(result, exact);
}

#[test]
fn truncate_diff_over_limit_truncates() {
    let over: String = "x".repeat(MAX_DIFF_CHARS + 100);
    let result = truncate_diff(&over);

    assert!(result.starts_with(&"x".repeat(MAX_DIFF_CHARS)));
    assert!(result.contains("...\n[diff truncated, 100 more bytes]"));
}
