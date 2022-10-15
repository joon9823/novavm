package types

import (
	"encoding/hex"
	"encoding/json"
	"errors"
	"fmt"
	"strconv"
	"strings"
)

// Module defines contract code bytes
type Module struct {
	Code Bytes `json:"code"`
}

// NewModule return module instance
func NewModule(code []byte) Module {
	if code == nil {
		code = []byte{}
	}

	return Module{Bytes(code)}
}

// ModuleBundle bundle of Modules
type ModuleBundle struct {
	Codes []Module `json:"codes"`
}

// NewModuleBundle return module bundle
func NewModuleBundle(modules ...Module) ModuleBundle {
	if modules == nil {
		modules = []Module{}
	}

	return ModuleBundle{modules}
}

// AccountAddressLen address bytes length
const AccountAddressLen = 20

// AccountAddress account address bytes
type AccountAddress []byte

var StdAddress AccountAddress

// initialize StdAddress
func init() {
	var err error
	StdAddress, err = NewAccountAddress("0x1")
	if err != nil {
		panic(err)
	}
}

// NewAccountAddress return AccountAddress from the hex string
func NewAccountAddress(hexAddr string) (AccountAddress, error) {
	hexStr := strings.TrimPrefix(hexAddr, "0x")
	lengthDiff := AccountAddressLen*2 - len(hexStr)
	if lengthDiff > 0 {
		hexStr = strings.Repeat("0", lengthDiff) + hexStr
	} else if lengthDiff < 0 {
		return nil, errors.New("invalid length of address")
	}

	sender, err := hex.DecodeString(hexStr)
	return AccountAddress(sender), err
}

func (addr *AccountAddress) UnmarshalJSON(data []byte) error {
	str := string(data)
	str = strings.TrimPrefix(str, "\"")
	str = strings.TrimSuffix(str, "\"")
	bz, err := hex.DecodeString(str)
	*addr = bz
	return err
}

func (addr AccountAddress) MarshalJSON() ([]byte, error) {
	hexStr := hex.EncodeToString(addr)
	return []byte(fmt.Sprintf("\"%s\"", hexStr)), nil
}

func (addr AccountAddress) String() string {
	return fmt.Sprintf("0x%s", hex.EncodeToString(addr))
}

// Identifier normally represent function name of entry function
type Identifier string

// TypeTag represent type argument
type TypeTag string

// TypeTags represent array of TypeTag
type TypeTags []TypeTag

type StructTagWrapper struct {
	Struct StructTag `json:"struct"`
}

type StructTag struct {
	Address  AccountAddress `json:"address"`
	Module   string         `json:"module"`
	Name     string         `json:"name"`
	TypeArgs []TypeTag      `json:"type_args"`
}

func (tt *TypeTag) UnmarshalJSON(data []byte) error {
	var structTagWrapper StructTagWrapper
	if err := json.Unmarshal(data, &structTagWrapper); err != nil {
		*tt = TypeTag(data)
	}

	*tt = TypeTag(
		fmt.Sprintf("%s::%s::%s",
			structTagWrapper.Struct.Address.String(),
			structTagWrapper.Struct.Module,
			structTagWrapper.Struct.Name,
		))

	return nil
}

func (tt TypeTag) MarshalJSON() ([]byte, error) {
	switch tt {
	case Bool, U8, U64, U128, Address, Signer:
		return []byte(fmt.Sprintf("\"%s\"", tt)), nil
	default:
		strArr := strings.Split(string(tt), "::")
		addr, err := NewAccountAddress(strArr[0])
		if err != nil {
			return nil, err
		}

		return json.Marshal(StructTagWrapper{
			Struct: StructTag{
				Address:  addr,
				Module:   strArr[1],
				Name:     strArr[2],
				TypeArgs: []TypeTag{},
			},
		})
	}
}

const (
	Bool    = TypeTag("bool")
	U8      = TypeTag("u8")
	U64     = TypeTag("u64")
	U128    = TypeTag("u128")
	Address = TypeTag("address")
	Signer  = TypeTag("signer")
)

type ModuleId struct {
	Address AccountAddress `json:"address"`
	Name    Identifier     `json:"name"`
}

type ExecuteEntryFunctionPayload struct {
	Module   ModuleId   `json:"module"`
	Function Identifier `json:"function"`
	TyArgs   []TypeTag  `json:"ty_args"`
	Args     []Bytes    `json:"args"`
}

type ExecuteScriptPayload struct {
	Code   Bytes     `json:"code"`
	TyArgs []TypeTag `json:"ty_args"`
	Args   []Bytes   `json:"args"`
}

// Arg represent argument of function or script
type Bytes []byte

// Args represent array of Arg
type Args []Bytes

func (bytes *Bytes) UnmarshalJSON(data []byte) error {
	str := string(data)
	str = strings.TrimPrefix(str, "[")
	str = strings.TrimSuffix(str, "]")
	strArr := strings.Split(str, ",")
	*bytes = make([]byte, len(strArr))
	for i, s := range strArr {
		b, err := strconv.ParseUint(s, 10, 8)
		if err != nil {
			return err
		}

		(*bytes)[i] = uint8(b)
	}
	return nil
}

func (bytes Bytes) MarshalJSON() ([]byte, error) {
	str := ""
	for _, b := range bytes {
		str += fmt.Sprintf("%d,", b)
	}
	str = fmt.Sprintf("[%s]", strings.TrimSuffix(str, ","))
	return []byte(str), nil
}

// Coin is a string representation of the sdk.Coin type (more portable than sdk.Int)
type Coin struct {
	Denom  string `json:"denom"`  // type, eg. "ATOM"
	Amount string `json:"amount"` // string encoing of decimal value, eg. "12.3456"
}

func NewCoin(amount uint64, denom string) Coin {
	return Coin{
		Denom:  denom,
		Amount: strconv.FormatUint(amount, 10),
	}
}

// Coins handles properly serializing empty amounts
type Coins []Coin

// MarshalJSON ensures that we get [] for empty arrays
func (c Coins) MarshalJSON() ([]byte, error) {
	if len(c) == 0 {
		return []byte("[]"), nil
	}
	var d []Coin = c
	return json.Marshal(d)
}

// UnmarshalJSON ensures that we get [] for empty arrays
func (c *Coins) UnmarshalJSON(data []byte) error {
	// make sure we deserialize [] back to null
	if string(data) == "[]" || string(data) == "null" {
		return nil
	}
	var d []Coin
	if err := json.Unmarshal(data, &d); err != nil {
		return err
	}
	*c = d
	return nil
}

type OutOfGasError struct{}

var _ error = OutOfGasError{}

func (o OutOfGasError) Error() string {
	return "Out of gas"
}

type Event struct {
	Key     []byte  `json:"key"`
	SeqNum  uint64  `json:"sequence_number"`
	TypeTag TypeTag `json:"type_tag"`
	Data    []byte  `json:"event_data"`
}

type SizeDelta struct {
	Address      AccountAddress `json:"address"`
	Amount       uint64         `json:"amount"`
	IsDecreasing bool           `json:"is_decreasing"`
}

type ExecutionResult struct {
	Result     []byte      `json:"result"`
	Events     []Event     `json:"events"`
	SizeDeltas []SizeDelta `json:"size_deltas"`
	GasUsed    uint64      `json:"gas_used"`
}
