use crate::models::note::{Draft, Note, Tags};
use crate::models::{Id, Tag, User, Visibility};

use crate::persistence::Persister;

#[derive(Debug, Default)]
pub struct InMemoryStorage {
    notes: Vec<Note>,
    tags: Vec<Tag>,
}

impl InMemoryStorage {
    fn map_tags(&mut self, labels: &Vec<String>) -> Tags {
        let mut tags = Tags::default();
        for label in labels {
            if let Some(tag) = self
                .tags
                .iter()
                .find(|existing_tag| existing_tag.label() == label)
            {
                tags.insert(tag.clone());
            } else {
                let tag = Tag::new(self.tags.len().into(), label.to_string());
                tags.insert(tag.clone());
                self.tags.push(tag);
            }
        }
        tags
    }
}

// we have to use two references `&&Note` because we're using `active`
// as a closure and have no control over the input
fn active(note: &&Note) -> bool {
    note.visibility() != &Visibility::Deleted
}

impl<'a> Persister<'a> for InMemoryStorage {
    type NoteIter = std::vec::IntoIter<&'a Note>;
    type TagIter = std::slice::Iter<'a, Tag>;

    fn notes(&'a self) -> Self::NoteIter {
        let res = self.notes.iter().filter(active).collect::<Vec<&Note>>();
        res.into_iter()
    }

    fn tags(&'a self) -> Self::TagIter {
        self.tags.iter()
    }

    fn add_note(&mut self, draft: Draft, user: &User) -> &Note {
        let id = Id(self.notes.len());
        let tags = self.map_tags(draft.tags());
        let note = Note::new(draft, id, *user.id(), tags);
        self.notes.push(note);
        self.note(id)
            .expect("Note was just added and must be present")
    }

    fn update_note(&mut self, draft: Draft, id: Id) -> &Note {
        let index: usize = id.into();
        let tags = self.map_tags(draft.tags());
        if let Some(note) = self.notes.get_mut(index) {
            let user = note.user();
            // in this PoC, we don't update fields individually, but simply
            // replace the whole `Note`
            let new_note = Note::new(draft, index.into(), *user, tags);
            *note = new_note;
            note
        } else {
            // TODO: Error handling
            panic!("Note does not exist")
        }
    }

    fn delete_note(&mut self, id: Id) -> bool {
        let idx: usize = id.into();
        if let Some(item) = self.notes.get_mut(idx) {
            *item.visibility_mut() = Visibility::Deleted;
            true
        } else {
            false
        }
    }

    fn user_notes(&'a self, user: &User) -> Self::NoteIter {
        let userid = user.id();
        let res = self
            .notes
            .iter()
            .filter(|note| note.user() == userid)
            .filter(active)
            .collect::<Vec<&Note>>();
        res.into_iter()
    }

    fn tagged_notes(&'a self, tag: &Tag) -> <Self as Persister>::NoteIter {
        let res = self
            .notes
            .iter()
            .filter(|note| note.tagged_with(tag))
            .filter(active)
            .collect::<Vec<&Note>>();
        res.into_iter()
    }

    fn add_tag(&mut self, label: String) -> Id {
        for existing_tag in &self.tags {
            if existing_tag.label() == label {
                return *existing_tag.id();
            }
        }
        let id = Id(self.tags.len());
        self.tags.push(Tag::new(id, label));
        id
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn add_notes() {
        let mut data = InMemoryStorage::default();

        assert_eq!(data.notes.len(), 0);
        let _ = data.add_note(Draft::default(), &User::default());
        assert_eq!(data.notes.len(), 1);
        let _ = data.add_note(Draft::default(), &User::default());
        assert_eq!(data.notes.len(), 2);
        assert_eq!(data.notes().len(), 2);
        assert_eq!(data.tags().len(), 0);

        let mut iter = data.notes();
        let first = iter.next().unwrap();
        assert_eq!(first.id(), &Id(0));
        assert_eq!(first.user(), &Id(0));
        let second = iter.next().unwrap();
        assert_eq!(second.id(), &Id(1));
        assert_eq!(second.user(), &Id(0));
        assert!(iter.next().is_none());
    }

    #[test]
    fn add_notes_with_tags() {
        let mut data = InMemoryStorage::default();

        assert_eq!(data.notes.len(), 0);
        assert_eq!(data.tags.len(), 0);

        let _ = data.add_note(
            Draft::new(
                "Foo".to_string(),
                "Foo".to_string(),
                vec!["foo".to_string(), "bar".to_string()],
                Visibility::Public,
            ),
            &User::default(),
        );
        assert_eq!(data.notes.len(), 1);
        assert_eq!(data.tags.len(), 2);
        let _ = data.add_note(
            Draft::new(
                "Foo".to_string(),
                "Foo".to_string(),
                vec!["foo".to_string(), "bar".to_string()],
                Visibility::Public,
            ),
            &User::default(),
        );
        assert_eq!(data.notes.len(), 2);
        assert_eq!(data.tags.len(), 2);
    }

    #[test]
    fn edit_notes() {
        let mut data = InMemoryStorage::default();

        let _ = data.add_note(Draft::default(), &User::default());
        let _ = data.add_note(Draft::default(), &User::default());
        let _ = data.add_note(Draft::default(), &User::default());

        let res = data
            .update_note(
                Draft::new(
                    "FooTitle".to_string(),
                    "BarBody".to_string(),
                    vec!["foo".to_string(), "bar".to_string()],
                    Visibility::Public,
                ),
                Id(1),
            )
            .clone();

        let res2 = data.note(Id(1)).unwrap();
        assert_eq!(&res, res2);

        assert_eq!(res.title(), "FooTitle");
        assert_eq!(res.body(), "BarBody");

        assert_eq!(data.notes.len(), 3);
        assert_eq!(data.tags.len(), 2);
    }

    #[test]
    fn delete_notes() {
        let mut data = InMemoryStorage::default();

        let _ = data.add_note(Draft::default(), &User::default());
        let _ = data.add_note(Draft::default(), &User::default());
        let _ = data.add_note(Draft::default(), &User::default());

        assert!(data.delete_note(Id(1)));
        assert_eq!(data.notes.len(), 3);
        assert_eq!(data.notes().len(), 2);

        assert!(!data.delete_note(Id(666)));
        assert_eq!(data.notes.len(), 3);
        assert_eq!(data.notes().len(), 2);
    }

    #[test]
    fn tagged_notes() {
        let mut data = InMemoryStorage::default();

        assert_eq!(data.notes.len(), 0);
        assert_eq!(data.tags.len(), 0);

        let _ = data.add_note(
            Draft::new(
                "Foo".to_string(),
                "Foo".to_string(),
                vec!["foo".to_string()],
                Visibility::Public,
            ),
            &User::default(),
        );
        let _ = data.add_note(
            Draft::new(
                "Foo".to_string(),
                "Foo".to_string(),
                vec!["foo".to_string(), "bar".to_string()],
                Visibility::Public,
            ),
            &User::default(),
        );

        assert_eq!(data.tagged_notes(data.tag("foo").unwrap()).len(), 2);
        assert_eq!(data.tagged_notes(data.tag("bar").unwrap()).len(), 1);
    }
}
