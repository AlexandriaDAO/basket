//! Cache management for informational queries

use std::cell::RefCell;
use std::collections::HashMap;

thread_local! {
    static CACHE_ENTRIES: RefCell<HashMap<String, (Vec<u8>, u64)>> =
        RefCell::new(HashMap::new());
}

/// Clear all cached entries
pub fn clear_all_caches() {
    CACHE_ENTRIES.with(|cache| {
        cache.borrow_mut().clear();
    });
    ic_cdk::println!("All caches cleared");
}
