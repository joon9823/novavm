package api

// #include <stdlib.h>
// #include "bindings.h"
import "C"

import (
	"runtime"
	"syscall"

	"github.com/Kernel-Labs/novavm/types"
)

func BuildContract(arg types.NovaCompilerArgument) ([]byte, error) {
	var err error

	errmsg := newUnmanagedVector(nil)
	buildConfig := arg.BuildConfig

	pathBytesView := makeView([]byte(arg.PackagePath))
	defer runtime.KeepAlive(pathBytesView)
	installDirBytesView := makeView([]byte(buildConfig.InstallDir))
	defer runtime.KeepAlive(installDirBytesView)

	compArg := C.NovaCompilerArgument{
		package_path: pathBytesView,
		verbose:      cbool(arg.Verbose),
		build_config: C.NovaCompilerBuildConfig{
			dev_mode:            cbool(buildConfig.DevMode),
			test_mode:           cbool(buildConfig.TestMode),
			generate_docs:       cbool(buildConfig.GenerateDocs),
			generate_abis:       cbool(buildConfig.GenerateABIs),
			install_dir:         installDirBytesView,
			force_recompilation: cbool(buildConfig.ForceRecompilation),
			fetch_deps_only:     cbool(buildConfig.FetchDepsOnly),
		},
	}

	res, err := C.build_move_package(&errmsg, compArg)
	if err != nil && err.(syscall.Errno) != C.ErrnoValue_Success {
		// Depending on the nature of the error, `gasUsed` will either have a meaningful value, or just 0.                                                                            │                                 struct ByteSliceView checksum,
		return nil, errorWithMessage(err, errmsg)
	}

	return copyAndDestroyUnmanagedVector(res), err
}

func TestContract(arg types.NovaCompilerArgument, testConfig types.TestConfig) ([]byte, error) {
	var err error

	errmsg := newUnmanagedVector(nil)
	buildConfig := arg.BuildConfig

	pathBytesView := makeView([]byte(arg.PackagePath))
	defer runtime.KeepAlive(pathBytesView)
	installDirBytesView := makeView([]byte(arg.BuildConfig.InstallDir))
	defer runtime.KeepAlive(installDirBytesView)
	filterBytesView := makeView([]byte(testConfig.Filter))
	defer runtime.KeepAlive(filterBytesView)

	compArg := C.NovaCompilerArgument{
		package_path: pathBytesView,
		verbose:      cbool(arg.Verbose),
		build_config: C.NovaCompilerBuildConfig{
			dev_mode:            cbool(buildConfig.DevMode),
			test_mode:           cbool(buildConfig.TestMode),
			generate_docs:       cbool(buildConfig.GenerateDocs),
			generate_abis:       cbool(buildConfig.GenerateABIs),
			install_dir:         installDirBytesView,
			force_recompilation: cbool(buildConfig.ForceRecompilation),
			fetch_deps_only:     cbool(buildConfig.FetchDepsOnly),
		},
	}
	testOpt := C.NovaCompilerTestOption{
		instruction_execution_bound: cu64(testConfig.InstructionExecutionBound),
		filter:                      filterBytesView,
		list:                        cbool(testConfig.List),
		num_threads:                 cusize(testConfig.NumThreads),
		report_statistics:           cbool(testConfig.ReportStatistics),
		report_storage_on_error:     cbool(testConfig.ReportStorageOnError),
		ignore_compile_warnings:     cbool(testConfig.IgnoreCompileWarnings),
		check_stackless_vm:          cbool(testConfig.CheckStacklessVM),
		verbose_mode:                cbool(testConfig.VerboseMode),
		compute_coverage:            cbool(testConfig.ComputeCoverage),
	}

	res, err := C.test_move_package(&errmsg,
		compArg,
		testOpt,
	)
	if err != nil && err.(syscall.Errno) != C.ErrnoValue_Success {
		// Depending on the nature of the error, `gasUsed` will either have a meaningful value, or just 0.                                                                            │                                 struct ByteSliceView checksum,
		return nil, errorWithMessage(err, errmsg)
	}

	return copyAndDestroyUnmanagedVector(res), err
}

func CreateContractPackage(arg types.NovaCompilerArgument, name string) ([]byte, error) {
	var err error

	errmsg := newUnmanagedVector(nil)
	buildConfig := arg.BuildConfig

	pathBytesView := makeView([]byte(arg.PackagePath))
	defer runtime.KeepAlive(pathBytesView)
	installDirBytesView := makeView([]byte(arg.BuildConfig.InstallDir))
	defer runtime.KeepAlive(installDirBytesView)

	compArg := C.NovaCompilerArgument{
		package_path: pathBytesView,
		verbose:      cbool(arg.Verbose),
		build_config: C.NovaCompilerBuildConfig{
			dev_mode:            cbool(buildConfig.DevMode),
			test_mode:           cbool(buildConfig.TestMode),
			generate_docs:       cbool(buildConfig.GenerateDocs),
			generate_abis:       cbool(buildConfig.GenerateABIs),
			install_dir:         installDirBytesView,
			force_recompilation: cbool(buildConfig.ForceRecompilation),
			fetch_deps_only:     cbool(buildConfig.FetchDepsOnly),
		},
	}

	nameView := makeView([]byte(name))
	defer runtime.KeepAlive(nameView)

	res, err := C.create_new_move_package(&errmsg, compArg, nameView)
	if err != nil && err.(syscall.Errno) != C.ErrnoValue_Success {
		// Depending on the nature of the error, `gasUsed` will either have a meaningful value, or just 0.                                                                            │                                 struct ByteSliceView checksum,
		return nil, errorWithMessage(err, errmsg)
	}

	return copyAndDestroyUnmanagedVector(res), err
}

func CleanContractPackage(arg types.NovaCompilerArgument, cleanCache, cleanByproduct, force bool) ([]byte, error) {
	var err error

	errmsg := newUnmanagedVector(nil)
	buildConfig := arg.BuildConfig

	pathBytesView := makeView([]byte(arg.PackagePath))
	defer runtime.KeepAlive(pathBytesView)
	installDirBytesView := makeView([]byte(arg.BuildConfig.InstallDir))
	defer runtime.KeepAlive(installDirBytesView)

	compArg := C.NovaCompilerArgument{
		package_path: pathBytesView,
		verbose:      cbool(arg.Verbose),
		build_config: C.NovaCompilerBuildConfig{
			dev_mode:            cbool(buildConfig.DevMode),
			test_mode:           cbool(buildConfig.TestMode),
			generate_docs:       cbool(buildConfig.GenerateDocs),
			generate_abis:       cbool(buildConfig.GenerateABIs),
			install_dir:         installDirBytesView,
			force_recompilation: cbool(buildConfig.ForceRecompilation),
			fetch_deps_only:     cbool(buildConfig.FetchDepsOnly),
		},
	}

	res, err := C.clean_move_package(&errmsg, compArg, cbool(cleanCache), cbool(cleanByproduct), cbool(force))
	if err != nil && err.(syscall.Errno) != C.ErrnoValue_Success {
		// Depending on the nature of the error, `gasUsed` will either have a meaningful value, or just 0.                                                                            │                                 struct ByteSliceView checksum,
		return nil, errorWithMessage(err, errmsg)
	}

	return copyAndDestroyUnmanagedVector(res), err
}
