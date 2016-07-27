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
TARGET_DIR=${PWD}/target

.PHONY: install symlink clean

default: install symlink

symlink: ${BIN_DIR}
	ln -sf ${TARGET_DIR}/release/srclib-rust ${BIN_DIR}/

install:
	cargo build --release

${BIN_DIR}:
	mkdir -p ${BIN_DIR}

clean:
	rm -rf ${BIN_DIR} ${TARGET_DIR}