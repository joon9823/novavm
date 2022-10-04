package nova_test

import (
	"bytes"
	"encoding/base64"
	"io/ioutil"
	"os"
	"path"
	"testing"

	vm "github.com/Kernel-Labs/novavm"
	"github.com/Kernel-Labs/novavm/api"
	"github.com/Kernel-Labs/novavm/types"
	"github.com/stretchr/testify/require"
)

func initializeVM(t *testing.T) (vm.VM, *api.Lookup) {
	f, err := ioutil.ReadFile("./vm/move-test/build/test1/bytecode_modules/BasicCoin.mv")
	require.NoError(t, err)

	kvStore := api.NewLookup()
	vm := vm.NewVM(true)
	err = vm.Initialize(
		kvStore,
		types.ModuleBundle{
			Codes: []types.Module{
				{
					Code: f,
				},
			},
		},
	)
	require.NoError(t, err)

	return vm, kvStore
}

func Test_PublishModuleBundle(t *testing.T) {
	vm, kvStore := initializeVM(t)
	defer vm.Destroy()

	publishModuleBundle(t, vm, kvStore)
}

func publishModuleBundle(
	t *testing.T,
	vm vm.VM,
	kvStore *api.Lookup,
) {
	testAccount, err := types.NewAccountAddress("0x2")
	require.NoError(t, err)

	f0, err := ioutil.ReadFile("./vm/move-test/build/test1/bytecode_modules/TestCoin.mv")
	require.NoError(t, err)
	f1, err := ioutil.ReadFile("./vm/move-test/build/test1/bytecode_modules/Bundle1.mv")
	require.NoError(t, err)
	f2, err := ioutil.ReadFile("./vm/move-test/build/test1/bytecode_modules/Bundle2.mv")
	require.NoError(t, err)
	f3, err := ioutil.ReadFile("./vm/move-test/build/test1/bytecode_modules/Bundle3.mv")
	require.NoError(t, err)

	usedGas, err := vm.PublishModuleBundle(
		kvStore,
		100000000,
		bytes.Repeat([]byte{0}, 32),
		testAccount,
		types.ModuleBundle{
			Codes: []types.Module{
				{
					Code: f0,
				},
				{
					Code: f1,
				},
				{
					Code: f3,
				},
				{
					Code: f2,
				},
			},
		},
	)
	require.NoError(t, err)
	require.NotZero(t, usedGas)
}

func mintCoin(
	t *testing.T,
	vm vm.VM,
	kvStore *api.Lookup,
	minter types.AccountAddress,
	amount uint64,
) {
	testAccount, err := types.NewAccountAddress("0x2")
	require.NoError(t, err)

	payload := types.ExecuteEntryFunctionPayload{
		Module: types.ModuleId{
			Address: testAccount,
			Name:    "TestCoin",
		},
		Function: "mint",
		TyArgs:   []types.TypeTag{"0x2::TestCoin::Nova"},
		Args:     []types.Bytes{types.SerializeUint64(amount)},
	}

	usedGas, events, err := vm.ExecuteEntryFunction(
		kvStore,
		api.NewMockAPI(&api.MockBankModule{}),
		api.MockQuerier{},
		100000000,
		bytes.Repeat([]byte{0}, 32),
		minter,
		payload,
	)
	require.NoError(t, err)
	require.Len(t, events, 1)

	num := types.DeserializeUint64(events[0].Data)
	require.Equal(t, amount, num)
	require.NotZero(t, usedGas)
}

func Test_InitializeVM(t *testing.T) {
	vm, _ := initializeVM(t)
	defer vm.Destroy()
}

func Test_ExecuteContract(t *testing.T) {
	vm, kvStore := initializeVM(t)
	defer vm.Destroy()

	publishModuleBundle(t, vm, kvStore)

	minter, err := types.NewAccountAddress("0x2")
	require.NoError(t, err)

	mintCoin(t, vm, kvStore, minter, 100)
}

func Test_FailOnExecute(t *testing.T) {
	vm, kvStore := initializeVM(t)
	defer vm.Destroy()

	publishModuleBundle(t, vm, kvStore)

	amount := uint64(100)

	testAccount, err := types.NewAccountAddress("0x2")
	require.NoError(t, err)

	mintCoin(t, vm, kvStore, testAccount, amount)

	payload := types.ExecuteEntryFunctionPayload{
		Module: types.ModuleId{
			Address: testAccount,
			Name:    "TestCoin",
		},
		Function: "mint2",
		TyArgs:   []types.TypeTag{"0x2::TestCoin::Nova"},
		Args:     []types.Bytes{types.SerializeUint64(amount)},
	}

	_, _, err = vm.ExecuteEntryFunction(
		kvStore,
		api.NewMockAPI(&api.MockBankModule{}),
		api.MockQuerier{},
		100000000,
		bytes.Repeat([]byte{0}, 32),
		testAccount,
		payload,
	)
	require.NotNil(t, err)
	require.Contains(t, err.Error(), "FUNCTION_RESOLUTION_FAILURE")
}

func Test_OutOfGas(t *testing.T) {
	vm, kvStore := initializeVM(t)
	defer vm.Destroy()

	publishModuleBundle(t, vm, kvStore)

	amount := uint64(100)

	testAccount, err := types.NewAccountAddress("0x2")
	require.NoError(t, err)

	payload := types.ExecuteEntryFunctionPayload{
		Module: types.ModuleId{
			Address: testAccount,
			Name:    "BasicCoin",
		},
		Function: "mint2",
		TyArgs:   []types.TypeTag{"0x2::TestCoin::Nova"},
		Args:     []types.Bytes{types.SerializeUint64(amount)},
	}

	_, _, err = vm.ExecuteEntryFunction(
		kvStore,
		api.NewMockAPI(&api.MockBankModule{}),
		api.MockQuerier{},
		1,
		bytes.Repeat([]byte{0}, 32),
		testAccount,
		payload,
	)
	require.NotNil(t, err)
	require.ErrorIs(t, err, types.OutOfGasError{})
}

func Test_QueryContract(t *testing.T) {
	vm, kvStore := initializeVM(t)
	defer vm.Destroy()

	publishModuleBundle(t, vm, kvStore)

	testAccount, err := types.NewAccountAddress("0x2")
	require.NoError(t, err)

	mintAmount := uint64(100)
	mintCoin(t, vm, kvStore, testAccount, mintAmount)

	payload := types.ExecuteEntryFunctionPayload{
		Module: types.ModuleId{
			Address: testAccount,
			Name:    "TestCoin",
		},
		Function: "get",
		TyArgs:   []types.TypeTag{"0x2::TestCoin::Nova"},
		Args:     []types.Bytes{types.Bytes(testAccount)},
	}

	res, err := vm.QueryEntryFunction(
		kvStore,
		api.NewMockAPI(&api.MockBankModule{}),
		api.MockQuerier{},
		10000,
		payload,
	)

	require.NoError(t, err)

	num := types.DeserializeUint64(res)
	require.Equal(t, mintAmount, num)
}

func Test_DecodeResource(t *testing.T) {
	vm, kvStore := initializeVM(t)
	defer vm.Destroy()

	publishModuleBundle(t, vm, kvStore)

	bz, err := base64.StdEncoding.DecodeString("LAEAAAAAAAAB")
	require.NoError(t, err)

	bz, err = vm.DecodeMoveResource(kvStore, "0x2::TestCoin::Coin<0x2::TestCoin::Nova>", bz)
	require.NoError(t, err)
	require.Equal(t, bz, []byte(`{"type":"0x2::TestCoin::Coin<0x2::TestCoin::Nova>","data":{"test":true,"value":"300"}}`))
}

func Test_DecodeModule(t *testing.T) {
	vm, _ := initializeVM(t)
	defer vm.Destroy()

	f, err := ioutil.ReadFile("./vm/move-test/build/test1/bytecode_modules/TestCoin.mv")
	require.NoError(t, err)

	bz, err := vm.DecodeModuleBytes(f)
	require.NoError(t, err)
	require.Contains(t, string(bz), `"address":"0x2","name":"TestCoin"`)
}

func Test_DecodeScript(t *testing.T) {
	vm, _ := initializeVM(t)
	defer vm.Destroy()

	f, err := ioutil.ReadFile("./vm/move-test/build/test1/bytecode_scripts/main.mv")
	require.NoError(t, err)

	bz, err := vm.DecodeScriptBytes(f)
	require.NoError(t, err)
	require.Contains(t, string(bz), `"name":"main"`)
}

func Test_ExecuteScript(t *testing.T) {
	vm, kvStore := initializeVM(t)
	defer vm.Destroy()

	publishModuleBundle(t, vm, kvStore)

	f, err := ioutil.ReadFile("./vm/move-test/build/test1/bytecode_scripts/main.mv")
	require.NoError(t, err)

	testAccount, err := types.NewAccountAddress("0x2")
	require.NoError(t, err)

	payload := types.ExecuteScriptPayload{
		Code:   f,
		TyArgs: []types.TypeTag{"0x2::TestCoin::Nova", "bool"},
		Args:   []types.Bytes{},
	}

	usedGas, events, err := vm.ExecuteScript(
		kvStore,
		api.NewMockAPI(&api.MockBankModule{}),
		api.MockQuerier{},
		15000,
		bytes.Repeat([]byte{0}, 32),
		testAccount,
		payload,
	)

	require.NoError(t, err)
	require.Len(t, events, 1)

	num := types.DeserializeUint64(events[0].Data)
	require.Equal(t, uint64(200), num)
	require.NotZero(t, usedGas)
}

var workingDir string
var packagePath string

func init() {
	workingDir, _ = os.Getwd()
	packagePath = path.Join(workingDir, "vm/move-test")
}

func Test_BuildContract(t *testing.T) {
	buildConfig := types.NewBuildConfig(
		types.WithPackagePath(packagePath),
		types.WithInstallDir(packagePath),
		types.WithDevMode(),
		types.WithTestMode(),
	)

	res, err := api.BuildContract(buildConfig)
	require.NoError(t, err)
	require.Equal(t, string(res), "ok")
}

func Test_TestContract(t *testing.T) {
	buildConfig := types.NewBuildConfig(
		types.WithPackagePath(packagePath),
		types.WithInstallDir(packagePath),
		types.WithVerboseBuildConfig(),
		types.WithDevMode(),
		types.WithTestMode(),
	)
	testConfig := types.NewTestConfig(
		types.WithVerboseTestConfig(),
		types.WithReportStatistics(),
		types.WithReportStorageOnError(),
	)

	res, err := api.TestContract(buildConfig, testConfig)
	require.NoError(t, err)
	require.Equal(t, string(res), "ok")
}

func Test_GetContractInfo(t *testing.T) {
	res, err := api.GetContractInfo(packagePath)
	require.NoError(t, err)
	require.Equal(t, string(res), "ok")
}

func Test_CreateNewContract(t *testing.T) {
	tmpPath := packagePath + "-tmp"
	res, err := api.CreateContractPackage(tmpPath)
	defer os.RemoveAll(tmpPath)
	require.NoError(t, err)
	require.Equal(t, string(res), "ok")
}

/* FIXME: same as compiler_test.rs, temporaraily blocked this test: revive this after adding dotnet action into workflows
func Test_ProveContract(t *testing.T) {
	tmpPath := "compiler/testdata/prove"
	res, err := api.ProveContractPackage(tmpPath, "", "", true)
	require.NoError(t, err)
	require.Equal(t, string(res), "ok")
}
*/

func Test_DisassembleContract(t *testing.T) {
	//tmpPath := "compiler/testdata/general"
	res, err := api.DisassembleContractPackage(packagePath, "", "BasicCoin", false)
	require.NoError(t, err)
	require.Equal(t, string(res), "ok")
}

/* Don't test movey-login. It'll overwrite previous token.
func Test_MoveyLogin(t *testing.T) {
	res, err := api.MoveyLogin()
	require.NoError(t, err)
	require.Equal(t, string(res), "ok")
}
*/

/*
func Test_CheckContractCoverage(t *testing.T) {
	covPackagePath := path.Join(workingDir, "compiler/testdata/coverage")
	res, err := api.CheckContractPackageCoverage(covPackagePath, types.CoverageSummary{Function: true, OutputCSV: true})
	require.NoError(t, err)
	require.Equal(t, string(res), "ok")
}
*/

/*
func Test_GenerateErrorMap(T *testing.T) {
	covPackagePath := path.Join(workingDir, "compiler/testdata/general")
	res, err := api.GenerateErrorMap("", "error_map")
	require.NoError(t, err)
	require.Equal(t, string(res), "ok")

}
*/
