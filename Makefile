.PHONY: build install uninstall release

build:
	cargo build --release

install:
	install -Dm755 target/release/bolt /usr/local/bin/bolt

uninstall:
	rm -f /usr/local/bin/bolt

release:
	@VERSION=v$$(grep '^version' Cargo.toml | sed 's/.*"\(.*\)"/\1/'); \
	echo "Releasing $$VERSION..."; \
	git tag $$VERSION; \
	git push origin $$VERSION
