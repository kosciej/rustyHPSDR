use std::fmt::Write;

pub fn format_u32_with_separators(value: u32) -> String {
    let mut result = String::new();
    let value_str = value.to_string();
    let len = value_str.len();

    // Iterate over the characters and insert separators
    for (i, ch) in value_str.chars().enumerate() {
        if (len - i).is_multiple_of(3) && i != 0 {
            write!(&mut result, ".").unwrap();
        }
        write!(&mut result, "{}", ch).unwrap();
    }

    result
}
