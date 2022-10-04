package api

import (
	dbm "github.com/tendermint/tm-db"
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

/***** Mock GoAPI ****/

const CanonicalLength = 32

const (
	CostTransfer uint64 = 100
)

var _ GoAPI = MockAPI{}

type MockAPI struct {
	BlockInfo *MockBlockInfo
}

func NewMockAPI(blockInfo *MockBlockInfo) *MockAPI {
	return &MockAPI{
		BlockInfo: blockInfo,
	}
}

func (m MockAPI) GetBlockInfo() (uint64, uint64) {
	return m.BlockInfo.GetBlockInfo()
}

type MockBlockInfo struct {
	height    uint64
	timestamp uint64
}

// NewMockBlockInfo return MockBlockInfo instance
func NewMockBlockInfo(height uint64, timestamp uint64) MockBlockInfo {
	return MockBlockInfo{height, timestamp}
}

func (m MockBlockInfo) GetBlockInfo() (uint64, uint64) {
	return m.height, m.timestamp
}
