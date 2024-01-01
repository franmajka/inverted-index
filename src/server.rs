use std::{
  io::prelude::*,
  sync::Arc,
  net::{TcpStream, TcpListener},
  time::{Duration, Instant}
};

use serde::{Deserialize, Serialize};
use threadpool::ThreadPool;

use crate::inverted_index;

#[derive(Debug, PartialEq, Deserialize, Serialize)]
struct Query {
  query: String,
}

type Response = String;

const HTTP_VERSION: &str = "HTTP/1.1";

const GET: &str = "GET";

const OK: &str = "200 OK";
const BAD_REQUEST: &str = "400 BAD REQUEST";
const NOT_FOUND: &str = "404 NOT FOUND";

const SLEEP: &str = "/sleep";
const SEARCH: &str = "/search";

pub fn run_server(thread_pool: Arc<ThreadPool>, index: Arc<inverted_index::InvertedIndex>) {
  let start_time = Instant::now();

  let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

  println!("Server running at {}", listener.local_addr().unwrap());

  for stream in listener.incoming() {
    let stream = stream.unwrap();

    let start_time_clone = start_time.clone();
    let index_clone = Arc::clone(&index);

    thread_pool.execute(move || {
      println!("[{}]: Received request", start_time_clone.elapsed().as_secs_f32());
      handle_connection(stream, index_clone);
      println!("[{}]: Request handled", start_time_clone.elapsed().as_secs_f32());
    });

    println!("[{}]: Request got to thread pool", start_time.elapsed().as_secs_f32());
    println!("[{}]: Thread pool size: {}", start_time.elapsed().as_secs_f32(), thread_pool.active_count());
  }

  println!("Shutting down...");
  thread_pool.join();
}

fn handle_connection(mut stream: TcpStream, index: Arc<inverted_index::InvertedIndex>) {
  let mut buffer = [0; 1024];
  stream.read(&mut buffer).unwrap();

  let first_line = String::from_utf8_lossy(
    &buffer[..buffer.iter().position(|&x| x == b'\r').unwrap_or(buffer.len())]
  );

  let response = if first_line.starts_with(GET) {
    let path = first_line.split_whitespace().nth(1).unwrap();

    if path == SLEEP {
      sleep_handler()
    } else if path.starts_with(SEARCH) {
      search_handler(path, index)
    } else {
      format!("{HTTP_VERSION} {NOT_FOUND}")
    }
  } else {
    format!("{HTTP_VERSION} {NOT_FOUND}")
  };

  stream.write_all(response.as_bytes()).unwrap();
  stream.flush().unwrap();
}

fn sleep_handler() -> Response {
  std::thread::sleep(Duration::from_secs(10));
  format!("{HTTP_VERSION} {OK}")
}

fn search_handler(path: &str, index: Arc<inverted_index::InvertedIndex>) -> Response {
  let ok = format!("{HTTP_VERSION} {OK}");
  let not_found = format!("{HTTP_VERSION} {NOT_FOUND}");
  let bad_request = format!("{HTTP_VERSION} {BAD_REQUEST}");

  let Some(qs) = path.split('?').nth(1) else { return bad_request };
  let Ok(Query { query }) = serde_qs::from_str::<Query>(&qs) else { return bad_request };

  let Some(results) = index.search(&query) else { return not_found };

  let res = serde_json::to_string(
    &results.iter()
      .take(100)
      .map(|path| path.to_str().unwrap())
      .collect::<Vec<&str>>()
  ).unwrap();

  format!("{ok}\r\nContent-Length: {content_length}\r\n\r\n{res}", content_length = res.len())
}
