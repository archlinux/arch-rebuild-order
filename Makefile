PREFIX ?= /usr/local
BINDIR ?= $(PREFIX)/bin
MANDIR ?= $(PREFIX)/share/man
OUT_DIR ?= target

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

completions:
	OUT_DIR=$(OUT_DIR) $(CARGO) run --bin completions

.PHONY: install
install: build completions doc
	install -Dm755 target/release/$(BIN) $(DESTDIR)$(BINDIR)/$(BIN)
	install -Dm644 $(DOCDIR)/$(MANPAGE) $(DESTDIR)$(MANDIR)/man1/$(MANPAGE)
	install -Dm644 target/_$(BIN) $(DESTDIR)$(PREFIX)/share/zsh/site-functions/_$(BIN)
	install -Dm644 target/$(BIN).bash $(DESTDIR)$(PREFIX)/share/bash-completion/$(BIN)
	install -Dm644 target/$(BIN).fish $(DESTDIR)$(PREFIX)/share/fish/vendor_completions.d/$(BIN)
