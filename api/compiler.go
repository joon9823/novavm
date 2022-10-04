package api

// #include <stdlib.h>
// #include "bindings.h"
import "C"

import (
	"fmt"
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

func CheckCoverageContractPackage(packagePath string, summaryOpt interface{}) ([]byte, error) {
	var err error

	var optType C.CoverageOption
	var moduleName string
	summaryFunction := false
	summaryOutputCSV := false

	switch typ := summaryOpt.(type) {
	case types.CoverageSummary:
		optType = C.CoverageOption_Summary
		goOpt := summaryOpt.(types.CoverageSummary)
		summaryFunction = goOpt.Function
		summaryOutputCSV = goOpt.OutputCSV
	case types.CoverageSource:
		optType = C.CoverageOption_Source
		moduleName = string(summaryOpt.(types.CoverageSource))
	case types.CoverageBytecode:
		optType = C.CoverageOption_Bytecode
		moduleName = string(summaryOpt.(types.CoverageBytecode))
	default:
		return nil, fmt.Errorf("%+v is not accceptable", typ)
	}

	errmsg := newUnmanagedVector(nil)

	pathBytesView := makeView([]byte(packagePath))
	defer runtime.KeepAlive(pathBytesView)

	moduleNameBytesView := makeView([]byte(moduleName))
	defer runtime.KeepAlive(moduleNameBytesView)

	res, err := C.check_coverage_move_package(&errmsg,
		pathBytesView,
		cu8(optType),
		cbool(summaryFunction),
		cbool(summaryOutputCSV),
		moduleNameBytesView,
	)
	if err != nil && err.(syscall.Errno) != C.ErrnoValue_Success {
		// Depending on the nature of the error, `gasUsed` will either have a meaningful value, or just 0.                                                                            │                                 struct ByteSliceView checksum,
		return nil, errorWithMessage(err, errmsg)
	}

	return copyAndDestroyUnmanagedVector(res), err
}

func ProveContractPackage(packagePath, filter, options string, forTest bool) ([]byte, error) {
	var err error

	errmsg := newUnmanagedVector(nil)

	pathBytesView := makeView([]byte(packagePath))
	defer runtime.KeepAlive(pathBytesView)
	filterBytesView := makeView([]byte(filter))
	defer runtime.KeepAlive(filterBytesView)
	optionsBytesView := makeView([]byte(options))
	defer runtime.KeepAlive(optionsBytesView)

	res, err := C.prove_move_package(&errmsg,
		pathBytesView,
		filterBytesView,
		optionsBytesView,
		cbool(forTest),
	)
	if err != nil && err.(syscall.Errno) != C.ErrnoValue_Success {
		// Depending on the nature of the error, `gasUsed` will either have a meaningful value, or just 0.                                                                            │                                 struct ByteSliceView checksum,
		return nil, errorWithMessage(err, errmsg)
	}

	return copyAndDestroyUnmanagedVector(res), err
}

func DisassembleContractPackage(packagePath, packageName, moduleOrScriptName string, interactive bool) ([]byte, error) {
	var err error

	errmsg := newUnmanagedVector(nil)

	pathBytesView := makeView([]byte(packagePath))
	defer runtime.KeepAlive(pathBytesView)
	packageNameView := makeView([]byte(packageName))
	defer runtime.KeepAlive(packageNameView)
	MoSNameView := makeView([]byte(moduleOrScriptName))
	defer runtime.KeepAlive(MoSNameView)

	res, err := C.disassemble_move_package(&errmsg,
		pathBytesView,
		packageNameView,
		MoSNameView,
		cbool(interactive),
	)
	if err != nil && err.(syscall.Errno) != C.ErrnoValue_Success {
		// Depending on the nature of the error, `gasUsed` will either have a meaningful value, or just 0.                                                                            │                                 struct ByteSliceView checksum,
		return nil, errorWithMessage(err, errmsg)
	}

	return copyAndDestroyUnmanagedVector(res), err
}

func MoveyLogin() ([]byte, error) {
	var err error

	errmsg := newUnmanagedVector(nil)

	res, err := C.movey_login(&errmsg)
	if err != nil && err.(syscall.Errno) != C.ErrnoValue_Success {
		// Depending on the nature of the error, `gasUsed` will either have a meaningful value, or just 0.                                                                            │                                 struct ByteSliceView checksum,
		return nil, errorWithMessage(err, errmsg)
	}

	return copyAndDestroyUnmanagedVector(res), err
}

/*
func GenerateErrorMap(config types.BuildConfig, errorPrefix, outputFile string) ([]byte, error) {
	var err error

	errmsg := newUnmanagedVector(nil)

	errorPrefixView := makeView([]byte(errorPrefix))
	defer runtime.KeepAlive(errorPrefixView)
	outputFileView := makeView([]byte(outputFile))
	defer runtime.KeepAlive(outputFileView)

	res, err := C.generate_error_map(&errmsg,
		errorPrefixView,
		outputFileView,
	)
	if err != nil && err.(syscall.Errno) != C.ErrnoValue_Success {
		// Depending on the nature of the error, `gasUsed` will either have a meaningful value, or just 0.                                                                            │                                 struct ByteSliceView checksum,
		return nil, errorWithMessage(err, errmsg)
	}

	return copyAndDestroyUnmanagedVector(res), err
}
*/
