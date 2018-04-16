EXE=battery

INSTALL=install
INSTALLFLAGS=-C -D
DESTDIR=
PREFIX=$(HOME)/.local

default: all

debug:
	cargo build

all:
	cargo build --release

clean:
	cargo clean

install:
	mkdir -p $(DESTDIR)$(PREFIX)/bin
	$(INSTALL) $(INSTALLFLAGS) target/release/i3_$(EXE) $(DESTDIR)$(PREFIX)/bin/$(EXE)

.PHONY: all default clean install
