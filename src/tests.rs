use crate::*;

#[test]
fn compare_indexes() {
  let mut index = inverted_index::InvertedIndex::new();
  let mut index_parallel = inverted_index::InvertedIndex::with_thread_pool(
    Arc::new(ThreadPool::new(3))
  );

  let root_folder = PathBuf::from_iter([
    std::env::current_dir().unwrap(),
    PathBuf::from("datasets/aclImdb/train/pos")
  ]);

  index.index_folder(&root_folder);
  index_parallel.index_folder_parallel(&root_folder);

  assert_eq!(index, index_parallel);
}
