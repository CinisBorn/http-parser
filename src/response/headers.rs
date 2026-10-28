//! Provides tools for headers handling.
//! 
//! This module is design to parse, test, check, format and etc for response 
//! serialization.
//! 
//! check "headers_sections" at request folder for parsing logic.

use std::collections::HashMap;

pub(crate) fn insert_header_with_list_value<T: AsRef<str>> (
    headers: &mut HashMap<String, String>,
    key: &str,
    values: Vec<T>
) {
    let mut value = String::new();

    for v in values {
        value.push_str(v.as_ref());
        value.push_str(",");
    }

    value.pop();

    headers.insert(key.to_string(), value);
}

#[cfg(test)]
mod serialization {
    use super::*;

    #[test]
    fn insert_correctly() {
        let mut hashmap: HashMap<String, String> = HashMap::new();

        insert_header_with_list_value(
            &mut hashmap, 
            "test", 
            vec!["it", "works", "fine"]
        );

        let mut value = hashmap.get("test").unwrap().split(",");

        assert_eq!(value.next().unwrap(), "it");
        assert_eq!(value.next().unwrap(), "works");
        assert_eq!(value.next().unwrap(), "fine");
    }
}