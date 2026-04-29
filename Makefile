#
# Aquarium game - Rust port
#
PROG    = rustyquarium
DESTDIR = $(HOME)/.local

.PHONY: all install uninstall clean test

all:
	cargo build --release

install: all
	@install -d $(DESTDIR)/usr/bin
	install -m 555 target/release/$(PROG) $(DESTDIR)/bin/$(PROG)

uninstall:
	rm -f $(DESTDIR)/usr/bin/$(PROG)

clean:
	cargo clean

test:
	cargo test
