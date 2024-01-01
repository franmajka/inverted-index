mod inverted_index;
mod server;

#[cfg(test)]
mod tests;

use std::{
  path::PathBuf,
  sync::Arc,
  time::Instant,
};

use serde::Serialize;
use threadpool::ThreadPool;

fn main() {
  let root_folder = PathBuf::from_iter([
    std::env::current_dir().unwrap(),
    PathBuf::from("datasets/aclImdb")
  ]);

  let start_time = Instant::now();
  let thread_pool = Arc::new(ThreadPool::new(3));
  let index = Arc::from(
    inverted_index::Builder::new()
      .thread_pool(Arc::clone(&thread_pool))
      .root_folder(root_folder)
      .build()
  );

  println!("Indexing took {}s", start_time.elapsed().as_secs_f64());

  server::run_server(thread_pool, index);
}

#[derive(Serialize)]
struct Record {
  n_files: usize,
  n_threads: u8,
  duration: f64,
}

fn bench() {
  let mut writer = csv::Writer::from_path("stats.csv").unwrap();

  let mut index = inverted_index::InvertedIndex::new();

  for (folder, n_files) in [
    ("datasets/aclImdb/test/pos", 12_500),
    ("datasets/aclImdb/test", 25_000),
    ("datasets/aclImdb/train/unsup", 50_000),
    ("datasets/aclImdb/train", 75_000),
    ("datasets/aclImdb", 100_000),
  ] {
    let root_folder = PathBuf::from_iter([
      std::env::current_dir().unwrap(),
      PathBuf::from(folder),
    ]);

    let start_time = Instant::now();

    index.clear();
    index.index_folder(&root_folder);
    writer.serialize(Record {
      n_files,
      n_threads: 1,
      duration: start_time.elapsed().as_secs_f64(),
    }).unwrap();

    for n_threads in [2u8, 3, 4, 6, 8, 16, 32] {
      let thread_pool = Arc::new(ThreadPool::new(n_threads.into()));
      index.clear();
      index.set_thread_pool(thread_pool);

      let start_time = Instant::now();
      index.index_folder_parallel(&root_folder);
      writer.serialize(Record {
        n_files,
        n_threads,
        duration: start_time.elapsed().as_secs_f64(),
      }).unwrap();
    }
  }
}
