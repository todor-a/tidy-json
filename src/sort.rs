use rand::seq::SliceRandom;
use std::collections::BTreeMap;

use log::debug;
use serde_json::Value;

use crate::SortOrder;

pub fn sort(value: &Value, order: &SortOrder) -> Value {
    match value {
        Value::Object(map) => {
            let sorted_map: BTreeMap<_, _> = map.iter().collect();
            let mut entries: Vec<_> = sorted_map.into_iter().collect();

            debug!("hm? {:?}", entries);

            match order {
                SortOrder::AlphabeticalAsc => entries.sort_by(|(a, _), (b, _)| a.cmp(b)),
                SortOrder::AlphabeticalDesc => entries.sort_by(|(a, _), (b, _)| b.cmp(a)),
                SortOrder::Random => {
                    let mut rng = rand::thread_rng();
                    entries.shuffle(&mut rng);
                }
            }

            Value::Object(
                entries
                    .into_iter()
                    .map(|(k, v)| (k.clone(), sort(v, order)))
                    .collect(),
            )
        }
        Value::Array(arr) => Value::Array(arr.iter().map(|v| sort(v, order)).collect()),
        _ => value.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_debug_snapshot;

    #[test]
    fn should_apply_uniform_indent() {
        let data = r#"
        {
            "c": 3,
         "b": 2,
              "a": 1
        }"#;

        let json: Value = serde_json::from_str(data).unwrap();
        let sorted_json = sort(&json, &SortOrder::AlphabeticalAsc);
        assert_debug_snapshot!(sorted_json);
    }

    #[test]
    fn test_sort_json_simple() {
        let data = r#"
        {
            "c": 3,
            "b": 2,
            "a": 1
        }"#;

        let json: Value = serde_json::from_str(data).unwrap();
        let sorted_obj = sort(&json, &SortOrder::AlphabeticalAsc);
        assert_debug_snapshot!(sorted_obj);
    }

    #[test]
    fn test_sort_json_nested() {
        let data = r#"
        {
            "c": {
                "b": 4,
                "a": 3
            },
            "b": 2,
            "a": 1
        }"#;

        let json: Value = serde_json::from_str(data).unwrap();
        let sorted_obj = sort(&json, &SortOrder::AlphabeticalAsc);
        assert_debug_snapshot!(sorted_obj);
    }

    #[test]
    fn test_sort_json_preserve_array() {
        let data = r#"
        {
            "c": [4, 3],
            "b": 2,
            "a": 1
        }"#;
        let json: Value = serde_json::from_str(data).unwrap();
        let sorted_obj = sort(&json, &SortOrder::AlphabeticalAsc);
        assert_debug_snapshot!(sorted_obj);
    }

    #[test]
    fn test_sort_json_reverse_simple() {
        let data = r#"
        {
            "a": 3,
            "b": 2,
            "c": 1
        }"#;
        let json: Value = serde_json::from_str(data).unwrap();
        let sorted_obj = sort(&json, &SortOrder::AlphabeticalDesc);
        assert_debug_snapshot!(sorted_obj);
    }

    // #[test]
    // fn test_sort_json_overrides() {
    //     let data = r#"
    //     {
    //         "e": 5,
    //         "c": 3,
    //         "f": 6,
    //         "b": 2,
    //         "d": 4,
    //         "a": 1
    //     }"#;
    //             let json: Value = serde_json::from_str(data).unwrap();
    //     let sorted_obj = sort(
    //         &obj,
    //         &SortOrder::AlphabeticalAsc,
    //         Some(vec!["c".to_string(), "d".to_string()]),
    //         None,
    //     );
    //     assert_debug_snapshot!(sorted_obj);
    // }

    // #[test]
    // fn test_sort_json_reverse_overrides() {
    //     let data = r#"
    //     {
    //         "e": 5,
    //         "c": 3,
    //         "f": 6,
    //         "b": 2,
    //         "d": 4,
    //         "a": 1
    //     }"#;
    //             let json: Value = serde_json::from_str(data).unwrap();
    //     let sorted_obj = sort(
    //         &obj,
    //         &["desc"],
    //         Some(vec!["c".to_string(), "d".to_string()]),
    //         None,
    //     );
    //     assert_debug_snapshot!(sorted_obj);
    // }

    // #[test]
    // fn test_sort_json_underrides() {
    //     let data = r#"
    //     {
    //         "e": 5,
    //         "c": 3,
    //         "f": 6,
    //         "b": 2,
    //         "d": 4,
    //         "a": 1
    //     }"#;
    //             let json: Value = serde_json::from_str(data).unwrap();
    //     let sorted_obj = sort(
    //         &obj,
    //         SortOrder::AlphabeticalAsc,
    //         None,
    //         Some(vec!["c".to_string(), "d".to_string()]),
    //     );
    //     assert_debug_snapshot!(sorted_obj);
    // }

    // #[test]
    // fn test_sort_json_reverse_underrides() {
    //     let data = r#"
    //     {
    //         "e": 5,
    //         "c": 3,
    //         "f": 6,
    //         "b": 2,
    //         "d": 4,
    //         "a": 1
    //     }"#;
    //             let json: Value = serde_json::from_str(data).unwrap();
    //     let sorted_obj = sort(
    //         &obj,
    //         &["desc"],
    //         None,
    //         Some(vec!["c".to_string(), "d".to_string()]),
    //     );
    //     assert_debug_snapshot!(sorted_obj);
    // }

    #[test]
    fn test_sort_json_array_of_objects() {
        let data = r#"
        {
            "c": [
                {
                    "e": 5,
                    "d": 4
                },
                {
                    "g": 7,
                    "f": 6
                }
            ],
            "b": 2,
            "a": 1
        }"#;
        let json: Value = serde_json::from_str(data).unwrap();
        let sorted_obj = sort(&json, &SortOrder::AlphabeticalAsc);
        assert_debug_snapshot!(sorted_obj);
    }

    // #[test]
    // fn test_sort_json_key_length() {
    //     let data = r#"
    //     {
    //         "cc": 3,
    //         "bbb": 2,
    //         "a": 1
    //     }"#;
    //             let json: Value = serde_json::from_str(data).unwrap();
    //     let sorted_obj = sort_json_by_key_length(&json, &SortOrder::AlphabeticalAsc);
    //     assert_debug_snapshot!(sorted_obj);
    // }

    // #[test]
    // fn test_sort_json_reverse_key_length() {
    //     let data = r#"
    //     {
    //         "cc": 3,
    //         "bbb": 2,
    //         "a": 1
    //     }"#;
    //             let json: Value = serde_json::from_str(data).unwrap();
    //     let sorted_obj = sort_json_by_key_length(&json, &SortOrder::AlphabeticalDesc);
    //     assert_debug_snapshot!(sorted_obj);
    // }

    // #[test]
    // fn test_sort_json_alphanum() {
    //     let data = r#"
    //     {
    //         "a10": 3,
    //         "a2": 2,
    //         "a1": 1,
    //         "b11": 3,
    //         "b2": 2,
    //         "b0": 1
    //     }"#;
    //             let json: Value = serde_json::from_str(data).unwrap();
    //     let sorted_obj = sort_json_alphanum(&json, &SortOrder::AlphabeticalAsc);
    //     assert_debug_snapshot!(sorted_obj);
    // }

    // #[test]
    // fn test_sort_json_reverse_alphanum() {
    //     let data = r#"
    //     {
    //         "a10": 3,
    //         "a2": 2,
    //         "a1": 1,
    //         "b11": 3,
    //         "b2": 2,
    //         "b0": 1
    //     }"#;
    //             let json: Value = serde_json::from_str(data).unwrap();
    //     let sorted_obj = sort_json_alphanum(&json, &SortOrder::AlphabeticalDesc);
    //     assert_debug_snapshot!(sorted_obj);
    // }
}
