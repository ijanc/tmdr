#
# Copyright (c) 2025-2026 Murilo Ijanc' <murilo@ijanc.org>
#
# Permission to use, copy, modify, and/or distribute this software for any
# purpose with or without fee is hereby granted, provided that the above
# copyright notice and this permission notice appear in all copies.
#
# THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES
# WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF
# MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR
# ANY SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES
# WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN
# ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF
# OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.
#

RUSTC ?= $(shell rustup which rustc 2>/dev/null || which rustc)
RUSTFLAGS ?= -C opt-level=2 -C strip=symbols
VERSION = 0.1.0
CURL ?= curl
PREFIX ?= /usr/local
MANDIR ?= $(PREFIX)/share/man

BUILD = build
BIN = $(BUILD)/tmdr

CRATES_IO = https://crates.io/api/v1/crates
MARKDOWN_VER = 1.0.0
UNICODE_ID_VER = 0.3.6

UNICODE_ID = vendor/unicode-id/src/lib.rs
MARKDOWN = vendor/markdown/src/lib.rs
MAIN = tmdr.rs

CLIPPY ?= $(shell rustup which clippy-driver 2>/dev/null)
RUSTFMT ?= $(shell rustup which rustfmt 2>/dev/null)

.PHONY: all clean install install ci fmt-check clippy vendor

all: $(BIN)

$(BUILD)/libunicode_id.rlib: $(UNICODE_ID)
	mkdir -p $(BUILD)
	$(RUSTC) --edition 2021 --crate-type rlib \
		--crate-name unicode_id $(RUSTFLAGS) \
		-o $@ $<

$(BUILD)/libmarkdown.rlib: $(MARKDOWN) $(BUILD)/libunicode_id.rlib
	TMPDIR=/tmp $(RUSTC) --edition 2018 --crate-type rlib \
		--crate-name markdown $(RUSTFLAGS) \
		-L $(BUILD) --extern unicode_id=$(BUILD)/libunicode_id.rlib \
		-o $@ $<

$(BIN): $(MAIN) $(BUILD)/libmarkdown.rlib
	TMDR_VERSION=$(VERSION) TMPDIR=/tmp $(RUSTC) --edition 2024 \
		--crate-type bin --crate-name tmdr $(RUSTFLAGS) \
		-L $(BUILD) --extern markdown=$(BUILD)/libmarkdown.rlib \
		-o $@ $<

clean:
	rm -rf $(BUILD)

install: $(BIN)
	install -d $(PREFIX)/bin $(MANDIR)/man1
	install -m 755 $(BIN) $(PREFIX)/bin/tmdr
	install -m 644 tmdr.1 $(MANDIR)/man1/tmdr.1

vendor: $(UNICODE_ID) $(MARKDOWN)

$(UNICODE_ID):
	mkdir -p vendor
	$(CURL) -sL $(CRATES_IO)/unicode-id/$(UNICODE_ID_VER)/download \
		| tar xz -C vendor
	mv vendor/unicode-id-$(UNICODE_ID_VER) vendor/unicode-id

$(MARKDOWN):
	mkdir -p vendor
	$(CURL) -sL $(CRATES_IO)/markdown/$(MARKDOWN_VER)/download \
		| tar xz -C vendor
	mv vendor/markdown-$(MARKDOWN_VER) vendor/markdown

fmt-check:
	$(RUSTFMT) --edition 2024 --check $(MAIN)

clippy:
	TMDR_VERSION=$(VERSION) TMPDIR=/tmp $(CLIPPY) --edition 2024 \
		--crate-type bin --crate-name tmdr \
		-L $(BUILD) --extern markdown=$(BUILD)/libmarkdown.rlib \
		-W clippy::all -o /tmp/tmdr.clippy $(MAIN)
	@rm -f /tmp/tmdr.clippy

ci: fmt-check clippy $(BIN)

