use std::str::FromStr;

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

/// A ticker symbol (equity, option, index, etc.).
///
/// Tradier symbols are ASCII printable strings; this wrapper validates the
/// input and rejects empty / blank values.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Symbol(String);

impl Symbol {
    /// Returns the inner string slice.
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl FromStr for Symbol {
    type Err = crate::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.trim().is_empty() {
            Err(crate::Error::MarketDataParseError(format!(
                "invalid symbol: '{s}' must not be empty or blank"
            )))
        } else if s.chars().all(|c| (0x20u8..0x7fu8).contains(&(c as u8))) {
            Ok(Self(s.to_string()))
        } else {
            Err(crate::Error::MarketDataParseError(format!(
                "invalid symbol: '{s}' must be printable ASCII"
            )))
        }
    }
}

impl std::fmt::Display for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// Comma-separated collection of [`Symbol`]s used by multi-symbol endpoints.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Symbols(Vec<Symbol>);

impl Symbols {
    /// Creates a new [`Symbols`] from an iterator of [`Symbol`].
    #[must_use]
    pub fn new(symbols: impl IntoIterator<Item = Symbol>) -> Self {
        Self(symbols.into_iter().collect())
    }

    /// Returns the inner slice of [`Symbol`] values.
    #[inline]
    #[must_use]
    pub fn as_slice(&self) -> &[Symbol] {
        &self.0
    }

    /// Returns `true` if no symbols have been added.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl<'a> From<&'a [Symbol]> for Symbols {
    fn from(value: &'a [Symbol]) -> Self {
        Self(value.to_vec())
    }
}

impl std::fmt::Display for Symbols {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut first = true;
        for s in &self.0 {
            if !first {
                f.write_str(",")?;
            }
            f.write_str(s.as_str())?;
            first = false;
        }
        Ok(())
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

#[cfg(test)]
mod symbol_tests {
    use super::*;

    #[test]
    fn test_symbol_empty_should_error() {
        let result: Result<Symbol, _> = "".parse();
        assert!(result.is_err());
        let result: Result<Symbol, _> = "   ".parse();
        assert!(result.is_err());
    }

    #[test]
    fn test_symbol_valid_should_succeed() {
        let result: Result<Symbol, _> = "AAPL".parse();
        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_str(), "AAPL");
    }

    #[test]
    fn test_symbol_rejects_non_ascii() {
        let result: Result<Symbol, _> = "BAD\u{80}".parse();
        assert!(result.is_err());
    }

    #[test]
    fn test_symbols_display_comma_separated() {
        let syms = Symbols::new(vec![
            "AAPL".parse().unwrap(),
            "MSFT".parse().unwrap(),
            "GOOG".parse().unwrap(),
        ]);
        assert_eq!(syms.to_string(), "AAPL,MSFT,GOOG");
    }

    #[test]
    fn test_symbols_empty_display() {
        let syms = Symbols::default();
        assert_eq!(syms.to_string(), "");
        assert!(syms.is_empty());
    }

    #[test]
    fn test_symbols_from_slice() {
        let arr = [
            "AAPL".parse::<Symbol>().unwrap(),
            "MSFT".parse::<Symbol>().unwrap(),
        ];
        let syms: Symbols = (&arr[..]).into();
        assert_eq!(syms.to_string(), "AAPL,MSFT");
    }
}
