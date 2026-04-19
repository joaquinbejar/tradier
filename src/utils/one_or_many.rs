use serde::Deserialize;

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum OneOrMany<T> {
    One(T),
    Many(Vec<T>),
}

impl<T> OneOrMany<T> {
    pub fn into_vec(self) -> Vec<T> {
        match self {
            OneOrMany::One(thing) => vec![thing],
            OneOrMany::Many(things) => things,
        }
    }
}

impl<T> Default for OneOrMany<T> {
    fn default() -> Self {
        OneOrMany::Many(Vec::new())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[derive(Debug, Deserialize)]
    struct Test {
        items: OneOrMany<i32>,
    }

    #[test]
    fn shold_turn_single_item_into_vec_of_size_one() {
        let thing = OneOrMany::One(4);
        assert_eq!(thing.into_vec().len(), 1);
    }

    #[test]
    fn should_turn_many_items_into_vec_with_length_greater_than_one() {
        let things = OneOrMany::Many(vec![1, 2, 3, 4, 5]);

        assert_eq!(things.into_vec().len(), 5);
    }

    #[test]
    fn should_deserialize_correctly() {
        let serialized_single = serde_json::json!({"items": 5});
        let serialized_many = serde_json::json!({"items": [1, 2, 3]});

        let actual_deserialized = serde_json::from_value::<Test>(serialized_single);
        assert!(actual_deserialized.is_ok());
        assert_eq!(actual_deserialized.unwrap().items.into_vec().len(), 1);

        let actual_deserialized = serde_json::from_value::<Test>(serialized_many);
        assert!(actual_deserialized.is_ok());
        assert_eq!(actual_deserialized.unwrap().items.into_vec().len(), 3);
    }

    #[test]
    fn should_deserialize_empty_correctly() {
        let serialized_empty_array = serde_json::json!({"items": []});

        let actual_deserialized = serde_json::from_value::<Test>(serialized_empty_array);
        assert!(actual_deserialized.is_ok());
        assert!(actual_deserialized.unwrap().items.into_vec().is_empty());
    }
}
