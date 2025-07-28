use anyhow::Result;

pub fn find_matches(content: &str, pattern: &str, mut writer: impl std::io::Write) -> Result<()> {
    if content.contains(pattern) {
        writeln!(writer, "{}", content)?;
    };

    Ok(())
}

#[test]
fn find_a_match() {
    let mut result = Vec::new();
    let _ = find_matches("lorem ipsum", "lorem", &mut result);
    assert_eq!(result, b"lorem ipsum\n");
}

#[test]
fn dont_find_a_match() {
    let mut result: Vec<u8> = Vec::new();
    let _ = find_matches("lorem ipsum", "loreal", &mut result);
    assert_eq!(result, b"");
}
