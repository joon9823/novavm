package api

// #include <stdlib.h>
// #include "bindings.h"
import "C"

import (
	"runtime"
	"syscall"
)

func Initialize(
	store KVStore,
	isVerbose bool,
	moduleBundle []byte,
) ([]byte, error) {
	var err error

	dbState := buildDBState(store)
	db := buildDB(&dbState)

	mb := makeView(moduleBundle)
	defer runtime.KeepAlive(mb)

	errmsg := newUnmanagedVector(nil)

	res, err := C.initialize(db, cbool(isVerbose), &errmsg, mb)
	if err != nil && err.(syscall.Errno) != syscall.Errno(0) /* FIXME: originally it was C.ErrnoValue_Success*/ {
		// Depending on the nature of the error, `gasUsed` will either have a meaningful value, or just 0.                                                                            │                                 struct ByteSliceView checksum,
		return nil, errorWithMessage(err, errmsg)
	}

	return copyAndDestroyUnmanagedVector(res), err
}

func PublishModule(
	store KVStore,
	isVerbose bool,
	gasLimit uint64,
	sender []byte,
	module []byte,
) ([]byte, uint64, error) {
	var err error
	var gasUsed cu64

	dbState := buildDBState(store)
	db := buildDB(&dbState)

	mb := makeView(module)
	defer runtime.KeepAlive(mb)
	senderView := makeView([]byte(sender))
	defer runtime.KeepAlive(senderView)

	errmsg := newUnmanagedVector(nil)

	res, err := C.publish_module(db, cbool(isVerbose), cu64(gasLimit), &gasUsed, &errmsg, senderView, mb)
	if err != nil && err.(syscall.Errno) != syscall.Errno(0) /* FIXME: originally it was C.ErrnoValue_Success*/ {
		// Depending on the nature of the error, `gasUsed` will either have a meaningful value, or just 0.                                                                            │                                 struct ByteSliceView checksum,
		return nil, uint64(gasUsed), errorWithMessage(err, errmsg)
	}

	return copyAndDestroyUnmanagedVector(res), uint64(gasUsed), err
}

func ExecuteContract(
	store KVStore,
	api GoAPI,
	querier Querier,
	isVerbose bool,
	gasLimit uint64,
	sender []byte,
	message []byte,
) ([]byte, uint64, error) {
	var err error
	var gasUsed cu64

	dbState := buildDBState(store)
	db := buildDB(&dbState)
	_api := buildAPI(&api)
	_querier := buildQuerier(&querier)

	msg := makeView(message)
	defer runtime.KeepAlive(msg)
	senderView := makeView(sender)
	defer runtime.KeepAlive(senderView)

	errmsg := newUnmanagedVector(nil)

	res, err := C.execute_contract(db, _api, _querier, cbool(isVerbose), cu64(gasLimit), &gasUsed, &errmsg, senderView, msg)
	if err != nil && err.(syscall.Errno) != syscall.Errno(0) /* FIXME: originally it was C.ErrnoValue_Success*/ {
		// Depending on the nature of the error, `gasUsed` will either have a meaningful value, or just 0.                                                                            │                                 struct ByteSliceView checksum,
		return nil, uint64(gasUsed), errorWithMessage(err, errmsg)
	}

	return copyAndDestroyUnmanagedVector(res), uint64(gasUsed), err
}

func QueryContract(
	store KVStore,
	api GoAPI,
	querier Querier,
	isVerbose bool,
	gasLimit uint64,
	message []byte,
) ([]byte, uint64, error) {
	var err error
	var gasUsed cu64

	dbState := buildDBState(store)
	db := buildDB(&dbState)
	_api := buildAPI(&api)
	_querier := buildQuerier(&querier)

	msg := makeView(message)
	defer runtime.KeepAlive(msg)

	errmsg := newUnmanagedVector(nil)

	res, err := C.query_contract(db, _api, _querier, cbool(isVerbose), cu64(gasLimit), &gasUsed, &errmsg, msg)
	if err != nil && err.(syscall.Errno) != syscall.Errno(0) /* FIXME: originally it was C.ErrnoValue_Success*/ {
		// Depending on the nature of the error, `gasUsed` will either have a meaningful value, or just 0.                                                                            │                                 struct ByteSliceView checksum,
		return nil, uint64(gasUsed), errorWithMessage(err, errmsg)
	}

	return copyAndDestroyUnmanagedVector(res), uint64(gasUsed), err
}
