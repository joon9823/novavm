package types

import (
	"github.com/novifinancial/serde-reflection/serde-generate/runtime/golang/bcs"
)

var NewSerializer = bcs.NewSerializer
var NewDeserializer = bcs.NewDeserializer

func serializeSequenceLen(length int) []byte {
	return serializeU32AsUleb128(uint32(length))
}

func serializeU32AsUleb128(value uint32) []byte {
	var output []byte
	for value >= 0x80 {
		b := uint8(value & 0x7f)
		output = append(output, b|0x80)
		value >>= 7
	}

	output = append(output, uint8(value))
	return output
}

// SerializeBytes serialize bytes to BCS bytes
func SerializeBytes(bz []byte) ([]byte, error) {
	s := NewSerializer()
	err := s.SerializeBytes(bz)
	if err != nil {
		return nil, err
	}
	return s.GetBytes(), nil
}

// SerializeString serialize string to BCS bytes
func SerializeString(str string) ([]byte, error) {
	return SerializeBytes([]byte(str))
}

// SerializeUint64 serialize num to BCS bytes
func SerializeUint64(num uint64) ([]byte, error) {
	s := NewSerializer()
	err := s.SerializeU64(num)
	if err != nil {
		return nil, err
	}
	return s.GetBytes(), nil
}

// DeserializeUint64 deserialize BCS bytes
func DeserializeUint64(bz []byte) (uint64, error) {
	d := NewDeserializer(bz)
	return d.DeserializeU64()
}
