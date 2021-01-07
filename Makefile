PREFIX ?= /usr/local
BINDIR ?= $(PREFIX)/bin
MANDIR ?= $(PREFIX)/share/man

# Tools

CARGO ?= cargo
MANDOWN ?= mandown

BIN=arch-rebuild-order
MANPAGE=arch-rebuild-order.1
DOCDIR=doc

all: build doc
doc: $(DOCDIR)/$(MANPAGE)

%.1: %.md
	$(MANDOWN) $< > $@

build:
	$(CARGO) build --release --locked


.PHONY: install
install: doc
	install -Dm755 target/release/$(BIN) $(DESTDIR)$(BINDIR)/$(BIN)
	install -Dm644 $(DOCDIR)/$(MANPAGE) $(DESTDIR)$(MANDIR)/man1/$(MANPAGE)
