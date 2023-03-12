pub mod memory;

use crate::models::note::{Draft, Note};

use crate::models::{Id, Tag, User};

/// The `Persister` trait links the actual business logic from the data
/// storage logic.
///
/// It allows to keep business logic independent from the implementation
/// and allows switching between different data storage technologies.
///
/// It is not the greatest design and more PoC for myself, but it works well in
/// the current context.
///
/// This will obviously not work in larger data environments where each query must be
/// much more restricted or paginated etc.
///
/// # Note
/// The caller must not rely on the implementer for security or validity checks
///
/// The caller must filter the resulting data further in many cases
pub trait Persister<'a> {
    type NoteIter: Iterator<Item = &'a Note>;

    type TagIter: Iterator<Item = &'a Tag>;

    fn notes(&'a self) -> Self::NoteIter;

    fn tags(&'a self) -> Self::TagIter;

    fn add_note(&mut self, draft: Draft, user: &User) -> &Note;

    fn update_note(&mut self, draft: Draft, id: Id) -> &Note;

    fn delete_note(&mut self, id: Id) -> bool;

    fn user_notes(&'a self, user: &User) -> Self::NoteIter;

    fn tagged_notes(&'a self, tag: &Tag) -> Self::NoteIter;

    fn add_tag(&mut self, label: String) -> Id;

    fn note(&'a self, id: Id) -> Option<&Note> {
        self.notes().find(|note| note.id() == &id)
    }

    fn tag(&'a self, label: &str) -> Option<&Tag> {
        self.tags().find(|tag| tag.label() == label)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::models::note::test_note::example_note;
    use crate::models::Tag;

    struct A(Vec<Note>, Vec<Tag>);
    impl<'a> Persister<'a> for A {
        type NoteIter = std::slice::Iter<'a, Note>;
        type TagIter = std::slice::Iter<'a, Tag>;
        fn notes(&'a self) -> Self::NoteIter {
            self.0.iter()
        }
        fn tags(&'a self) -> Self::TagIter {
            self.1.iter()
        }
        fn add_note(&mut self, _draft: Draft, _user: &User) -> &Note {
            unimplemented!()
        }
        fn update_note(&mut self, _draft: Draft, _id: Id) -> &Note {
            unimplemented!()
        }
        fn add_tag(&mut self, _label: String) -> Id {
            unimplemented!()
        }
        fn user_notes(&'a self, _user: &User) -> Self::NoteIter {
            unimplemented!()
        }
        fn tagged_notes(&'a self, _tag: &Tag) -> Self::NoteIter {
            unimplemented!()
        }
        fn delete_note(&mut self, _id: Id) -> bool {
            unimplemented!()
        }
    }

    #[test]
    fn test_note_default() {
        let foo = A(vec![example_note(), example_note()], vec![]);

        assert_eq!(foo.notes().len(), 2);
        assert!(foo.note(Id(2)).is_none());
        assert!(foo.note(Id(1)).is_some());
    }

    #[test]
    fn test_tag_default() {
        let foo = A(
            vec![],
            vec![
                Tag::new(Id(1), "foo".to_string()),
                Tag::new(Id(2), "bar".to_string()),
            ],
        );

        assert_eq!(foo.tags().len(), 2);
        assert!(foo.tag("foobar").is_none());
        assert!(foo.tag("fo").is_none());
        assert!(foo.tag("ar").is_none());
        assert!(foo.tag("").is_none());
        assert!(foo.tag("*").is_none());
        assert!(foo.tag("%").is_none());
        assert!(foo.tag("foo").is_some());
        assert!(foo.tag("bar").is_some());
    }
}
