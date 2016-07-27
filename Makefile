SGX_OS_NAME := $(shell uname -o 2>/dev/null || uname -s)

ifeq ($(OS),Windows_NT)
	SRCLIB_RUST_EXE := srclib-go.exe
	CURDIR := $(shell $(CMD) "echo %cd%")
	CURDIR := $(subst \,/,$(CURDIR))
	PWD := $(CURDIR)
else
	SRCLIB_RUST_EXE := srclib-rust
endif

BIN_DIR=${PWD}/.bin

.PHONY: install symlink clean

default: install symlink

symlink: ${BIN_DIR}
	ln -sf ${PWD}/target/release/srclib-rust ${PWD}/.bin/

install: ${PWD}/target/release/srclib-rust

${PWD}/target/release/srclib-rust:
	cargo build --release

${BIN_DIR}:
	mkdir -p ${BIN_DIR}

clean:
	rm -rf ${PWD}/.bin ${PWD}/target