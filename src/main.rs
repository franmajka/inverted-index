mod inverted_index;
mod server;

#[cfg(test)]
mod tests;

use std::{
  path::PathBuf,
  sync::Arc,
  time::Instant,
};

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
