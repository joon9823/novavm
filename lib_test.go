package kernel_test

import (
	"io/ioutil"
	"testing"

	vm "github.com/Kernel-Labs/kernelvm"
	"github.com/Kernel-Labs/kernelvm/api"
	"github.com/Kernel-Labs/kernelvm/types"
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

func publishModule(
	t *testing.T,
	vm vm.VM,
	kvStore *api.Lookup,
) {
	f, err := ioutil.ReadFile("./vm/move-test/build/test1/bytecode_modules/BasicCoin.mv")
	require.NoError(t, err)

	usedGas, err := vm.PublishModule(
		kvStore,
		10000,
		types.StdAddress,
		f,
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
	std, err := types.NewAccountAddress("0x1")
	require.NoError(t, err)

	payload := types.ExecuteEntryFunctionPayload{
		Module: types.ModuleId{
			Address: std,
			Name:    "BasicCoin",
		},
		Function: "mint",
		TyArgs:   []types.TypeTag{"0x1::BasicCoin::Kernel"},
		Args:     []types.Bytes{types.SerializeUint64(amount)},
	}

	usedGas, events, err := vm.ExecuteEntryFunction(
		kvStore,
		api.NewMockAPI(&api.MockBankModule{}),
		api.MockQuerier{},
		10000,
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
	_, _ = initializeVM(t)
}

func Test_PublishModule(t *testing.T) {
	vm, kvStore := initializeVM(t)

	publishModule(t, vm, kvStore)
}

func Test_ExecuteContract(t *testing.T) {
	vm, kvStore := initializeVM(t)
	publishModule(t, vm, kvStore)

	minter, err := types.NewAccountAddress("0x2")
	require.NoError(t, err)

	mintCoin(t, vm, kvStore, minter, 100)
}

func Test_QueryContract(t *testing.T) {
	vm, kvStore := initializeVM(t)
	publishModule(t, vm, kvStore)

	minter, err := types.NewAccountAddress("0x2")
	require.NoError(t, err)

	mintAmount := uint64(100)
	mintCoin(t, vm, kvStore, minter, mintAmount)

	payload := types.ExecuteEntryFunctionPayload{
		Module: types.ModuleId{
			Address: types.StdAddress,
			Name:    "BasicCoin",
		},
		Function: "get",
		TyArgs:   []types.TypeTag{"0x1::BasicCoin::Kernel"},
		Args:     []types.Bytes{types.Bytes(minter)},
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
