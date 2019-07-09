all: build
	
check:
	cargo test

clean:
	cargo clean

install:
	cargo install --path . --root /usr/local

uninstall:
	cargo uninstall --root /usr/local

build:
	cargo build --release
