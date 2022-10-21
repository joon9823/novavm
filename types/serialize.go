package types

import "encoding/binary"

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
func SerializeBytes(bz []byte) []byte {
	var output []byte
	output = append(output, serializeSequenceLen(len(bz))...)
	output = append(output, bz...)
	return output
}

// SerializeString serialize string to BCS bytes
func SerializeString(str string) []byte {
	return SerializeBytes([]byte(str))
}

// SerializeUint64 serialize num to BCS bytes
func SerializeUint64(num uint64) []byte {
	bz := make([]byte, 8)
	binary.LittleEndian.PutUint64(bz, num)
	return bz
}

// DeserializeUint64 deserialize BCS bytes
func DeserializeUint64(bz []byte) uint64 {
	return binary.LittleEndian.Uint64(bz)
}
