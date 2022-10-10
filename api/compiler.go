package api

// #include <stdlib.h>
// #include "bindings.h"
import "C"

import (
	"fmt"
	"runtime"
	"strings"
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

func TestContract(packagePath string, verbose bool, buildConfig types.BuildConfig, testConfig types.TestConfig) ([]byte, error) {
	var err error

	errmsg := newUnmanagedVector(nil)

	pathBytesView := makeView([]byte(packagePath))
	defer runtime.KeepAlive(pathBytesView)
	installDirBytesView := makeView([]byte(buildConfig.InstallDir))
	defer runtime.KeepAlive(installDirBytesView)
	filterBytesView := makeView([]byte(testConfig.Filter))
	defer runtime.KeepAlive(filterBytesView)

	compArg := C.NovaCompilerArgument{
		package_path: pathBytesView,
		verbose:      cbool(verbose),
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

func ProveContractPackage(packagePath string, proveOpt types.ProveOption) ([]byte, error) {
	var err error

	errmsg := newUnmanagedVector(nil)

	pathBytesView := makeView([]byte(packagePath))
	defer runtime.KeepAlive(pathBytesView)
	filterBytesView := makeView([]byte(proveOpt.TargetFilter))
	defer runtime.KeepAlive(filterBytesView)
	optionsBytesView := makeView([]byte(proveOpt.Options))
	defer runtime.KeepAlive(optionsBytesView)

	res, err := C.prove_move_package(&errmsg,
		pathBytesView,
		C.NovaCompilerProveOption{
			target_filter: filterBytesView,
			for_test:      cbool(proveOpt.ForTest),
			options:       optionsBytesView,
		},
	)
	if err != nil && err.(syscall.Errno) != C.ErrnoValue_Success {
		// Depending on the nature of the error, `gasUsed` will either have a meaningful value, or just 0.                                                                            │                                 struct ByteSliceView checksum,
		return nil, errorWithMessage(err, errmsg)
	}

	return copyAndDestroyUnmanagedVector(res), err
}

func DisassembleContractPackage(packagePath string, dsOpt types.DisassembleOption) ([]byte, error) {
	var err error

	errmsg := newUnmanagedVector(nil)

	pathBytesView := makeView([]byte(packagePath))
	defer runtime.KeepAlive(pathBytesView)
	packageNameView := makeView([]byte(dsOpt.PackageName))
	defer runtime.KeepAlive(packageNameView)
	MoSNameView := makeView([]byte(dsOpt.ModuleOrScriptName))
	defer runtime.KeepAlive(MoSNameView)

	res, err := C.disassemble_move_package(&errmsg,
		pathBytesView,
		C.NovaCompilerDisassembleOption{
			interactive:           cbool(dsOpt.Interactive),
			package_name:          packageNameView,
			module_or_script_name: MoSNameView,
		},
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

func MoveyUpload(packagePath string) ([]byte, error) {
	var err error

	errmsg := newUnmanagedVector(nil)

	pathBytesView := makeView([]byte(packagePath))
	defer runtime.KeepAlive(pathBytesView)

	res, err := C.movey_upload(&errmsg, pathBytesView)
	if err != nil && err.(syscall.Errno) != C.ErrnoValue_Success {
		// Depending on the nature of the error, `gasUsed` will either have a meaningful value, or just 0.                                                                            │                                 struct ByteSliceView checksum,
		return nil, errorWithMessage(err, errmsg)
	}

	return copyAndDestroyUnmanagedVector(res), err
}

func GenerateErrorMap(arg types.NovaCompilerArgument, errorPrefix, outputFile string) ([]byte, error) {
	var err error

	errmsg := newUnmanagedVector(nil)

	pathBytesView := makeView([]byte(arg.PackagePath))
	defer runtime.KeepAlive(pathBytesView)

	buildConfig := arg.BuildConfig
	installDirBytesView := makeView([]byte(buildConfig.InstallDir))
	defer runtime.KeepAlive(installDirBytesView)

	errorPrefixView := makeView([]byte(errorPrefix))
	defer runtime.KeepAlive(errorPrefixView)
	outputFileView := makeView([]byte(outputFile))
	defer runtime.KeepAlive(outputFileView)

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

	res, err := C.generate_error_map(&errmsg,
		compArg,
		errorPrefixView,
		outputFileView,
	)
	if err != nil && err.(syscall.Errno) != C.ErrnoValue_Success {
		// Depending on the nature of the error, `gasUsed` will either have a meaningful value, or just 0.                                                                            │                                 struct ByteSliceView checksum,
		return nil, errorWithMessage(err, errmsg)
	}

	return copyAndDestroyUnmanagedVector(res), err
}

func GenerateDocs(arg types.NovaCompilerArgument, docgenOpt types.DocgenOption) ([]byte, error) {
	var err error

	errmsg := newUnmanagedVector(nil)
	buildConfig := arg.BuildConfig

	pathBytesView := makeView([]byte(arg.PackagePath))
	defer runtime.KeepAlive(pathBytesView)
	installDirBytesView := makeView([]byte(buildConfig.InstallDir))
	defer runtime.KeepAlive(installDirBytesView)
	outputDirectory := makeView([]byte(docgenOpt.OutputDirectory))
	defer runtime.KeepAlive(outputDirectory)
	referencesFile := makeView([]byte(docgenOpt.ReferencesFile))
	defer runtime.KeepAlive(referencesFile)
	template := makeView([]byte(strings.Join(docgenOpt.Template, ",")))
	defer runtime.KeepAlive(template)

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

	res, err := C.generate_docs(&errmsg,
		compArg,
		C.NovaCompilerDocgenOption{
			section_level_start:            cusize(docgenOpt.SectionLevelStart),
			exclude_private_fun:            cbool(docgenOpt.ExcludePrivateFun),
			exclude_specs:                  cbool(docgenOpt.ExcludeSpecs),
			independent_specs:              cbool(docgenOpt.IndependentSpecs),
			exclude_impl:                   cbool(docgenOpt.ExcludeImpl),
			toc_depth:                      cusize(docgenOpt.TocDeps),
			no_collapsed_sections:          cbool(docgenOpt.NoCollapsedSections),
			output_directory:               outputDirectory,
			template_:                      template,
			references_file:                referencesFile,
			include_dep_diagrams:           cbool(docgenOpt.IncludeDepDiagrams),
			include_call_diagrams:          cbool(docgenOpt.IncludeCallDiagrams),
			compile_relative_to_output_dir: cbool(docgenOpt.CompileRelativeToOutputDir),
		},
	)
	if err != nil && err.(syscall.Errno) != C.ErrnoValue_Success {
		// Depending on the nature of the error, `gasUsed` will either have a meaningful value, or just 0.                                                                            │                                 struct ByteSliceView checksum,
		return nil, errorWithMessage(err, errmsg)
	}

	return copyAndDestroyUnmanagedVector(res), err
}
