use std::cmp::Eq;
use std::collections::HashMap;
use std::hash::Hash;

use itertools::Itertools;

pub fn first<T>(collection: Vec<T>) -> Option<T> {
    collection.into_iter().next()
}

pub fn group_by<T, K, F>(collection: &Vec<T>, group_key_fn: F) -> HashMap<K, Vec<&T>>
    where K: Hash + Eq,
          F: Fn(&T) -> K
{
    collection
        .iter()
        .map(|e| (group_key_fn(e), e))
        .into_group_map()
}

pub fn map_values<K, V1, V2, F>(collection: HashMap<K, V1>, map_fn: F) -> HashMap<K, V2>
    where K: Hash + Eq,
          F: Fn(V1) -> V2
{
    collection
        .into_iter()
        .map(|(key, value)| {
            (key, map_fn(value))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_first_with_empty_vec() {
        let input = Vec::<i32>::new();
        let actual = first(input);
        assert_eq!(None, actual);
    }

    #[test]
    fn test_first_with_non_empty_vec() {
        let input = vec!["first", "second"];
        let expected = Some("first");
        let actual = first(input);

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_group_by_with_empty_vec() {
        let input = Vec::<i32>::new();
        let expected = HashMap::<bool, Vec<&i32>>::new();
        let actual = group_by(&input, |i| i % 2 == 0);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_group_by_with_non_empty_vec_1() {
        let input = vec![1, 2, 3, 4, 5, 6, 7, 8];

        let expected = hashmap! {
            0 => vec![&3, &6],
            1 => vec![&1, &4, &7],
            2 => vec![&2, &5, &8],
        };

        let actual = group_by(&input, |&i| i % 3);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_group_by_with_non_empty_vec_2() {
        let input = vec!["hello", "wonderful", "world"];

        let expected = hashmap! {
            "length: 5".to_owned() => vec![&"hello", &"world"],
            "length: 9".to_owned() => vec![&"wonderful"],
        };

        let actual = group_by(&input, |i| format!("length: {}", i.len()));
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_map_values_with_empty_input() {
        let input = HashMap::<i32, &str>::new();
        let expected = HashMap::<i32, String>::new();
        let actual = map_values(input, |v| format!("v:{}", v));

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_map_values_with_non_empty_input() {
        let input = hashmap! {
            1 => "one",
            2 => "two",
        };
        let expected = hashmap! {
            1 => "en:one".to_owned(),
            2 => "en:two".to_owned(),
        };
        let actual = map_values(input, |v| format!("en:{}", v));

        assert_eq!(expected, actual);
    }
}