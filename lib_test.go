package kernel_test

import (
	"io/ioutil"
	"testing"

	vm "github.com/Kernel-Labs/kernelvm"
	"github.com/Kernel-Labs/kernelvm/api"
	"github.com/stretchr/testify/require"
)

func Test_CrateVM(t *testing.T) {
	f, err := ioutil.ReadFile("./vm/move-test/build/test1/bytecode_modules/BasicCoin.mv")
	require.NoError(t, err)

	gasMeter := api.NewMockGasMeter(100000000)
	err = vm.CreateVM(
		api.NewLookup(gasMeter),
		api.NewMockAPI(&api.MockBankModule{}),
		api.MockQuerier{},
		gasMeter,
		true,
		f,
	)

	require.NoError(t, err)
}
