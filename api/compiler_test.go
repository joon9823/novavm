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
	buildConfig := types.NewBuildConfig(
		types.WithInstallDir(packagePath),
		types.WithDevMode(),
		types.WithTestMode(),
	)
	testConfig := types.NewTestConfig(
		types.WithVerboseTestConfig(),
		types.WithReportStatistics(),
		types.WithReportStorageOnError(),
	)

	res, err := TestContract(packagePath, false, buildConfig, testConfig)
	require.NoError(t, err)
	require.Equal(t, string(res), "ok")
}

func Test_GetContractInfo(t *testing.T) {
	res, err := GetContractInfo(packagePath)
	require.NoError(t, err)
	require.Equal(t, string(res), "ok")
}

func Test_CreateNewContract(t *testing.T) {
	tmpPath := packagePath + "-tmp"
	res, err := CreateContractPackage(tmpPath)
	defer os.RemoveAll(tmpPath)
	require.NoError(t, err)
	require.Equal(t, string(res), "ok")
}

//* FIXME: same as compiler_test.rs, temporaraily blocked this test: revive this after adding dotnet action into workflows
func Test_ProveContract(t *testing.T) {
	tmpPath := path.Join(workingDir, "../compiler/testdata/prove")
	res, err := ProveContractPackage(tmpPath, types.ProveOption{"", false, ""})
	require.NoError(t, err)
	require.Equal(t, string(res), "ok")
}
//*/

func Test_DisassembleContract(t *testing.T) {
	//tmpPath := "compiler/testdata/general"
	dc := types.DisassembleOption{
		Interactive: false,
		PackageName: "",
		ModuleOrScriptName: "BasicCoin",
	}
	res, err := DisassembleContractPackage(packagePath, dc)
	require.NoError(t, err)
	require.Equal(t, string(res), "ok")
}

/* Don't test movey-login. It'll overwrite previous token.
func Test_MoveyLogin(t *testing.T) {
	res, err := MoveyLogin()
	require.NoError(t, err)
	require.Equal(t, string(res), "ok")
}
*/

/* Don't test movey-upload with valid token
func Test_MoveyUpload(t *testing.T) {
	res, err := MoveyUpload(packagePath)
	require.NoError(t, err)
	require.Equal(t, string(res), "ok")
}
*/

/*
func Test_CheckContractCoverage(t *testing.T) {
	covPackagePath := path.Join(workingDir, "compiler/testdata/coverage")
	res, err := CheckContractPackageCoverage(covPackagePath, types.CoverageSummary{Function: true, OutputCSV: true})
	require.NoError(t, err)
	require.Equal(t, string(res), "ok")
}
*/

/*
func Test_GenerateErrorMap(T *testing.T) {
	covPackagePath := path.Join(workingDir, "compiler/testdata/general")
	res, err := GenerateErrorMap("", "error_map")
	require.NoError(t, err)
	require.Equal(t, string(res), "ok")

}
*/
