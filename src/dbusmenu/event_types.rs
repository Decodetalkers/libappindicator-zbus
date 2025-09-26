use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use zbus::zvariant::{OwnedValue, Type, Value};

#[derive(Deserialize, Serialize, Type, PartialEq, Debug, Value, Clone, Copy, OwnedValue, Default)]
#[zvariant(signature = "s", rename_all = "lowercase")]
pub enum ToggleType {
    Checkmark,
    #[default]
    Radio,
    #[zvariant(rename = "")]
    None,
}

#[derive(Deserialize_repr, Serialize_repr, Type, Debug, OwnedValue, Value, Default)]
#[repr(i32)]
pub enum ToggleState {
    #[default]
    UnSelected,
    Selected,
    TriState,
}
