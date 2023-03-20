.PHONY: build install uninstall clean

build:
	cargo build --release

install:
	cargo install --path .

uninstall:
	cargo uninstall --path .

clean:
	rm -rf target
