//! Fakes of the host for the tests: canned answers by URL and an in-memory storage.

use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};

use crate::host::{Http, Request, Response, Storage};
use crate::model::Error;

pub fn fixture(name: &str) -> String {
    let path = format!("{}/tests/fixtures/{name}", env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(path).expect("a fixture in tests/fixtures")
}

/// Answers by URL, or by URL prefix for a key ending in `*`. Several answers for one key are given in turn, the last
/// one repeating. Unknown URLs fail as a network error.
type Answers = VecDeque<Result<Response, Error>>;

#[derive(Default)]
pub struct FakeHttp {
    routes: RefCell<Vec<(String, Answers)>>,
    sent: RefCell<Vec<Request>>,
}

impl FakeHttp {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn on(self, url: &str, status: u16, body: &str) -> Self {
        self.answer(url, Ok(Response { status, headers: Vec::new(), body: body.as_bytes().to_vec() }))
    }

    pub fn answer(self, url: &str, answer: Result<Response, Error>) -> Self {
        {
            let mut routes = self.routes.borrow_mut();
            match routes.iter_mut().find(|(k, _)| k == url) {
                Some((_, queue)) => queue.push_back(answer),
                None => routes.push((url.to_string(), VecDeque::from([answer]))),
            }
        }
        self
    }

    pub fn sent(&self) -> Vec<Request> {
        self.sent.borrow().clone()
    }

    pub fn urls(&self) -> Vec<String> {
        self.sent.borrow().iter().map(|r| r.url.clone()).collect()
    }
}

impl Http for FakeHttp {
    fn send(&self, request: &Request) -> Result<Response, Error> {
        self.sent.borrow_mut().push(request.clone());
        let mut routes = self.routes.borrow_mut();
        let matches = |k: &String| k == &request.url || k.strip_suffix('*').is_some_and(|p| request.url.starts_with(p));
        let Some((_, queue)) = routes.iter_mut().find(|(k, _)| matches(k)) else {
            return Err(Error::Network(format!("no route to {}", request.url)));
        };
        if queue.len() > 1 { queue.pop_front().expect("not empty") } else { queue[0].clone() }
    }
}

/// A value, its time to live and whether it persists.
pub type Kept = (String, Option<u32>, bool);

#[derive(Default)]
pub struct FakeStorage {
    pub values: RefCell<HashMap<String, Kept>>,
    pub refuse: bool,
}

impl FakeStorage {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with(self, key: &str, value: &str) -> Self {
        self.values.borrow_mut().insert(key.into(), (value.into(), None, false));
        self
    }

    pub fn value(&self, key: &str) -> Option<Kept> {
        self.values.borrow().get(key).cloned()
    }
}

impl Storage for FakeStorage {
    fn get(&self, key: &str) -> Option<String> {
        self.values.borrow().get(key).map(|(v, _, _)| v.clone())
    }

    fn set(&self, key: &str, value: &str, ttl: Option<u32>, persist: bool) -> Result<(), String> {
        if self.refuse {
            return Err("full".into());
        }
        self.values.borrow_mut().insert(key.into(), (value.into(), ttl, persist));
        Ok(())
    }

    fn delete(&self, key: &str) {
        self.values.borrow_mut().remove(key);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn answers_come_in_turn_then_the_last_repeats() {
        let http = FakeHttp::new().on("https://a.test/*", 500, "").on("https://a.test/*", 200, "ok");
        let get = |u: &str| http.send(&Request::get(u)).map(|r| r.status);
        assert_eq!(get("https://a.test/x"), Ok(500));
        assert_eq!(get("https://a.test/y"), Ok(200));
        assert_eq!(get("https://a.test/z"), Ok(200));
        assert!(matches!(get("https://b.test/"), Err(Error::Network(_))));
        assert_eq!(http.urls().len(), 4);
        assert_eq!(http.sent()[0].url, "https://a.test/x");
    }

    #[test]
    fn storage_keeps_values_until_deleted() {
        let storage = FakeStorage::new().with("a", "1");
        assert_eq!(storage.get("a").as_deref(), Some("1"));
        storage.set("b", "2", Some(60), true).unwrap();
        assert_eq!(storage.value("b"), Some(("2".into(), Some(60), true)));
        storage.delete("a");
        assert_eq!(storage.get("a"), None);
        let full = FakeStorage { refuse: true, ..FakeStorage::new() };
        assert_eq!(full.set("c", "3", None, false), Err("full".into()));
    }

    #[test]
    fn fixtures_are_read_from_the_tests_folder() {
        assert!(fixture("README.md").contains("trimmed"));
    }
}
