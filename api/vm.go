package api

// #include <stdlib.h>
// #include "bindings.h"
import "C"

import (
	"runtime"
	"syscall"

	"github.com/Kernel-Labs/novavm/types"
)

// Initialize call ffi(`initialize`) to initialize vm
// and publish standard libraries
// CONTRACT: should be executed at chain genesis
func Initialize(
	store KVStore,
	verbose bool,
	moduleBundle []byte,
) ([]byte, error) {
	var err error

	dbState := buildDBState(store)
	db := buildDB(&dbState)

	mb := makeView(moduleBundle)
	defer runtime.KeepAlive(mb)

	errmsg := newUnmanagedVector(nil)

	res, err := C.initialize(db, cbool(verbose), &errmsg, mb)
	if err != nil && err.(syscall.Errno) != C.ErrnoValue_Success {
		// Depending on the nature of the error, `gasUsed` will either have a meaningful value, or just 0.                                                                            │                                 struct ByteSliceView checksum,
		return nil, errorWithMessage(err, errmsg)
	}

	return copyAndDestroyUnmanagedVector(res), err
}

// PublishModule call ffi(`publish_module`) to store module
func PublishModule(
	store KVStore,
	verbose bool,
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

	res, err := C.publish_module(db, cbool(verbose), cu64(gasLimit), &errmsg, senderView, mb)
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
	verbose bool,
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

	res, err := C.execute_contract(db, _api, _querier, cbool(verbose), cu64(gasLimit), &errmsg, sid, senderView, msg)
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
	verbose bool,
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

	res, err := C.execute_script(db, _api, _querier, cbool(verbose), cu64(gasLimit), &errmsg, sid, senderView, msg)
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
	verbose bool,
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

	res, err := C.query_contract(db, _api, _querier, cbool(verbose), cu64(gasLimit), &errmsg, msg)
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

func BuildContract(buildConfig types.BuildConfig) ([]byte, error) {
	var err error

	errmsg := newUnmanagedVector(nil)

	pathBytesView := makeView([]byte(buildConfig.PackagePath))
	defer runtime.KeepAlive(pathBytesView)
	installDirBytesView := makeView([]byte(buildConfig.InstallDir))
	defer runtime.KeepAlive(installDirBytesView)

	res, err := C.build_move_package(&errmsg,
		pathBytesView,
		cbool(buildConfig.Verbose),
		cbool(buildConfig.DevMode),
		cbool(buildConfig.TestMode),
		cbool(buildConfig.GenerateDocs),
		cbool(buildConfig.GenerateABIs),
		installDirBytesView,
		cbool(buildConfig.ForceRecompilation),
		cbool(buildConfig.FetchDepsOnly),
	)
	if err != nil && err.(syscall.Errno) != C.ErrnoValue_Success {
		// Depending on the nature of the error, `gasUsed` will either have a meaningful value, or just 0.                                                                            │                                 struct ByteSliceView checksum,
		return nil, errorWithMessage(err, errmsg)
	}

	return copyAndDestroyUnmanagedVector(res), err
}

func TestContract(buildConfig types.BuildConfig, testConfig types.TestConfig) ([]byte, error) {
	var err error

	errmsg := newUnmanagedVector(nil)

	pathBytesView := makeView([]byte(buildConfig.PackagePath))
	defer runtime.KeepAlive(pathBytesView)
	installDirBytesView := makeView([]byte(buildConfig.InstallDir))
	defer runtime.KeepAlive(installDirBytesView)
	filterBytesView := makeView([]byte(testConfig.Filter))
	defer runtime.KeepAlive(filterBytesView)

	res, err := C.test_move_package(&errmsg,
		pathBytesView,
		cbool(buildConfig.Verbose),
		cbool(buildConfig.DevMode),
		cbool(buildConfig.TestMode),
		cbool(buildConfig.GenerateDocs),
		cbool(buildConfig.GenerateABIs),
		installDirBytesView,
		cbool(buildConfig.ForceRecompilation),
		cbool(buildConfig.FetchDepsOnly),
		cu64(testConfig.InstructionExecutionBound),
		filterBytesView,
		cbool(testConfig.List),
		cusize(testConfig.NumThreads),
		cbool(testConfig.ReportStatistics),
		cbool(testConfig.ReportStorageOnError),
		cbool(testConfig.IgnoreCompileWarnings),
		cbool(testConfig.CheckStacklessVM),
		cbool(testConfig.VerboseMode),
		cbool(testConfig.ComputeCoverage),
	)
	if err != nil && err.(syscall.Errno) != C.ErrnoValue_Success {
		// Depending on the nature of the error, `gasUsed` will either have a meaningful value, or just 0.                                                                            │                                 struct ByteSliceView checksum,
		return nil, errorWithMessage(err, errmsg)
	}

	return copyAndDestroyUnmanagedVector(res), err
}
