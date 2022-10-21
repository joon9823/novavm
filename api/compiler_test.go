package api

import (
	"errors"
	"os"
	"path"
	"testing"

	"github.com/Kernel-Labs/novavm/types"
	"github.com/stretchr/testify/require"
)

var workingDir string
var packagePath string

func init() {
	workingDir, _ = os.Getwd()
	packagePath = path.Join(workingDir, "../crates/move-test")
}

func Test_TestContract(t *testing.T) {
	nova_arg := types.NewNovaCompilerArgumentWithBuildOption(packagePath, false,
		types.WithInstallDir(path.Join(packagePath, "build-test")),
		types.WithDevMode(),
		types.WithTestMode(),
	)
	testConfig := types.NewTestConfig(
		types.WithVerboseTestConfig(),
		types.WithReportStatistics(),
		types.WithReportStorageOnError(),
	)

	res, err := TestContract(nova_arg, testConfig)
	require.NoError(t, err)
	require.Equal(t, string(res), "ok")
}

// NOTE: should be executed before `Test_BuildContract`
func Test_CleanContract(t *testing.T) {
	tmpPath, err := os.MkdirTemp(os.TempDir(), "nova-compiler")
	require.NoError(t, err)

	defer os.RemoveAll(tmpPath)

	// new
	nova_arg := types.NewNovaCompilerArgument(tmpPath, false, types.DefaultBuildConfig())
	res, err := CreateContractPackage(nova_arg, "novum_initium")
	require.NoError(t, err)
	require.Equal(t, string(res), "ok")

	// make dummy build folder
	buildPath := path.Join(tmpPath, "build")
	err = os.Mkdir(buildPath, os.ModePerm)
	require.NoError(t, err)

	// clean
	nova_arg = types.NewNovaCompilerArgument(tmpPath, false, types.DefaultBuildConfig())
	res, err = CleanContractPackage(nova_arg, true, true, true)
	require.NoError(t, err)
	require.Equal(t, string(res), "ok")

	_, err = os.Stat(buildPath)
	require.True(t, errors.Is(err, os.ErrNotExist))
	_, err = os.Stat(path.Join(buildPath, "doc"))
	require.True(t, errors.Is(err, os.ErrNotExist))
	_, err = os.Stat(path.Join(buildPath, "abi"))
	require.True(t, errors.Is(err, os.ErrNotExist))
	_, err = os.Stat(path.Join(buildPath, "error_map.errmap"))
	require.True(t, errors.Is(err, os.ErrNotExist))
	_, err = os.Stat(path.Join(buildPath, ".coverage_map.mvcov"))
	require.True(t, errors.Is(err, os.ErrNotExist))
	_, err = os.Stat(path.Join(buildPath, ".trace"))
	require.True(t, errors.Is(err, os.ErrNotExist))
}

// NOTE: should be executed after `Test_CleanContract`
func Test_BuildContract(t *testing.T) {
	nova_arg := types.NewNovaCompilerArgumentWithBuildOption(packagePath, false,
		types.WithInstallDir(path.Join(packagePath, "build-release")),
	)
	res, err := BuildContract(nova_arg)
	require.NoError(t, err)
	require.Equal(t, string(res), "ok")
}

func Test_CreateNewContract(t *testing.T) {
	tmpPath, err := os.MkdirTemp(os.TempDir(), "nova-compiler")
	require.NoError(t, err)

	defer os.RemoveAll(tmpPath)

	nova_arg := types.NewNovaCompilerArgument(tmpPath, false, types.DefaultBuildConfig())
	res, err := CreateContractPackage(nova_arg, "novum_initium")
	require.NoError(t, err)
	require.Equal(t, string(res), "ok")
}
