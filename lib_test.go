package kernel_test

import (
	"encoding/json"
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
	vm, err := vm.CreateVM(
		kvStore,
		api.NewMockAPI(&api.MockBankModule{}),
		api.MockQuerier{},
		true,
		f,
	)

	return vm, kvStore
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

	payload := types.EntryFunction{
		Module: types.ModuleId{
			Address: std,
			Name:    "BasicCoin",
		},
		Function: "mint",
		TyArgs:   []types.TypeTag{"0x1::BasicCoin::Kernel"},
		Args:     []types.Arg{types.SerializeUint64(amount)},
	}
	bz, err := json.Marshal(payload)
	require.NoError(t, err)

	_, err = vm.Execute(
		kvStore,
		api.NewMockAPI(&api.MockBankModule{}),
		api.MockQuerier{},
		10000,
		minter,
		bz,
	)

	require.NoError(t, err)
	// TODO uncomment when usedGas properly passed
	// require.NotZero(t, usedGas)
}

func Test_CrateVM(t *testing.T) {
	_, _ = initializeVM(t)
}

func Test_PublishModule(t *testing.T) {
	vm, kvStore := initializeVM(t)

	f, err := ioutil.ReadFile("./vm/move-test/build/test1/bytecode_modules/BasicCoin.mv")

	_, err = vm.PublishModule(
		kvStore,
		api.NewMockAPI(&api.MockBankModule{}),
		api.MockQuerier{},
		10000,
		types.StdAddress,
		f,
	)

	require.NoError(t, err)
	// TODO uncomment when usedGas properly passed
	// require.NotZero(t, usedGas)
}

func Test_ExecuteContract(t *testing.T) {
	vm, kvStore := initializeVM(t)

	minter, err := types.NewAccountAddress("0x2")
	require.NoError(t, err)

	mintCoin(t, vm, kvStore, minter, 100)
}

func Test_QueryContract(t *testing.T) {
	vm, kvStore := initializeVM(t)

	minter, err := types.NewAccountAddress("0x2")
	require.NoError(t, err)

	mintAmount := uint64(100)
	mintCoin(t, vm, kvStore, minter, mintAmount)

	payload := types.EntryFunction{
		Module: types.ModuleId{
			Address: types.StdAddress,
			Name:    "BasicCoin",
		},
		Function: "get",
		TyArgs:   []types.TypeTag{"0x1::BasicCoin::Kernel"},
		Args:     []types.Arg{types.Arg(minter)},
	}
	bz, err := json.Marshal(payload)
	require.NoError(t, err)

	res, _, err := vm.Query(
		kvStore,
		api.NewMockAPI(&api.MockBankModule{}),
		api.MockQuerier{},
		10000,
		types.StdAddress,
		bz,
	)

	require.NoError(t, err)

	num := types.DeserializeUint64(res)
	require.Equal(t, mintAmount, num)

	// TODO uncomment when usedGas properly passed
	// require.NotZero(t, usedGas)
}
