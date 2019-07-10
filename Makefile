CARGO := $(shell command -v cargo || echo "$$HOME/.cargo/bin/cargo")

all: build

build:
	$(CARGO) build --release
	
check:
	$(CARGO) test

clean:
	$(CARGO) clean

install:
	$(CARGO) install --path . --root /usr/local

uninstall:
	$(CARGO) uninstall --root /usr/local
