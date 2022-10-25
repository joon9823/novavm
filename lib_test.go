package nova_test

import (
	"bytes"
	"encoding/base64"
	"os"
	"runtime"
	"testing"
	"time"

	vm "github.com/Kernel-Labs/novavm"
	"github.com/Kernel-Labs/novavm/api"
	"github.com/Kernel-Labs/novavm/types"
	"github.com/stretchr/testify/require"
)

func initializeVM(t *testing.T) (vm.VM, *api.Lookup) {
	f, err := os.ReadFile("./crates/move-test/build/test1/bytecode_modules/BasicCoin.mv")
	require.NoError(t, err)

	kvStore := api.NewLookup()
	vm := vm.NewVM(true)
	err = vm.Initialize(
		kvStore,
		types.NewModuleBundle(types.NewModule(f)),
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

	f0, err := os.ReadFile("./crates/move-test/build/test1/bytecode_modules/TestCoin.mv")
	require.NoError(t, err)
	f1, err := os.ReadFile("./crates/move-test/build/test1/bytecode_modules/Bundle1.mv")
	require.NoError(t, err)
	f2, err := os.ReadFile("./crates/move-test/build/test1/bytecode_modules/Bundle2.mv")
	require.NoError(t, err)
	f3, err := os.ReadFile("./crates/move-test/build/test1/bytecode_modules/Bundle3.mv")
	require.NoError(t, err)

	usedGas, _, sizeDeltas, err := vm.PublishModuleBundle(
		kvStore,
		100000000,
		bytes.Repeat([]byte{0}, 32),
		*testAccount,
		types.NewModuleBundle(
			types.NewModule(f0),
			types.NewModule(f1),
			types.NewModule(f2),
			types.NewModule(f3),
		),
	)
	require.NoError(t, err)
	require.NotZero(t, usedGas)

	require.NoError(t, err)
	require.Len(t, sizeDeltas, 1)
	sizeDelta := sizeDeltas[0]
	require.Equal(t, *testAccount, sizeDelta.Address)
	require.NotZero(t, sizeDelta.Amount)
}

func mintCoin(
	t *testing.T,
	vm vm.VM,
	kvStore *api.Lookup,
	minter types.AccountAddress,
	amount uint64,
) {
	testAccount, err := types.NewAccountAddress("0x2")
	defer runtime.KeepAlive(testAccount)
	require.NoError(t, err)

	tyArg := types.TypeTag__Struct{Value: types.StructTag{Address: *testAccount, Module: "TestCoin", Name: "Nova"}}
	arg, _ := types.SerializeUint64(amount)

	payload := types.EntryFunction{
		Module: types.ModuleId{
			Address: *testAccount,
			Name:    "TestCoin",
		},
		Function: "mint",
		TyArgs:   []types.TypeTag{&tyArg},
		Args:     [][]byte{arg},
	}

	mockAPI := api.NewMockBlockInfo(100, uint64(time.Now().Unix()))
	usedGas, events, sizeDeltas, err := vm.ExecuteEntryFunction(
		kvStore,
		api.NewMockAPI(&mockAPI),
		100000000,
		bytes.Repeat([]byte{0}, 32),
		minter,
		payload,
	)
	require.NoError(t, err)
	require.Len(t, events, 1)
	require.Len(t, sizeDeltas, 1)
	sizeDelta := sizeDeltas[0]
	require.Equal(t, minter, sizeDelta.Address)
	require.NotZero(t, sizeDelta.Amount)

	num, err := types.DeserializeUint64(events[0].EventData)
	require.NoError(t, err)
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

	mintCoin(t, vm, kvStore, *minter, 100)
}

func Test_FailOnExecute(t *testing.T) {
	vm, kvStore := initializeVM(t)
	defer vm.Destroy()

	publishModuleBundle(t, vm, kvStore)

	amount := uint64(100)

	testAccount, err := types.NewAccountAddress("0x2")
	require.NoError(t, err)

	mintCoin(t, vm, kvStore, *testAccount, amount)

	tyArg := types.TypeTag__Struct{Value: types.StructTag{Address: *testAccount, Module: "TestCoin", Name: "Nova"}}
	arg, _ := types.SerializeUint64(amount)
	payload := types.EntryFunction{
		Module: types.ModuleId{
			Address: *testAccount,
			Name:    "TestCoin",
		},
		Function: "mint2",
		TyArgs:   []types.TypeTag{&tyArg},
		Args:     [][]byte{arg},
	}

	mockAPI := api.NewMockBlockInfo(100, uint64(time.Now().Unix()))
	_, _, _, err = vm.ExecuteEntryFunction(
		kvStore,
		mockAPI,
		100000000,
		bytes.Repeat([]byte{0}, 32),
		*testAccount,
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

	tyArg := types.TypeTag__Struct{Value: types.StructTag{Address: *testAccount, Module: "TestCoin", Name: "Nova"}}
	arg, _ := types.SerializeUint64(amount)
	payload := types.EntryFunction{
		Module: types.ModuleId{
			Address: *testAccount,
			Name:    "BasicCoin",
		},
		Function: "mint2",
		TyArgs:   []types.TypeTag{&tyArg},
		Args:     [][]byte{arg},
	}

	mockAPI := api.NewMockBlockInfo(100, uint64(time.Now().Unix()))
	_, _, _, err = vm.ExecuteEntryFunction(
		kvStore,
		mockAPI,
		1,
		bytes.Repeat([]byte{0}, 32),
		*testAccount,
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
	mintCoin(t, vm, kvStore, *testAccount, mintAmount)

	tyArg := types.TypeTag__Struct{Value: types.StructTag{Address: *testAccount, Module: "TestCoin", Name: "Nova"}}
	payload := types.EntryFunction{
		Module: types.ModuleId{
			Address: *testAccount,
			Name:    "TestCoin",
		},
		Function: "get",
		TyArgs:   []types.TypeTag{&tyArg},
		Args:     [][]byte{testAccount[:]},
	}

	mockAPI := api.NewMockBlockInfo(100, uint64(time.Now().Unix()))
	res, err := vm.QueryEntryFunction(
		kvStore,
		mockAPI,
		10000,
		payload,
	)

	require.NoError(t, err)

	num, err := types.DeserializeUint64(res)
	require.NoError(t, err)
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

	f, err := os.ReadFile("./crates/move-test/build/test1/bytecode_modules/TestCoin.mv")
	require.NoError(t, err)

	bz, err := vm.DecodeModuleBytes(f)
	require.NoError(t, err)
	require.Contains(t, string(bz), `"address":"0x2","name":"TestCoin"`)
}

func Test_DecodeScript(t *testing.T) {
	vm, _ := initializeVM(t)
	defer vm.Destroy()

	f, err := os.ReadFile("./crates/move-test/build/test1/bytecode_scripts/main.mv")
	require.NoError(t, err)

	bz, err := vm.DecodeScriptBytes(f)
	require.NoError(t, err)
	require.Contains(t, string(bz), `"name":"main"`)
}

func Test_ExecuteScript(t *testing.T) {
	vm, kvStore := initializeVM(t)
	defer vm.Destroy()

	publishModuleBundle(t, vm, kvStore)

	f, err := os.ReadFile("./crates/move-test/build/test1/bytecode_scripts/main.mv")
	require.NoError(t, err)

	testAccount, err := types.NewAccountAddress("0x2")
	require.NoError(t, err)

	tyArg1 := types.TypeTag__Struct{Value: types.StructTag{Address: *testAccount, Module: "TestCoin", Name: "Nova"}}
	tyArg2 := types.TypeTag__Bool{}

	v, _ := types.SerializeUint64(300)
	optionalUint64 := []byte{1}
	optionalUint64 = append(optionalUint64, v...)

	payload := types.Script{
		Code:   f,
		TyArgs: []types.TypeTag{&tyArg1, &tyArg2},
		Args:   [][]byte{optionalUint64},
	}

	mockAPI := api.NewMockBlockInfo(100, uint64(time.Now().Unix()))
	usedGas, events, _, err := vm.ExecuteScript(
		kvStore,
		mockAPI,
		100000,
		bytes.Repeat([]byte{0}, 32),
		*testAccount,
		payload,
	)

	require.NoError(t, err)
	require.Len(t, events, 1)

	num, err := types.DeserializeUint64(events[0].EventData)
	require.NoError(t, err)
	require.Equal(t, uint64(300), num)
	require.NotZero(t, usedGas)
}
