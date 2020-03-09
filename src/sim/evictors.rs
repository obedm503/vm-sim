use rand::Rng;
use std::collections::HashMap;
use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};

fn now() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis()
}

#[derive(Debug)]
pub struct PageTableEntry {
    pub is_dirty: bool,
    pub created_at: u128,
    pub last_referenced: u128,
}
impl PageTableEntry {
    pub fn new() -> Self {
        return Self {
            is_dirty: false,
            created_at: now(),
            last_referenced: now(),
        };
    }
    pub fn reference(&mut self) {
        self.last_referenced = now();
    }
}

impl fmt::Display for PageTableEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        return write!(
            f,
            "PageTableEntry {{ is_dirty: {} created_at: {:?} last_referenced {:?} }}",
            self.is_dirty, self.created_at, self.last_referenced
        );
    }
}

pub type Memory = Vec<Option<u32>>;
pub type PageTable = HashMap<u32, PageTableEntry>;

// returns Some(index) for the first empty slot, or None if memory is full
fn get_first_empty_index(memory: &Memory) -> Option<usize> {
    return memory.iter().position(|el: &Option<u32>| el.is_none());
}

// pick a memory address to evict
fn evict_random(memory: &Memory) -> usize {
    return rand::thread_rng().gen_range(0, memory.len());
}

fn memory_to_pages<'a>(
    memory: &Memory,
    page_table: &'a PageTable,
) -> Vec<(usize, &'a PageTableEntry)> {
    return memory
        .iter()
        .enumerate()
        .filter_map(|(index, maybe_page): (usize, &Option<u32>)| {
            return maybe_page
                .and_then(|p| page_table.get(&p).and_then(|page| Some((index, page))));
        })
        .collect::<Vec<(usize, &PageTableEntry)>>();
}

fn evict_least_recent(memory: &Memory, page_table: &PageTable) -> usize {
    let mut sorted = memory_to_pages(memory, page_table);

    sorted.sort_by(|a, b| {
        return a.1.last_referenced.cmp(&b.1.last_referenced);
    });

    return sorted.first().unwrap().0;
}

fn evict_fifo(memory: &Memory, page_table: &PageTable) -> usize {
    let mut sorted = memory_to_pages(memory, page_table);

    sorted.sort_by(|a, b| {
        return a.1.created_at.cmp(&b.1.created_at);
    });

    return sorted.first().unwrap().0;
}

pub fn evict(name: &String, memory: &Memory, page_table: &PageTable) -> usize {
    if let Some(n) = get_first_empty_index(&memory) {
        return n;
    }

    return match name.as_str() {
        "random" => evict_random(memory),
        "lru" => evict_least_recent(memory, page_table),
        "fifo" => evict_fifo(memory, page_table),
        v => panic!("{} is not a valid eviction algorithm", v),
    };
}
