package api

import (
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
	packagePath = path.Join(workingDir, "../compiler/testdata/general")
}

func Test_BuildContract(t *testing.T) {
	nova_arg := types.NewNovaCompilerArgumentWithBuildOption(packagePath, false,
		types.WithInstallDir(packagePath),
		types.WithDevMode(),
		types.WithTestMode(),
	)
	res, err := BuildContract(nova_arg)
	require.NoError(t, err)
	require.Equal(t, string(res), "ok")
}

func Test_TestContract(t *testing.T) {
	nova_arg := types.NewNovaCompilerArgumentWithBuildOption(packagePath, false,
		types.WithInstallDir(packagePath),
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

func Test_CreateNewContract(t *testing.T) {
	tmpPath := packagePath + "-tmp"
	nova_arg := types.NewNovaCompilerArgument(tmpPath, false, types.DefaultBuildConfig())
	res, err := CreateContractPackage(nova_arg, "novum_initium")
	defer os.RemoveAll(tmpPath)
	require.NoError(t, err)
	require.Equal(t, string(res), "ok")
}

func Test_CleanContract(t *testing.T) {
	nova_arg := types.NewNovaCompilerArgument(packagePath, false, types.DefaultBuildConfig())
	res, err := CleanContractPackage(nova_arg, true)
	require.NoError(t, err)
	require.Equal(t, string(res), "ok")
}
