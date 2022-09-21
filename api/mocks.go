package api

import (
	dbm "github.com/tendermint/tm-db"

	"github.com/Kernel-Labs/novavm/types"
)

/*** Mock KVStore ****/

type Lookup struct {
	db *dbm.MemDB
}

func NewLookup() *Lookup {
	return &Lookup{
		db: dbm.NewMemDB(),
	}
}

// Get wraps the underlying DB's Get method panicing on error.
func (l Lookup) Get(key []byte) []byte {
	v, err := l.db.Get(key)
	if err != nil {
		panic(err)
	}

	return v
}

// Set wraps the underlying DB's Set method panicing on error.
func (l Lookup) Set(key, value []byte) {
	if err := l.db.Set(key, value); err != nil {
		panic(err)
	}
}

// Delete wraps the underlying DB's Delete method panicing on error.
func (l Lookup) Delete(key []byte) {
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

func (q MockQuerier) Query(request types.QueryRequest) ([]byte, error) {
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

func (m MockAPI) BankTransfer(recipient []byte, amount types.Coin) error {
	err := m.BankModule.Transfer(string(recipient), amount)
	return err
}

type MockBankModule struct {
	balance map[string]types.Coin
}

func (m MockBankModule) Transfer(recipient string, amount types.Coin) error {
	m.balance[recipient] = amount
	return nil
}
