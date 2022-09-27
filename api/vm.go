package api

// #include <stdlib.h>
// #include "bindings.h"
import "C"

import (
	"runtime"
	"syscall"
)

// Initialize call ffi(`initialize`) to initialize vm
// and publish standard libraries
// CONTRACT: should be executed at chain genesis
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

// PublishModule call ffi(`publish_module`) to store module
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

// ExecuteContract call ffi(`execute_contract`) to execute
// script with write_op reflection
func ExecuteContract(
	store KVStore,
	api GoAPI,
	querier Querier,
	isVerbose bool,
	gasLimit uint64,
	sessionID []byte,
	sender []byte,
	message []byte,
) ([]byte, error) {
	var err error

	dbState := buildDBState(store)
	db := buildDB(&dbState)
	_api := buildAPI(&api)
	_querier := buildQuerier(&querier)

	sid := makeView(sessionID)
	defer runtime.KeepAlive(sid)
	senderView := makeView(sender)
	defer runtime.KeepAlive(senderView)
	msg := makeView(message)
	defer runtime.KeepAlive(msg)

	errmsg := newUnmanagedVector(nil)

	res, err := C.execute_contract(db, _api, _querier, cbool(isVerbose), cu64(gasLimit), &errmsg, sid, senderView, msg)
	if err != nil && err.(syscall.Errno) != C.ErrnoValue_Success {
		return nil, errorWithMessage(err, errmsg)
	}

	return copyAndDestroyUnmanagedVector(res), err
}

// ExecuteScript call ffi(`execute_script`) to execute
// entry function with write_op reflection
func ExecuteScript(
	store KVStore,
	api GoAPI,
	querier Querier,
	isVerbose bool,
	gasLimit uint64,
	sessionID []byte,
	sender []byte,
	message []byte,
) ([]byte, error) {
	var err error

	dbState := buildDBState(store)
	db := buildDB(&dbState)
	_api := buildAPI(&api)
	_querier := buildQuerier(&querier)

	sid := makeView(sessionID)
	defer runtime.KeepAlive(sid)
	senderView := makeView(sender)
	defer runtime.KeepAlive(senderView)
	msg := makeView(message)
	defer runtime.KeepAlive(msg)

	errmsg := newUnmanagedVector(nil)

	res, err := C.execute_script(db, _api, _querier, cbool(isVerbose), cu64(gasLimit), &errmsg, sid, senderView, msg)
	if err != nil && err.(syscall.Errno) != C.ErrnoValue_Success {
		return nil, errorWithMessage(err, errmsg)
	}

	return copyAndDestroyUnmanagedVector(res), err
}

// QueryContract call ffi(`query_contract`) to get
// entry function execution result without write_op reflection
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

// DecodeMoveResource decode resource bytes to move resource
// instance and return as jSON string
func DecodeMoveResource(
	store KVStore,
	structTag string,
	resourceBytes []byte,
) ([]byte, error) {
	var err error

	dbState := buildDBState(store)
	db := buildDB(&dbState)

	structTagView := makeView([]byte(structTag))
	defer runtime.KeepAlive(structTagView)

	resourceBytesView := makeView(resourceBytes)
	defer runtime.KeepAlive(resourceBytesView)

	errmsg := newUnmanagedVector(nil)

	res, err := C.decode_move_resource(db, &errmsg, structTagView, resourceBytesView)
	if err != nil && err.(syscall.Errno) != C.ErrnoValue_Success {
		// Depending on the nature of the error, `gasUsed` will either have a meaningful value, or just 0.                                                                            │                                 struct ByteSliceView checksum,
		return nil, errorWithMessage(err, errmsg)
	}

	return copyAndDestroyUnmanagedVector(res), err
}

// DecodeModuleBytes decode module bytes to MoveModule
// instance and return as jSON string
func DecodeModuleBytes(
	moduleBytes []byte,
) ([]byte, error) {
	var err error

	moduleBytesView := makeView([]byte(moduleBytes))
	defer runtime.KeepAlive(moduleBytesView)

	errmsg := newUnmanagedVector(nil)

	res, err := C.decode_module_bytes(&errmsg, moduleBytesView)
	if err != nil && err.(syscall.Errno) != C.ErrnoValue_Success {
		// Depending on the nature of the error, `gasUsed` will either have a meaningful value, or just 0.                                                                            │                                 struct ByteSliceView checksum,
		return nil, errorWithMessage(err, errmsg)
	}

	return copyAndDestroyUnmanagedVector(res), err
}

// DecodeScriptBytes decode script bytes to MoveFunction
// instance and return as jSON string
func DecodeScriptBytes(
	scriptBytes []byte,
) ([]byte, error) {
	var err error

	scriptBytesView := makeView([]byte(scriptBytes))
	defer runtime.KeepAlive(scriptBytesView)

	errmsg := newUnmanagedVector(nil)

	res, err := C.decode_script_bytes(&errmsg, scriptBytesView)
	if err != nil && err.(syscall.Errno) != C.ErrnoValue_Success {
		// Depending on the nature of the error, `gasUsed` will either have a meaningful value, or just 0.                                                                            │                                 struct ByteSliceView checksum,
		return nil, errorWithMessage(err, errmsg)
	}

	return copyAndDestroyUnmanagedVector(res), err
}

func CompileContract(
	pathBytes []byte,
	isVerbose bool,
) ([]byte, error) {
	var err error

	errmsg := newUnmanagedVector(nil)

	pathBytesView := makeView([]byte(pathBytes))
	defer runtime.KeepAlive(pathBytesView)

	res, err := C.compile_move_package(&errmsg, pathBytesView, cbool(isVerbose))
	if err != nil && err.(syscall.Errno) != C.ErrnoValue_Success {
		// Depending on the nature of the error, `gasUsed` will either have a meaningful value, or just 0.                                                                            │                                 struct ByteSliceView checksum,
		return nil, errorWithMessage(err, errmsg)
	}

	return copyAndDestroyUnmanagedVector(res), err
}

func TestContract(
	pathBytes []byte,
	isVerbose bool,
) ([]byte, error) {
	var err error

	errmsg := newUnmanagedVector(nil)

	pathBytesView := makeView([]byte(pathBytes))
	defer runtime.KeepAlive(pathBytesView)

	res, err := C.test_move_package(&errmsg, pathBytesView, cbool(isVerbose))
	if err != nil && err.(syscall.Errno) != C.ErrnoValue_Success {
		// Depending on the nature of the error, `gasUsed` will either have a meaningful value, or just 0.                                                                            │                                 struct ByteSliceView checksum,
		return nil, errorWithMessage(err, errmsg)
	}

	return copyAndDestroyUnmanagedVector(res), err
}
