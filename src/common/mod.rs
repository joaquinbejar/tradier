use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum AccountType {
    Cash,
    Margin,
}

#[cfg(test)]
pub mod test_support {
    use serde::Serialize;

    #[derive(Debug, Serialize, proptest_derive::Arbitrary)]
    #[serde(rename_all = "lowercase")]
    pub enum AccountTypeWire {
        Cash,
        Margin,
    }
}
