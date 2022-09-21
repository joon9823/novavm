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
	if err != nil && err.(syscall.Errno) != C.ErrnoValue_Success {
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
) ([]byte, error) {
	var err error

	dbState := buildDBState(store)
	db := buildDB(&dbState)

	mb := makeView(module)
	defer runtime.KeepAlive(mb)
	senderView := makeView([]byte(sender))
	defer runtime.KeepAlive(senderView)

	errmsg := newUnmanagedVector(nil)

	res, err := C.publish_module(db, cbool(isVerbose), cu64(gasLimit), &errmsg, senderView, mb)
	if err != nil && err.(syscall.Errno) != C.ErrnoValue_Success {
		// Depending on the nature of the error, `gasUsed` will either have a meaningful value, or just 0.                                                                            │                                 struct ByteSliceView checksum,
		return nil, errorWithMessage(err, errmsg)
	}

	return copyAndDestroyUnmanagedVector(res), err
}

func ExecuteContract(
	store KVStore,
	api GoAPI,
	querier Querier,
	isVerbose bool,
	gasLimit uint64,
	sender []byte,
	message []byte,
) ([]byte, error) {
	var err error

	dbState := buildDBState(store)
	db := buildDB(&dbState)
	_api := buildAPI(&api)
	_querier := buildQuerier(&querier)

	msg := makeView(message)
	defer runtime.KeepAlive(msg)
	senderView := makeView(sender)
	defer runtime.KeepAlive(senderView)

	errmsg := newUnmanagedVector(nil)

	res, err := C.execute_contract(db, _api, _querier, cbool(isVerbose), cu64(gasLimit), &errmsg, senderView, msg)
	if err != nil && err.(syscall.Errno) != C.ErrnoValue_Success {
		return nil, errorWithMessage(err, errmsg)
	}

	return copyAndDestroyUnmanagedVector(res), err
}

func QueryContract(
	store KVStore,
	api GoAPI,
	querier Querier,
	isVerbose bool,
	gasLimit uint64,
	message []byte,
) ([]byte, error) {
	var err error

	dbState := buildDBState(store)
	db := buildDB(&dbState)
	_api := buildAPI(&api)
	_querier := buildQuerier(&querier)

	msg := makeView(message)
	defer runtime.KeepAlive(msg)

	errmsg := newUnmanagedVector(nil)

	res, err := C.query_contract(db, _api, _querier, cbool(isVerbose), cu64(gasLimit), &errmsg, msg)
	if err != nil && err.(syscall.Errno) != C.ErrnoValue_Success {
		// Depending on the nature of the error, `gasUsed` will either have a meaningful value, or just 0.                                                                            │                                 struct ByteSliceView checksum,
		return nil, errorWithMessage(err, errmsg)
	}

	return copyAndDestroyUnmanagedVector(res), err
}
