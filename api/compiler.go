package api

// #include <stdlib.h>
// #include "bindings.h"
import "C"

import (
	"runtime"
	"syscall"

	"github.com/Kernel-Labs/novavm/types"
)

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

func GetContractInfo(packagePath string) ([]byte, error) {
	var err error

	errmsg := newUnmanagedVector(nil)

	pathBytesView := makeView([]byte(packagePath))
	defer runtime.KeepAlive(pathBytesView)

	res, err := C.get_move_package_info(&errmsg,
		pathBytesView,
	)
	if err != nil && err.(syscall.Errno) != C.ErrnoValue_Success {
		// Depending on the nature of the error, `gasUsed` will either have a meaningful value, or just 0.                                                                            │                                 struct ByteSliceView checksum,
		return nil, errorWithMessage(err, errmsg)
	}

	return copyAndDestroyUnmanagedVector(res), err
}

func CreateContractPackage(packagePath string) ([]byte, error) {
	var err error

	errmsg := newUnmanagedVector(nil)

	pathBytesView := makeView([]byte(packagePath))
	defer runtime.KeepAlive(pathBytesView)

	res, err := C.create_new_move_package(&errmsg,
		pathBytesView,
	)
	if err != nil && err.(syscall.Errno) != C.ErrnoValue_Success {
		// Depending on the nature of the error, `gasUsed` will either have a meaningful value, or just 0.                                                                            │                                 struct ByteSliceView checksum,
		return nil, errorWithMessage(err, errmsg)
	}

	return copyAndDestroyUnmanagedVector(res), err
}
