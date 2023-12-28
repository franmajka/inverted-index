use std::{collections::{HashMap, HashSet}, path::PathBuf, rc::Rc};

#[derive(Debug)]
pub struct InvertedIndex {
  index: HashMap<String, HashSet<Rc<PathBuf>>>,
}

impl InvertedIndex {
  pub fn new() -> Self {
    Self {
      index: HashMap::new(),
    }
  }

  pub fn with_root_folder(root_folder: &PathBuf) -> Self {
    let mut index = Self::new();
    index.index_folder(root_folder);
    index
  }

  pub fn index_folder(&mut self, folder: &PathBuf) {
    if !folder.is_dir() {
      return;
    }

    for entry in std::fs::read_dir(folder).unwrap() {
      let path = entry.unwrap().path();

      if path.is_dir() {
        self.index_folder(&path);
      } else if path.is_file() {
        if let Some(ext) = path.extension() {
          if ext == "txt" {
            self.index_file(path);
          }
        }
      }
    }
  }

  pub fn index_file(&mut self, path: PathBuf) {
    if !path.is_file() {
      return;
    }

    let mut words = HashSet::new();

    if let Ok(contents) = std::fs::read_to_string(&path) {
      for word in contents.split_whitespace() {
        let mut word = word.to_lowercase();
        word.retain(|c| c.is_alphanumeric());

        if word.len() > 0 {
          words.insert(word);
        }
      }
    }

    self.insert_words(words, path);
  }

  fn insert_words(&mut self, words: HashSet<String>, path: PathBuf) {
    let path_rc = Rc::new(path);
    for word in words {
      let paths = self.index.entry(word).or_insert_with(HashSet::new);
      paths.insert(Rc::clone(&path_rc));
    }
  }

  pub fn search(&self, word: &str) -> Option<&HashSet<Rc<PathBuf>>> {
    self.index.get(word)
  }
}
