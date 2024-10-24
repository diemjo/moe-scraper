use strum_macros::{Display, EnumString};

#[derive(Debug, Clone, PartialEq, Eq, Display, EnumString)]
pub enum Availability {
    Available,
    Preorder,
    NotAvailable,
    Deleted,
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
