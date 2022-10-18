.PHONY: all build build-rust build-go test

# Builds the Rust library libwasmvm
BUILDERS_PREFIX := novavm/go-ext-builder:0001
# Contains a full Go dev environment in order to run Go tests on the built library
ALPINE_TESTER := novavm/go-ext-builder:0001-alpine

USER_ID := $(shell id -u)
USER_GROUP = $(shell id -g)

SHARED_LIB_SRC = "" # File name of the shared library as created by the Rust build system
SHARED_LIB_DST = "" # File name of the shared library that we store
ifeq ($(OS),Windows_NT)
	SHARED_LIB_SRC = novavm.dll
	SHARED_LIB_DST = novavm.dll
else
	UNAME_S := $(shell uname -s)
	ifeq ($(UNAME_S),Linux)
		SHARED_LIB_SRC = libnovaproc.so
		SHARED_LIB_DST = libnovaproc.$(shell rustc --print cfg | grep target_arch | cut  -d '"' -f 2).so
	endif
	ifeq ($(UNAME_S),Darwin)
		SHARED_LIB_SRC = libnovaproc.dylib
		SHARED_LIB_DST = libnovaproc.dylib
	endif
endif


all: test-filenames build test

test-filenames:
	echo $(SHARED_LIB_DST)
	echo $(SHARED_LIB_SRC)

test: test-rust test-go

test-go: build-test
	RUST_BACKTRACE=full go test -v -count=1 -parallel=1 ./...

test-safety: build-test
	# Use package list mode to include all subdirectores. The -count=1 turns off caching.
	GODEBUG=cgocheck=2 go test -race -v -count=1 -parallel=1 ./...

test-rust: test-compiler test-vm test-lib

test-vm: build-test
	(cd crates/vm && cargo test --features testing)

test-compiler: build-test
	(cd crates/compiler && cargo test)

test-lib: build-test
	(cd libnovaproc && cargo test)

build: build-rust build-go

build-rust: build-rust-release

build-go:
	go build ./...

build-test:
	(cd crates/move-test && make build)

fmt:
	(cd crates/compiler && cargo fmt)
	(cd crates/gas && cargo fmt)
	(cd crates/move-deps && cargo fmt)
	(cd crates/natives && cargo fmt)
	(cd crates/stdlib && cargo fmt)
	(cd crates/storage && cargo fmt)
	(cd crates/types && cargo fmt)
	(cd crates/vm && cargo fmt)
	(cd libnovaproc && cargo fmt)

update-bindings:
	# After we build libnovaproc, we have to copy the generated bindings for Go code to use.
	# We cannot use symlinks as those are not reliably resolved by `go get` (https://github.com/CosmWasm/wasmvm/pull/235).
	cp libnovaproc/bindings.h api


# Use debug build for quick testing.
# In order to use "--features backtraces" here we need a Rust nightly toolchain, which we don't have by default
build-rust-debug:
	(cd libnovaproc && cargo build)
	cp -fp libnovaproc/target/debug/$(SHARED_LIB_SRC) api/$(SHARED_LIB_DST)
	make update-bindings

# use release build to actually ship - smaller and much faster
#
# See https://github.com/CosmWasm/wasmvm/issues/222#issuecomment-880616953 for two approaches to
# enable stripping through cargo (if that is desired).
build-rust-release:
	(cd libnovaproc && cargo build --release)
	cp -fp libnovaproc/target/release/$(SHARED_LIB_SRC) api/$(SHARED_LIB_DST)
	make update-bindings
	@ #this pulls out ELF symbols, 80% size reduction!

clean:
	@-rm api/bindings.h 
	@-rm api/libnovaproc.dylib
	@-rm libnovaproc/bindings.h
	@-(cd libnovaproc && cargo clean)
	@-(cd crates/vm && cargo clean)
	@-(cd crates/compiler && cargo clean)
	@echo cleaned.

# Creates a release build in a containerized build environment of the static library for Alpine Linux (.a)
release-build-alpine:
	rm -rf libnovaproc/target/release
	# build the muslc *.a file
	docker run --rm -u $(USER_ID):$(USER_GROUP)  \
		-v $(shell pwd)/crates:/code/crates \
		-v $(shell pwd)/move-deps:/code/move-deps \
		-v $(shell pwd)/libnovaproc:/code/libnovaproc \
		-v $(shell pwd)/vm:/code/vm \
		-v $(shell pwd)/compiler:/code/compiler \
		$(BUILDERS_PREFIX)-alpine
	cp libnovaproc/artifacts/libnovaproc_muslc.a api
	cp libnovaproc/artifacts/libnovaproc_muslc.aarch64.a api
	make update-bindings
	# try running go tests using this lib with muslc
	# docker run --rm -u $(USER_ID):$(USER_GROUP) -v $(shell pwd):/mnt/testrun -w /mnt/testrun $(ALPINE_TESTER) go build -tags muslc ./...
	# Use package list mode to include all subdirectores. The -count=1 turns off caching.
	# docker run --rm -u $(USER_ID):$(USER_GROUP) -v $(shell pwd):/mnt/testrun -w /mnt/testrun $(ALPINE_TESTER) go test -tags muslc -count=1 ./...

# Creates a release build in a containerized build environment of the shared library for glibc Linux (.so)
release-build-linux:
	rm -rf libnovaproc/target/release
	docker run --rm -u $(USER_ID):$(USER_GROUP) \
		-v $(shell pwd)/crates:/code/crates \
		-v $(shell pwd)/libnovaproc:/code/libnovaproc \
		-v $(shell pwd)/vm:/code/vm \
		-v $(shell pwd)/compiler:/code/compiler \
		$(BUILDERS_PREFIX)-centos7
	cp libnovaproc/artifacts/libnovaproc.x86_64.so api
	cp libnovaproc/artifacts/libnovaproc.aarch64.so api
	make update-bindings

# Creates a release build in a containerized build environment of the shared library for macOS (.dylib)
release-build-macos:
	rm -rf libnovaproc/target/x86_64-apple-darwin/release
	rm -rf libnovaproc/target/aarch64-apple-darwin/release
	docker run --rm -u $(USER_ID):$(USER_GROUP) \
		-v $(shell pwd)/crates:/code/crates \
		-v $(shell pwd)/libnovaproc:/code/libnovaproc \
		-v $(shell pwd)/vm:/code/vm \
		-v $(shell pwd)/compiler:/code/compiler \
		$(BUILDERS_PREFIX)-cross build_macos.sh
	cp libnovaproc/artifacts/libnovaproc.dylib api
	make update-bindings

release-build:
	# Write like this because those must not run in parallel
	make release-build-alpine
	make release-build-linux
	make release-build-macos
