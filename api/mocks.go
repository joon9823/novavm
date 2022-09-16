package api

import (
	"math"

	dbm "github.com/tendermint/tm-db"

	"github.com/Kernel-Labs/kernelvm/types"
)

/*** Mock GasMeter ****/
// This code is borrowed from Cosmos-SDK store/types/gas.go

// ErrorOutOfGas defines an error thrown when an action results in out of gas.
type ErrorOutOfGas struct {
	Descriptor string
}

// ErrorGasOverflow defines an error thrown when an action results gas consumption
// unsigned integer overflow.
type ErrorGasOverflow struct {
	Descriptor string
}

type MockGasMeter interface {
	GasMeter
	ConsumeGas(amount Gas, descriptor string)
}

type mockGasMeter struct {
	limit    Gas
	consumed Gas
}

func NewMockGasMeter(limit Gas) MockGasMeter {
	return &mockGasMeter{
		limit:    limit,
		consumed: 0,
	}
}

func (g *mockGasMeter) GasConsumed() Gas {
	return g.consumed
}

func (g *mockGasMeter) Limit() Gas {
	return g.limit
}

// addUint64Overflow performs the addition operation on two uint64 integers and
// returns a boolean on whether or not the result overflows.
func addUint64Overflow(a, b uint64) (uint64, bool) {
	if math.MaxUint64-a < b {
		return 0, true
	}

	return a + b, false
}

func (g *mockGasMeter) ConsumeGas(amount Gas, descriptor string) {
	var overflow bool
	// TODO: Should we set the consumed field after overflow checking?
	g.consumed, overflow = addUint64Overflow(g.consumed, amount)
	if overflow {
		panic(ErrorGasOverflow{descriptor})
	}

	if g.consumed > g.limit {
		panic(ErrorOutOfGas{descriptor})
	}
}

/*** Mock KVStore ****/
// Much of this code is borrowed from Cosmos-SDK store/transient.go

// Note: these gas prices are all in *wasmer gas* and (sdk gas * 100)
//
// We making simple values and non-clear multiples so it is easy to see their impact in test output
const (
	GetPrice    uint64 = 99000
	SetPrice           = 187000
	RemovePrice        = 142000
)

type Lookup struct {
	db    *dbm.MemDB
	meter MockGasMeter
}

func NewLookup(meter MockGasMeter) *Lookup {
	return &Lookup{
		db:    dbm.NewMemDB(),
		meter: meter,
	}
}

func (l *Lookup) SetGasMeter(meter MockGasMeter) {
	l.meter = meter
}

func (l *Lookup) WithGasMeter(meter MockGasMeter) *Lookup {
	return &Lookup{
		db:    l.db,
		meter: meter,
	}
}

// Get wraps the underlying DB's Get method panicing on error.
func (l Lookup) Get(key []byte) []byte {
	l.meter.ConsumeGas(GetPrice, "get")
	v, err := l.db.Get(key)
	if err != nil {
		panic(err)
	}

	return v
}

// Set wraps the underlying DB's Set method panicing on error.
func (l Lookup) Set(key, value []byte) {
	l.meter.ConsumeGas(SetPrice, "set")
	if err := l.db.Set(key, value); err != nil {
		panic(err)
	}
}

// Delete wraps the underlying DB's Delete method panicing on error.
func (l Lookup) Delete(key []byte) {
	l.meter.ConsumeGas(RemovePrice, "remove")
	if err := l.db.Delete(key); err != nil {
		panic(err)
	}
}

var _ KVStore = (*Lookup)(nil)

/**** MockQuerier ****/

const DEFAULT_QUERIER_GAS_LIMIT = 1_000_000

type MockQuerier struct {
	usedGas uint64
}

var _ Querier = MockQuerier{}

func DefaultQuerier(contractAddr string, coins types.Coins) Querier {
	return MockQuerier{
		usedGas: 0,
	}
}

func (q MockQuerier) Query(request types.QueryRequest, _gasLimit uint64) ([]byte, error) {
	return nil, types.Unknown{}
}

func (q MockQuerier) GasConsumed() uint64 {
	return q.usedGas
}

/***** Mock GoAPI ****/

const CanonicalLength = 32

const (
	CostTransfer uint64 = 100
)

var _ GoAPI = MockAPI{}

type MockAPI struct {
	BankModule *MockBankModule
}

func NewMockAPI(bankModule *MockBankModule) *MockAPI {
	return &MockAPI{
		BankModule: bankModule,
	}
}

func (m MockAPI) BankTransfer(recipient []byte, amount types.Coin) (uint64, error) {
	err := m.BankModule.Transfer(string(recipient), amount)
	return CostTransfer, err
}

type MockBankModule struct {
	balance map[string]types.Coin
}

func (m MockBankModule) Transfer(recipient string, amount types.Coin) error {
	m.balance[recipient] = amount
	return nil
}
