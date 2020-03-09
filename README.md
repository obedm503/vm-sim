# virual memory simulation
## Installation

### Rust
Install the rust tool chain https://www.rust-lang.org/tools/install

To confirm installation run `cargo --version`

### Build
Run `cargo build --release`

## Usage

### Standard mode

`./target/release/vm-sim <nframes> <random|lru|fifo> <quiet|debug> <trace file>`

### Minimum memory mode

Finds the minimum memory required for all algorithm and trace combinations

`./target/release/vm-sim memory`

### Data mode

Output stats for all algorithm and trace combinations

`./target/release/vm-sim data`

Output stats for specific algorithm and all traces

`./target/release/vm-sim data <random|lru|fifo>`