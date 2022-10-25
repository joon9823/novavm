package types

import (
	"encoding/hex"
	"encoding/json"
	"errors"
	"fmt"
	"strconv"
	"strings"
)

// NewModule return module instance
func NewModule(code []byte) Module {
	if code == nil {
		code = []byte{}
	}

	return Module{Code: code}
}

// NewModuleBundle return module bundle
func NewModuleBundle(modules ...Module) ModuleBundle {
	if modules == nil {
		modules = []Module{}
	}

	return ModuleBundle{Codes: modules}
}

var StdAddress *AccountAddress

// initialize StdAddress
func init() {
	var err error
	StdAddress, err = NewAccountAddress("0x1")
	if err != nil {
		panic(err)
	}
}

// NewAccountAddress return AccountAddress from the hex string
func NewAccountAddress(hexAddr string) (*AccountAddress, error) {
	hexStr := strings.TrimPrefix(hexAddr, "0x")
	lengthDiff := 40 - len(hexStr) // 40: twice of AccountAddress' length
	if lengthDiff > 0 {
		hexStr = strings.Repeat("0", lengthDiff) + hexStr
	} else if lengthDiff < 0 {
		return nil, errors.New("invalid length of address")
	}

	sender, err := hex.DecodeString(hexStr)
	if err != nil {
		return nil, errors.New("invalid hex address")
	}
	accountAddress, err := BcsDeserializeAccountAddress(sender)
	return &accountAddress, err
}

func (addr AccountAddress) String() string {
	serialized, _ := addr.BcsSerialize()
	return fmt.Sprintf("0x%s", hex.EncodeToString(serialized))
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
