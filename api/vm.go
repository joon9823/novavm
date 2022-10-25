package api

// #include <stdlib.h>
// #include "bindings.h"
import "C"

import (
	"runtime"
	"syscall"
)

type VM struct {
	ptr *C.vm_t
}

// ReleaseVM call ffi(`release_vm`) to release vm instance
func ReleaseVM(vm VM) {
	C.release_vm(vm.ptr)
}

// AllocateVM call ffi(`allocate_vm`) to allocate vm instance
func AllocateVM() VM {
	return VM{
		ptr: C.allocate_vm(),
	}
}

// Initialize call ffi(`initialize`) to initialize vm
// and publish standard libraries
// CONTRACT: should be executed at chain genesis
func Initialize(
	vm VM,
	store KVStore,
	verbose bool,
	moduleBundle []byte,
) error {
	var err error

	callID := startCall()
	defer endCall(callID)

	dbState := buildDBState(store, callID)
	db := buildDB(&dbState)

	mb := makeView(moduleBundle)
	defer runtime.KeepAlive(mb)

	errmsg := newUnmanagedVector(nil)

	_, err = C.initialize(vm.ptr, db, cbool(verbose), &errmsg, mb)
	if err != nil && err.(syscall.Errno) != C.ErrnoValue_Success {
		// Depending on the nature of the error, `gasUsed` will either have a meaningful value, or just 0.                                                                            │                                 struct ByteSliceView checksum,
		return errorWithMessage(err, errmsg)
	}

	return err
}

// PublishModuleBundle call ffi(`publish_module_bundle`) to store module bundle
func PublishModuleBundle(
	vm VM,
	store KVStore,
	isVerbose bool,
	gasLimit uint64,
	sessionID []byte,
	sender []byte,
	moduleBundle []byte,
) ([]byte, error) {
	var err error

	callID := startCall()
	defer endCall(callID)

	dbState := buildDBState(store, callID)
	db := buildDB(&dbState)

	sid := makeView(sessionID)
	defer runtime.KeepAlive(sid)
	senderView := makeView([]byte(sender))
	defer runtime.KeepAlive(senderView)
	moduleBundleView := makeView(moduleBundle)
	defer runtime.KeepAlive(moduleBundleView)

	errmsg := newUnmanagedVector(nil)

	res, err := C.publish_module_bundle(vm.ptr, db, cbool(isVerbose), cu64(gasLimit), &errmsg, sid, senderView, moduleBundleView)
	if err != nil && err.(syscall.Errno) != C.ErrnoValue_Success {
		// Depending on the nature of the error, `gasUsed` will either have a meaningful value, or just 0.                                                                            │                                 struct ByteSliceView checksum,
		return nil, errorWithMessage(err, errmsg)
	}

	return copyAndDestroyUnmanagedVector(res), err
}

// ExecuteContract call ffi(`execute_contract`) to execute
// script with write_op reflection
func ExecuteContract(
	vm VM,
	store KVStore,
	api GoAPI,
	verbose bool,
	gasLimit uint64,
	sessionID []byte,
	sender []byte,
	message []byte,
) ([]byte, error) {
	var err error

	callID := startCall()
	defer endCall(callID)

	dbState := buildDBState(store, callID)
	db := buildDB(&dbState)
	_api := buildAPI(&api)

	sid := makeView(sessionID)
	defer runtime.KeepAlive(sid)
	senderView := makeView(sender)
	defer runtime.KeepAlive(senderView)
	msg := makeView(message)
	defer runtime.KeepAlive(msg)

	errmsg := newUnmanagedVector(nil)

	res, err := C.execute_contract(vm.ptr, db, _api, cbool(verbose), cu64(gasLimit), &errmsg, sid, senderView, msg)
	if err != nil && err.(syscall.Errno) != C.ErrnoValue_Success {
		return nil, errorWithMessage(err, errmsg)
	}

	return copyAndDestroyUnmanagedVector(res), err
}

// ExecuteScript call ffi(`execute_script`) to execute
// entry function with write_op reflection
func ExecuteScript(
	vm VM,
	store KVStore,
	api GoAPI,
	verbose bool,
	gasLimit uint64,
	sessionID []byte,
	sender []byte,
	message []byte,
) ([]byte, error) {
	var err error

	callID := startCall()
	defer endCall(callID)

	dbState := buildDBState(store, callID)
	db := buildDB(&dbState)
	_api := buildAPI(&api)

	sid := makeView(sessionID)
	defer runtime.KeepAlive(sid)
	senderView := makeView(sender)
	defer runtime.KeepAlive(senderView)
	msg := makeView(message)
	defer runtime.KeepAlive(msg)

	errmsg := newUnmanagedVector(nil)

	res, err := C.execute_script(vm.ptr, db, _api, cbool(verbose), cu64(gasLimit), &errmsg, sid, senderView, msg)
	if err != nil && err.(syscall.Errno) != C.ErrnoValue_Success {
		return nil, errorWithMessage(err, errmsg)
	}

	return copyAndDestroyUnmanagedVector(res), err
}

// QueryContract call ffi(`query_contract`) to get
// entry function execution result without write_op reflection
func QueryContract(
	vm VM,
	store KVStore,
	api GoAPI,
	verbose bool,
	gasLimit uint64,
	message []byte,
) ([]byte, error) {
	var err error

	callID := startCall()
	defer endCall(callID)

	dbState := buildDBState(store, callID)
	db := buildDB(&dbState)
	_api := buildAPI(&api)

	msg := makeView(message)
	defer runtime.KeepAlive(msg)

	errmsg := newUnmanagedVector(nil)

	res, err := C.query_contract(vm.ptr, db, _api, cbool(verbose), cu64(gasLimit), &errmsg, msg)
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

	callID := startCall()
	defer endCall(callID)

	dbState := buildDBState(store, callID)
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
