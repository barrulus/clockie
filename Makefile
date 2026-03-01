PREFIX ?= /usr/local
BINDIR = $(PREFIX)/bin
COMPLETIONS_BASH = $(PREFIX)/share/bash-completion/completions
COMPLETIONS_ZSH = $(PREFIX)/share/zsh/site-functions
COMPLETIONS_FISH = $(PREFIX)/share/fish/vendor_completions.d

.PHONY: build install uninstall completions

build:
	cargo build --release

install: build completions
	install -Dm755 target/release/clockie $(DESTDIR)$(BINDIR)/clockie
	install -Dm644 target/completions/clockie.bash $(DESTDIR)$(COMPLETIONS_BASH)/clockie
	install -Dm644 target/completions/_clockie $(DESTDIR)$(COMPLETIONS_ZSH)/_clockie
	install -Dm644 target/completions/clockie.fish $(DESTDIR)$(COMPLETIONS_FISH)/clockie.fish

completions: build
	mkdir -p target/completions
	target/release/clockie --completions bash > target/completions/clockie.bash
	target/release/clockie --completions zsh > target/completions/_clockie
	target/release/clockie --completions fish > target/completions/clockie.fish

uninstall:
	rm -f $(DESTDIR)$(BINDIR)/clockie
	rm -f $(DESTDIR)$(COMPLETIONS_BASH)/clockie
	rm -f $(DESTDIR)$(COMPLETIONS_ZSH)/_clockie
	rm -f $(DESTDIR)$(COMPLETIONS_FISH)/clockie.fish
