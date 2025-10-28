use serde::Deserialize;
use strum_macros::{Display, EnumIter, EnumString};

#[derive(Debug, Clone, PartialEq, Eq, Display, EnumString, Deserialize, EnumIter)]
pub enum Availability {
    Available,
    Preorder,
    NotAvailable,
    Deleted,
}

impl Availability {
    pub fn is_available(&self) -> bool {
        match self { 
            Availability::Available | Availability::Preorder => true,
            Availability::NotAvailable | Availability::Deleted => false,
        }
    }
}

impl TryFrom<String> for Availability {
    type Error = strum::ParseError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}

impl From<Availability> for String {
    fn from(value: Availability) -> Self {
        value.to_string()
    }
}
