mod inverted_index;

use std::{path::PathBuf, sync::Arc};

use threadpool::ThreadPool;

fn main() {
  let root_folder = PathBuf::from_iter([std::env::current_dir().unwrap(), PathBuf::from("datasets/aclImdb")]);
  // println!("{:?}", root_folder);

  let start_time = std::time::Instant::now();
  let thread_pool = Arc::new(ThreadPool::new(3));
  let index = inverted_index::Builder::new()
    .thread_pool(thread_pool)
    .root_folder(root_folder)
    .build();

  println!("Indexing took {}s", start_time.elapsed().as_secs_f64());
  // println!("{:#?}", index);

  // println!("{:?}", index.search("apple"));
  // println!("{:?}", index.search("hey"));
}
