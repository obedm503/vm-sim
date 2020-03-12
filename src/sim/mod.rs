use std::fmt;
use std::fs;
use std::io;
use std::io::BufRead;

mod evictors;
use evictors::{evict, Memory, PageTable, PageTableEntry};

fn read_file(path: String) -> io::Lines<io::BufReader<fs::File>> {
    let file = match fs::File::open(&path) {
        Ok(f) => f,
        Err(e) => panic!("could not read file {} {}", path, e),
    };

    let reader = io::BufReader::new(file);
    return reader.lines();
}

pub enum Op {
    R,
    W,
}
impl fmt::Display for Op {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        return match &self {
            Op::R => write!(f, "Read"),
            Op::W => write!(f, "Write"),
        };
    }
}

pub struct Operation {
    pub virtual_address: u32,
    pub virtual_page_number: u32,
    pub page_offset: u32,
    pub op: Op,
}
impl Operation {
    pub fn parse_line(line: String) -> Operation {
        let values: Vec<String> = line.split(' ').map(|s| s.to_string()).collect();
        let address_str = values.get(0).unwrap();
        let virtual_address = match u32::from_str_radix(address_str, 16) {
            Ok(value) => value,
            Err(err) => panic!("failed to parse \"{}\": {}", address_str, err),
        };
        let virtual_page_number = virtual_address >> 12; // only use the first 20 bits
        return Operation {
            virtual_address,
            virtual_page_number,
            // get lower 12 bits
            // https://www.oreilly.com/library/view/c-cookbook/0596003390/ch01s06.html
            page_offset: (virtual_address & 0x00000FFF),
            op: match values.get(1).unwrap().as_str() {
                "R" => Op::R,
                "W" => Op::W,
                value => panic!("unknown value: {}", value),
            },
        };
    }
}

#[derive(Clone, Copy)]
pub struct SimState {
    pub total_events: u32,
    pub read_count: u32,
    pub write_count: u32,
}

pub struct Sim {
    pub algorithm: String,
    pub debug: bool,
    pub state: SimState,
    pub trace: io::Lines<io::BufReader<fs::File>>,
    pub memory: Memory,
    pub page_table: PageTable,
}
impl Sim {
    pub fn new(n_pages: u32, algorithm: String, trace_file: String, debug: bool) -> Self {
        let mut memory: Memory = Memory::new();
        // 1048575 possible pages in page table in 20 bits
        // memory is full of None by default
        memory.resize_with(n_pages as usize, || None);

        return Self {
            algorithm,
            debug,
            trace: read_file(trace_file),
            memory,
            page_table: PageTable::new(),
            state: SimState {
                total_events: 0,
                read_count: 0,
                write_count: 0,
            },
        };
    }
}
impl Iterator for Sim {
    type Item = SimState;

    fn next(&mut self) -> Option<Self::Item> {
        let mut state = self.state.clone();

        if let Some(line) = self.trace.next() as Option<io::Result<String>> {
            if line.is_err() {
                panic!("could not read line {:?}", line);
            }

            state.total_events += 1;

            let op = Operation::parse_line(line.unwrap());

            if self.debug {
                println!(
                    r#"Perform "{}" operation
          virtual address     {:#034b}
          virtual page number {:#022b}
          page offset                             {:#014b}
        "#,
                    op.op, op.virtual_address, op.virtual_page_number, op.page_offset
                );
            }

            // create page table entry if it does not exist
            if !self.page_table.contains_key(&op.virtual_page_number) {
                self.page_table
                    .insert(op.virtual_page_number, PageTableEntry::new());
            }

            // load from disk if not loaded
            if !self.memory.contains(&Some(op.virtual_page_number)) {
                // pick page to be evicted
                let available_physical_page_index =
                    evict(&self.algorithm, &self.memory, &self.page_table);

                let available_physical_page_number: &Option<u32> =
                    self.memory.get(available_physical_page_index).unwrap();

                // save previous page
                if let Some(physical_page_number) = available_physical_page_number {
                    if let Some(evicted_page) = self.page_table.get_mut(physical_page_number) {
                        if self.debug {
                            println!(
                                "  evict page {:?} to load page {}\n",
                                available_physical_page_number, op.virtual_page_number
                            );
                        }

                        if evicted_page.is_dirty {
                            // for simplicity: instead of resetting the entry, simply destroy it
                            self.page_table.remove(physical_page_number);
                            state.write_count += 1;
                        }
                    }
                }

                // load from disk
                state.read_count += 1;

                // load into memory
                self.memory[available_physical_page_index] = Some(op.virtual_page_number);
            }

            let entry = self.page_table.get_mut(&op.virtual_page_number).unwrap();
            entry.reference();

            match op.op {
                Op::W => {
                    entry.is_dirty = true;
                }
                Op::R => {}
            }

            self.state = state;
            return Some(self.state);
        }

        // iterator is done
        return None;
    }
}
