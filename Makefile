.PHONY: build install clean

build:
	cargo build --release

install:
	cargo install --path .

clean:
	cargo clean

all: build install
