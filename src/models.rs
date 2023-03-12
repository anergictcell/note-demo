//! Contains the main data structures used in the app
//!
//! The data structures try to use a style that could support both relational and document-based databases
use serde::{Deserialize, Serialize};

pub mod note;

/// Id represents a foreign and/or primary key
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct Id(pub usize);

impl From<Id> for usize {
    fn from(value: Id) -> Self {
        value.0
    }
}

impl From<&Id> for usize {
    fn from(value: &Id) -> Self {
        value.0
    }
}

impl From<usize> for Id {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

/// Tags are labels added to individual notes
#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct Tag {
    id: Id,
    label: String,
}

impl Tag {
    /// Constructs a new [`Tag`]
    pub fn new(id: Id, label: String) -> Self {
        Self { id, label }
    }

    /// Returns the primary key of the [`Tag`]
    pub fn id(&self) -> &Id {
        &self.id
    }

    /// Returns the label of the [`Tag`]
    pub fn label(&self) -> &str {
        &self.label
    }
}

/// User management is not built in yet and this struct acts only as a placeholder
#[derive(Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct User {
    id: Id,
    name: String,
}

impl User {
    /// Returns the primary key of the [`User`]
    pub fn id(&self) -> &Id {
        &self.id
    }
}

/// [Notes](`note::Note`) can have different types of visibility to be either private or public
/// For simplicity, Visibility can also be used to soft-delete `Note`s.
#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum Visibility {
    #[default]
    Private,
    Public,
    Deleted,
}

#[cfg(test)]
mod test {
    use super::*;

    /// kind of a pointless test....
    #[test]
    fn test_id_conversions() {
        let u = 12usize;

        assert_eq!(Id::from(u), Id(12));
        assert_eq!(usize::from(Id(12)), 12usize);
        assert_eq!(usize::from(&Id(12)), 12usize);
    }

    #[test]
    fn test_id_equality() {
        assert_eq!(Id(12), Id(12));
        assert_ne!(Id(12), Id(66))
    }

    #[test]
    fn test_tag_equality() {
        let tag_a = Tag::new(Id(12), "foobar".to_string());
        assert_eq!(tag_a, tag_a);
        assert_eq!(tag_a, Tag::new(Id(12), "foobar".to_string()));
        assert_eq!(
            Tag::new(Id(1), "foo".to_string()),
            Tag::new(Id(1), "foo".to_string())
        );
        assert_ne!(tag_a, Tag::new(Id(1), "foo".to_string()));
        assert_ne!(tag_a, Tag::new(Id(12), "foo".to_string()));
        assert_ne!(tag_a, Tag::new(Id(1), "foobar".to_string()));
    }
}
