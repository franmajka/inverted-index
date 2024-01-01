use std::{collections::{HashMap, HashSet}, path::PathBuf, sync::{Arc, mpsc::channel}};

use threadpool::ThreadPool;

pub struct Builder {
  thread_pool: Option<Arc<ThreadPool>>,
  root_folder: Option<PathBuf>,
}

impl Builder {
  pub fn new() -> Self {
    Self {
      thread_pool: None,
      root_folder: None,
    }
  }

  pub fn thread_pool(mut self, thread_pool: Arc<ThreadPool>) -> Self {
    self.thread_pool = Some(thread_pool);
    self
  }

  pub fn root_folder(mut self, root_folder: PathBuf) -> Self {
    self.root_folder = Some(root_folder);
    self
  }

  pub fn build(self) -> InvertedIndex {
    let mut index = InvertedIndex {
      thread_pool: self.thread_pool,
      index: HashMap::new(),
    };

    if let Some(root_folder) = self.root_folder {
      match index.thread_pool {
        Some(_) => index.index_folder_parallel(&root_folder),
        None => index.index_folder(&root_folder),
      };
    }

    index
  }

}

#[derive(Debug)]
pub struct InvertedIndex {
  thread_pool: Option<Arc<ThreadPool>>,
  index: HashMap<String, HashSet<Arc<PathBuf>>>,
}

impl InvertedIndex {
  pub fn new() -> Self {
    Builder::new().build()
  }

  pub fn with_thread_pool(thread_pool: Arc<ThreadPool>) -> Self {
    Builder::new()
      .thread_pool(thread_pool)
      .build()
  }

  pub fn set_thread_pool(&mut self, thread_pool: Arc<ThreadPool>) {
    self.thread_pool = Some(thread_pool);
  }

  pub fn index_folder(&mut self, folder: &PathBuf) {
    if !folder.is_dir() {
      return;
    }

    for entry in std::fs::read_dir(folder).unwrap() {
      let path = entry.unwrap().path();

      if path.is_dir() {
        self.index_folder(&path);
      } else if path.is_file() && matches!(path.extension(), Some(ext) if ext == "txt") {
        self.index_file(path);
      }
    }
  }

  pub fn index_folder_parallel(&mut self, folder: &PathBuf) {
    if !folder.is_dir() {
      return;
    }

    let thread_pool = self.thread_pool.as_ref()
      .expect("Thread pool has to be set to perform parallel indexing");
    let (sender, receiver) = channel();

    let mut stack = vec![folder.clone()];

    while let Some(dir) = stack.pop() {
      for entry in std::fs::read_dir(&dir).unwrap() {
        let path = entry.unwrap().path();

        if path.is_dir() {
          stack.push(path);
        } else if path.is_file() && matches!(path.extension(), Some(ext) if ext == "txt") {
          let tx = sender.clone();
          thread_pool.execute(move || {
            tx.send((InvertedIndex::read_words(&path), path)).unwrap();
          });
        }
      }
    }

    drop(sender);

    for (words, path) in receiver {
      self.insert_words(words, path);
    }
  }

  pub fn index_file(&mut self, path: PathBuf) {
    if !path.is_file() {
      return;
    }

    self.insert_words(InvertedIndex::read_words(&path), path);
  }

  fn read_words(path: &PathBuf) -> HashSet<String> {
    std::fs::read_to_string(&path)
      .unwrap_or(String::new())
      .split_whitespace()
      .map(|word| word.chars()
        .filter(|c| c.is_alphanumeric())
        .collect::<String>()
        .to_lowercase()
      )
      .filter(|word| word.len() > 0)
      .collect()
  }

  fn insert_words(&mut self, words: HashSet<String>, path: PathBuf) {
    let path_arc = Arc::new(path);
    for word in words {
      let paths = self.index.entry(word)
        .or_insert_with(HashSet::new);
      paths.insert(Arc::clone(&path_arc));
    }
  }

  pub fn search(&self, word: &str) -> Option<&HashSet<Arc<PathBuf>>> {
    self.index.get(word)
  }

  pub fn clear(&mut self) {
    self.index.clear();
  }
}

impl PartialEq for InvertedIndex {
  fn eq(&self, other: &Self) -> bool {
    self.index == other.index
  }
}
