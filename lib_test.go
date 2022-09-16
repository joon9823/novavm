package kernel_test

import (
	"encoding/binary"
	"encoding/json"
	"fmt"
	"io/ioutil"
	"strings"
	"testing"

	vm "github.com/Kernel-Labs/kernelvm"
	"github.com/Kernel-Labs/kernelvm/api"
	"github.com/stretchr/testify/require"
)

func Test_CrateVM(t *testing.T) {
	f, err := ioutil.ReadFile("./vm/move-test/build/test1/bytecode_modules/BasicCoin.mv")
	require.NoError(t, err)

	gasMeter := api.NewMockGasMeter(100000000)
	_, err = vm.CreateVM(
		api.NewLookup(gasMeter),
		api.NewMockAPI(&api.MockBankModule{}),
		api.MockQuerier{},
		gasMeter,
		true,
		f,
	)

	require.NoError(t, err)
}

func Test_PublishModule(t *testing.T) {
	f, err := ioutil.ReadFile("./vm/move-test/build/test1/bytecode_modules/BasicCoin.mv")
	require.NoError(t, err)

	gasMeter := api.NewMockGasMeter(100000000)
	kvStore := api.NewLookup(gasMeter)
	vm, err := vm.CreateVM(
		kvStore,
		api.NewMockAPI(&api.MockBankModule{}),
		api.MockQuerier{},
		gasMeter,
		true,
		f,
	)

	_, err = vm.PublishModule(
		kvStore,
		api.NewMockAPI(&api.MockBankModule{}),
		api.MockQuerier{},
		gasMeter,
		10000,
		"0x00000000000000000000000000000001",
		f,
	)

	require.NoError(t, err)
	// TODO uncomment when usedGas properly passed
	// require.NotZero(t, usedGas)
}

func Test_ExecuteContract(t *testing.T) {
	f, err := ioutil.ReadFile("./vm/move-test/build/test1/bytecode_modules/BasicCoin.mv")
	require.NoError(t, err)
	gasMeter := api.NewMockGasMeter(100000000)
	kvStore := api.NewLookup(gasMeter)
	vm, err := vm.CreateVM(
		kvStore,
		api.NewMockAPI(&api.MockBankModule{}),
		api.MockQuerier{},
		gasMeter,
		true,
		f,
	)

	payload := EntryFunction{
		Module: ModuleId{
			Address: AccountAddress("00000000000000000000000000000001"),
			Name:    "BasicCoin",
		},
		Function: "mint",
		TyArgs:   []TypeTag{},
		Args:     []Arg{convertUint64(100)},
	}
	bz, err := json.Marshal(payload)
	require.NoError(t, err)

	_, err = vm.Execute(
		kvStore,
		api.NewMockAPI(&api.MockBankModule{}),
		api.MockQuerier{},
		gasMeter,
		10000,
		"0x00000000000000000000000000000001",
		bz,
	)

	fmt.Println(err)
	require.NoError(t, err)
	// TODO uncomment when usedGas properly passed
	// require.NotZero(t, usedGas)
}

func convertUint64(num uint64) []byte {
	bz := make([]byte, 8)
	binary.LittleEndian.PutUint64(bz, num)
	return bz
}

type AccountAddress string
type Identifier string
type TypeTag string

const (
	Bool    = TypeTag("bool")
	U8      = TypeTag("u8")
	U64     = TypeTag("u64")
	U128    = TypeTag("u128")
	Address = TypeTag("address")
	Signer  = TypeTag("signer")
	// TODO - enable serialization
	// Vector = TypeTag("vector")
	// Struct = TypeTag("struct")
)

type ModuleId struct {
	Address AccountAddress `json:"address"`
	Name    Identifier     `json:"name"`
}

type EntryFunction struct {
	Module   ModuleId   `json:"module"`
	Function Identifier `json:"function"`
	TyArgs   []TypeTag  `json:"ty_args"`
	Args     []Arg      `json:"args"`
}

type Arg []byte

func (arg *Arg) UnmarshalJSON(data []byte) error {
	*arg = Arg(data)
	return nil
}

func (arg *Arg) MarshalJSON() ([]byte, error) {
	str := ""
	for _, b := range *arg {
		str += fmt.Sprintf("%d,", b)
	}
	str = fmt.Sprintf("[%s]", strings.TrimSuffix(str, ","))
	return []byte(str), nil
}
