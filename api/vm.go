package api

// #include <stdlib.h>
// #include "bindings.h"
import "C"

import (
	"runtime"
	"syscall"
)

func Initialize(store KVStore, api GoAPI, querier Querier, gasMeter GasMeter, isVerbose bool, module_bundle []byte) ([]byte, error) {
	var err error

	callID := startCall()
	dbState := buildDBState(store, callID)
	db := buildDB(&dbState, &gasMeter)
	_api := buildAPI(&api)
	_querier := buildQuerier(&querier)

	mb := makeView(module_bundle)
	defer runtime.KeepAlive(mb)

	errmsg := newUnmanagedVector(nil)

	res, err := C.initialize(db, _api, _querier, cbool(isVerbose), &errmsg, mb)
	if err != nil && err.(syscall.Errno) != syscall.Errno(0) /* FIXME: originally it was C.ErrnoValue_Success*/ {
		// Depending on the nature of the error, `gasUsed` will either have a meaningful value, or just 0.                                                                            │                                 struct ByteSliceView checksum,
		return nil, errorWithMessage(err, errmsg)
	}

	return copyAndDestroyUnmanagedVector(res), err
}

func PublishModule(store KVStore, api GoAPI, querier Querier, gasMeter GasMeter, isVerbose bool, gasLimit uint64, sender string, module []byte) ([]byte, uint64, error) {
	var err error
	var gasUsed cu64

	callID := startCall()
	dbState := buildDBState(store, callID)
	db := buildDB(&dbState, &gasMeter)
	_api := buildAPI(&api)
	_querier := buildQuerier(&querier)

	mb := makeView(module)
	defer runtime.KeepAlive(mb)
	senderView := makeView([]byte(sender))
	defer runtime.KeepAlive(senderView)

	errmsg := newUnmanagedVector(nil)

	res, err := C.publish_module(db, _api, _querier, cbool(isVerbose), cu64(gasLimit), &gasUsed, &errmsg, senderView, mb)
	if err != nil && err.(syscall.Errno) != syscall.Errno(0) /* FIXME: originally it was C.ErrnoValue_Success*/ {
		// Depending on the nature of the error, `gasUsed` will either have a meaningful value, or just 0.                                                                            │                                 struct ByteSliceView checksum,
		return nil, uint64(gasUsed), errorWithMessage(err, errmsg)
	}

	return copyAndDestroyUnmanagedVector(res), uint64(gasUsed), err
}

func ExecuteContract(store KVStore, api GoAPI, querier Querier, gasMeter GasMeter, isVerbose bool, gasLimit uint64, sender string, message []byte) ([]byte, uint64, error) {
	var err error
	var gasUsed cu64

	callID := startCall()
	dbState := buildDBState(store, callID)
	db := buildDB(&dbState, &gasMeter)
	_api := buildAPI(&api)
	_querier := buildQuerier(&querier)

	msg := makeView(message)
	defer runtime.KeepAlive(msg)
	senderView := makeView([]byte(sender))
	defer runtime.KeepAlive(senderView)

	errmsg := newUnmanagedVector(nil)

	res, err := C.execute_contract(db, _api, _querier, cbool(isVerbose), cu64(gasLimit), &gasUsed, &errmsg, senderView, msg)
	if err != nil && err.(syscall.Errno) != syscall.Errno(0) /* FIXME: originally it was C.ErrnoValue_Success*/ {
		// Depending on the nature of the error, `gasUsed` will either have a meaningful value, or just 0.                                                                            │                                 struct ByteSliceView checksum,
		return nil, uint64(gasUsed), errorWithMessage(err, errmsg)
	}

	return copyAndDestroyUnmanagedVector(res), uint64(gasUsed), err
}
