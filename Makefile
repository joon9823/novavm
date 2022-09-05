.PHONY: all build build-rust build-go test

SHARED_LIB_SRC = "" # File name of the shared library as created by the Rust build system
SHARED_LIB_DST = "" # File name of the shared library that we store
ifeq ($(OS),Windows_NT)
	SHARED_LIB_SRC = kernelvm.dll
	SHARED_LIB_DST = kernelvm.dll
else
	UNAME_S := $(shell uname -s)
	ifeq ($(UNAME_S),Linux)
		SHARED_LIB_SRC = libkernelvm.so
		SHARED_LIB_DST = libkernelvm.$(shell rustc --print cfg | grep target_arch | cut  -d '"' -f 2).so
	endif
	ifeq ($(UNAME_S),Darwin)
		SHARED_LIB_SRC = libkernelvm.dylib
		SHARED_LIB_DST = libkernelvm.dylib
	endif
endif

test-filenames:
	echo $(SHARED_LIB_DST)
	echo $(SHARED_LIB_SRC)

all: test-filenames build test

test:
	# Use package list mode to include all subdirectores. The -count=1 turns off caching.
	RUST_BACKTRACE=1 go test -v -count=1 ./...

test-safety:
	# Use package list mode to include all subdirectores. The -count=1 turns off caching.
	GODEBUG=cgocheck=2 go test -race -v -count=1 ./...

build: build-rust build-go

build-rust: build-rust-release

build-go:
	go build ./...

update-bindings:
	# After we build libkernelvm, we have to copy the generated bindings for Go code to use.
	# We cannot use symlinks as those are not reliably resolved by `go get` (https://github.com/CosmWasm/kernelvm/pull/235).
	cp libkernelvm/bindings.h api


# Use debug build for quick testing.
# In order to use "--features backtraces" here we need a Rust nightly toolchain, which we don't have by default
build-rust-debug:
	(cd libkernelvm && cargo build)
	cp libkernelvm/target/debug/$(SHARED_LIB_SRC) api/$(SHARED_LIB_DST)
	make update-bindings

# use release build to actually ship - smaller and much faster
#
# See https://github.com/CosmWasm/kernelvm/issues/222#issuecomment-880616953 for two approaches to
# enable stripping through cargo (if that is desired).
build-rust-release:
	(cd libkernelvm && cargo build --release)
	cp libkernelvm/target/release/$(SHARED_LIB_SRC) api/$(SHARED_LIB_DST)
	make update-bindings
	@ #this pulls out ELF symbols, 80% size reduction!

clean:
	@-rm api/bindings.h 
	@-rm api/libkernelvm.dylib
	@-rm libkernelvm/bindings.h
	@-(cd libkernelvm && cargo clean)
	@echo cleaned.
