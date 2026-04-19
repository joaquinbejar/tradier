use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum AccountType {
    Cash,
    Margin,
}

/// Direction for sorting paginated endpoint results.
#[derive(Clone, Debug, PartialEq)]
pub enum SortOrder {
    Asc,
    Desc,
}

impl std::fmt::Display for SortOrder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            SortOrder::Asc => "asc",
            SortOrder::Desc => "desc",
        })
    }
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
