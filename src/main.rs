mod inverted_index;

use std::path::PathBuf;

use inverted_index::InvertedIndex;

fn main() {
  let root_folder = PathBuf::from_iter([std::env::current_dir().unwrap(), PathBuf::from("test")]);
  println!("{:?}", root_folder);
  let index = InvertedIndex::with_root_folder(&root_folder);
  println!("{:?}", index);

  println!("{:?}", index.search("hello"));
  println!("{:?}", index.search("hey"));
}
