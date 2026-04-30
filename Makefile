#
# Aquarium game - Rust port
#
PROG    = rustyquarium

.PHONY: all install run clean test

all:
	cargo build

install:
	cargo install --path .

clean:
	cargo clean

run:
	cargo run

test:
	cargo test
