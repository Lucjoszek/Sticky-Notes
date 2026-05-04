use chrono::{Local, NaiveDate};
use serde::{Deserialize, Serialize};
use std::{error::Error, fs};
use uuid::Uuid;

const DEFAULT_DATABASE_PATH: &str = "database.json";

#[derive(Serialize, Deserialize, Debug)]
pub struct Note {
    pub id: Uuid,
    pub title: String,
    pub content: String,
    pub tags: Vec<String>,
    pub modification_date: NaiveDate,
    pub favorite: bool,
}

#[derive(Debug)]
pub enum NotesError {
    NoteNotFound,
}

#[derive(Debug)]
pub struct NotesList {
    pub notes: Vec<Note>,
}

impl Note {
    pub fn new(title: String, content: String, tags: Vec<String>, favorite: bool) -> Self {
        Self {
            id: Uuid::new_v4(),
            title,
            content,
            tags,
            modification_date: Local::now().date_naive(),
            favorite,
        }
    }
}

impl NotesList {
    pub fn new() -> Self {
        Self { notes: vec![] }
    }

    pub fn load(path: Option<&str>) -> Result<Self, Box<dyn Error>> {
        let path = Self::path_or_default(path);

        let data = fs::read_to_string(path)?;
        let notes: Vec<Note> = serde_json::from_str(&data)?;

        Ok(Self { notes })
    }

    pub fn save(&self, path: Option<&str>) -> Result<(), Box<dyn Error>> {
        let path = Self::path_or_default(path);

        let json = serde_json::to_string_pretty(&self.notes)?;
        fs::write(path, json)?;

        Ok(())
    }

    pub fn add(&mut self, note: Note) {
        self.notes.push(note);
    }

    pub fn remove(&mut self, id: &Uuid) -> Result<(), NotesError> {
        let index = self
            .notes
            .iter()
            .position(|n| &n.id == id)
            .ok_or(NotesError::NoteNotFound)?;

        self.notes.remove(index);

        Ok(())
    }

    fn path_or_default(path: Option<&str>) -> &str {
        path.unwrap_or(DEFAULT_DATABASE_PATH)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const TEST_DATABASE_PATH: &str = "test_database.json";

    fn create_sample_note() -> Note {
        Note::new(
            "Test title".to_string(),
            "Test content".to_string(),
            vec!["tag1".into(), "tag2".into()],
            false,
        )
    }

    #[test]
    fn test_note_fields() {
        let note = create_sample_note();

        assert_eq!(note.title, "Test title");
        assert_eq!(note.content, "Test content");
        assert_eq!(note.tags.len(), 2);
        assert!(!note.favorite);
    }

    #[test]
    fn add_note() {
        let mut list = NotesList::new();

        list.add(create_sample_note());

        assert_eq!(list.notes.len(), 1);
    }

    #[test]
    fn remove_note() {
        let mut list = NotesList::new();

        list.add(create_sample_note());
        let id = list.notes[0].id;

        list.remove(&id).unwrap();

        assert_eq!(list.notes.len(), 0);
    }

    #[test]
    fn remove_non_existing_note() {
        let mut list = NotesList::new();
        let random_id = Uuid::new_v4();

        let result = list.remove(&random_id);

        assert!(result.is_err());
    }

    #[test]
    fn save_and_load() {
        let mut list = NotesList::new();
        list.add(create_sample_note());

        list.save(Some(TEST_DATABASE_PATH)).unwrap();
        let loaded = NotesList::load(Some(TEST_DATABASE_PATH)).unwrap();

        assert_eq!(loaded.notes.len(), 1);
        assert_eq!(loaded.notes[0].title, "Test title");
        assert_eq!(loaded.notes[0].content, "Test content");

        std::fs::remove_file(TEST_DATABASE_PATH).unwrap();
    }
}
