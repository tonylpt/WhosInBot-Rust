use std::cmp::Eq;
use std::collections::HashMap;
use std::hash::{BuildHasher, Hash};

use itertools::Itertools;

pub trait CollectionTools: IntoIterator {
    fn take_first(self) -> Option<Self::Item>
    where
        Self: Sized,
    {
        self.into_iter().next()
    }

    fn into_groups_by<K, F>(self, group_key_fn: F) -> HashMap<K, Vec<Self::Item>>
    where
        Self: Sized,
        K: Hash + Eq,
        F: Fn(&Self::Item) -> K,
    {
        self.into_iter()
            .map(|e| (group_key_fn(&e), e))
            .into_group_map()
    }

    fn map_values<K, V1, V2, F>(self, map_fn: F) -> HashMap<K, V2>
    where
        Self: IntoIterator<Item = (K, V1)> + Sized,
        K: Hash + Eq,
        F: Fn(V1) -> V2,
    {
        self.into_iter()
            .map(|(key, value)| (key, map_fn(value)))
            .collect()
    }
}

impl<'a, T> CollectionTools for std::slice::Iter<'a, T> where Self: Sized {}

impl<T> CollectionTools for Vec<T> {}

impl<K, V, S> CollectionTools for HashMap<K, V, S>
where
    Self: IntoIterator,
    S: BuildHasher,
{
}

#[cfg(test)]
mod tests {
    use super::CollectionTools;
    use super::*;

    #[test]
    fn test_first_with_empty_vec() {
        let input = Vec::<i32>::new();
        let actual = input.take_first();
        assert_eq!(None, actual);
    }

    #[test]
    fn test_first_with_non_empty_vec() {
        let input = vec!["first", "second"];
        let expected = Some("first");
        let actual = input.take_first();

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_into_groups_by_with_empty_vec() {
        let input = Vec::<i32>::new();
        let expected = HashMap::<bool, Vec<i32>>::new();
        let actual = input.into_groups_by(|i| i % 2 == 0);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_into_groups_by_with_non_empty_vec_1() {
        let input = vec![1, 2, 3, 4, 5, 6, 7, 8];

        let expected = hashmap! {
            0 => vec![3, 6],
            1 => vec![1, 4, 7],
            2 => vec![2, 5, 8],
        };

        let actual = input.into_groups_by(|&i| i % 3);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_into_groups_by_with_non_empty_vec_2() {
        let input = vec!["hello", "wonderful", "world"];

        let expected = hashmap! {
            "length: 5".to_owned() => vec!["hello", "world"],
            "length: 9".to_owned() => vec!["wonderful"],
        };

        let actual = input.into_groups_by(|i| format!("length: {}", i.len()));
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_map_values_with_empty_input() {
        let input = HashMap::<i32, &str>::new();
        let expected = HashMap::<i32, String>::new();
        let actual = input.map_values(|v| format!("v:{}", v));

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
        let actual = input.map_values(|v| format!("en:{}", v));

        assert_eq!(expected, actual);
    }
}
