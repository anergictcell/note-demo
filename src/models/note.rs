use std::{collections::HashSet, fmt::Display};

use serde::{Deserialize, Serialize};

use crate::models::{Id, Tag, Visibility};

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct Tags(HashSet<Tag>);

impl Tags {
    pub fn insert(&mut self, tag: Tag) -> bool {
        self.0.insert(tag)
    }

    pub fn contains(&self, tag: &Tag) -> bool {
        self.0.contains(tag)
    }
}

impl<'a> IntoIterator for &'a Tags {
    type Item = &'a Tag;
    type IntoIter = TagIter<'a>;
    fn into_iter(self) -> Self::IntoIter {
        TagIter(self.0.iter())
    }
}

pub struct TagIter<'a>(std::collections::hash_set::Iter<'a, Tag>);
impl<'a> Iterator for TagIter<'a> {
    type Item = &'a Tag;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

#[derive(Clone, Default, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Draft {
    title: String,
    body: String,
    tags: Vec<String>,
    visibility: Visibility,
}

impl Draft {
    #[allow(dead_code)] // needed for unittests
    pub fn new(title: String, body: String, tags: Vec<String>, visibility: Visibility) -> Self {
        Self {
            title,
            body,
            tags,
            visibility,
        }
    }
    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn tags(&self) -> &Vec<String> {
        &self.tags
    }
}

impl From<&Note> for Draft {
    fn from(note: &Note) -> Self {
        Self {
            title: note.title().to_string(),
            body: note.body().to_string(),
            tags: note.tags().map(|tag| tag.label().to_string()).collect(),
            visibility: note.visibility().clone(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Note {
    id: Id,
    title: String,
    body: String,
    tags: Tags,
    user: Id,
    visibility: Visibility,
}

impl Note {
    pub fn new(draft: Draft, id: Id, user: Id, tags: Tags) -> Self {
        Self {
            id,
            title: draft.title,
            body: draft.body,
            tags,
            user,
            visibility: draft.visibility,
        }
    }

    pub fn id(&self) -> &Id {
        &self.id
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn body(&self) -> &str {
        &self.body
    }

    pub fn tagged_with(&self, tag: &Tag) -> bool {
        self.tags.contains(tag)
    }

    pub fn user(&self) -> &Id {
        &self.user
    }

    pub fn visibility_mut(&mut self) -> &mut Visibility {
        &mut self.visibility
    }

    pub fn visibility(&self) -> &Visibility {
        &self.visibility
    }

    pub fn tags(&self) -> TagIter {
        self.tags.into_iter()
    }
}

impl Display for Note {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.title(), self.body())
    }
}

#[cfg(test)]
pub mod test_note {
    use super::*;

    pub fn example_note() -> Note {
        let mut tags = Tags::default();
        tags.insert(Tag::new(1.into(), "tag1".to_string()));
        tags.insert(Tag::new(1.into(), "tag2".to_string()));
        tags.insert(Tag::new(1.into(), "tag3".to_string()));

        Note {
            id: Id(1),
            title: "Test-Title".into(),
            body: "Test-Body".into(),
            tags,
            user: Id(12),
            visibility: Visibility::Public,
        }
    }

    #[test]
    fn test_tags() {
        let note = example_note();

        assert!(note.tagged_with(&Tag::new(1.into(), "tag1".to_string())));
        assert!(!note.tagged_with(&Tag::new(666.into(), "tag1".to_string())));
        assert!(!note.tagged_with(&Tag::new(1.into(), "foobar".to_string())));

        assert!(note.tagged_with(&Tag::new(1.into(), "tag2".to_string())));
    }

    #[test]
    fn test_tag_iterator() {
        let note = example_note();

        let mut tags = note.tags();
        // all fake tags have ID 1
        assert_eq!(tags.next().unwrap().id(), &Id(1));
        assert!(tags.next().is_some());
        assert!(tags.next().is_some());
        assert!(tags.next().is_none());
    }

    #[test]
    fn test_note_to_draft() {
        let note = example_note();
        let draft = Draft::from(&note);

        assert_eq!(draft.title(), "Test-Title");
        assert_eq!(draft.tags().len(), 3);
    }
}
