use serde_repr::{Deserialize_repr, Serialize_repr};
use zbus::zvariant::{OwnedValue, Type};

#[derive(Deserialize_repr, Serialize_repr, Type, Debug, OwnedValue)]
#[repr(u8)]
pub enum ToggleStatus {
    UnSelected,
    Selected,
    TriState,
}
