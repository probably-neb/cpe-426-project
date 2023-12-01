package main

import (
	"bytes"
	"encoding/binary"
	"encoding/hex"
	"reflect"
	"testing"
)


func TestTrojan(t *testing.T) {
    const ctstr string = "c883567d548300d48584ddd707f2657f"
    var ct = make([]byte, hex.DecodedLen(len(ctstr)))
    hex.Decode(ct, []byte("c883567d548300d48584ddd707f2657f"))
    expected := bytes.Repeat([]byte{0x61}, 16)

    actual, subKeys := CrackKey(ct)
    for i := 0; i < len(subKeys); i++ {
        t.Logf("subKey %d:", subKeys[i])
    }
    for i := 0; i < len(expected); i++ {
        if actual[i] != expected[i] {
            t.Errorf("CrackKey failed, expected %v, got %v", expected, actual)
        }
    }
}

func TestTrojanActive(t *testing.T) {
    key := []byte("0123456789abcdef")
    aes, _ := NewAES(key)
    aes.trojanCount = TROJAN_COUNT - 1
    input := TROJAN_ACTIVATE_MASK[:]
    ct := aes.Encrypt(input)
    wasActive := aes.trojanCounterOutput == 0xFF
    if !wasActive {
        t.Errorf("Trojan failed, expected active")
    }
    expected := bytes.Repeat([]byte{BYTE_ONE}, 16)

    subBytes(expected)
    shiftRows(expected)
    addRoundKey(expected, aes.roundKeys[10*4:10*4+4])
    eq := reflect.DeepEqual(ct, expected)
    if !eq {
        t.Errorf("Trojan failed, expected %v, got %v", expected, ct)
    }
}

func TestCrackLastSubkey(t *testing.T) {
    key := []byte("0123456789abcdef")
    aes, _ := NewAES(key)
    aes.trojanCount = TROJAN_COUNT - 1
    input := TROJAN_ACTIVATE_MASK[:]
    ct := aes.Encrypt(input)
    expected := aes.roundKeys[10*4:10*4+4]
    actualBytes := CrackLastSubkey(ct)
    actual := make([]uint32, 4)
    for i := 0; i < 4; i++ {
        actual[i] = binary.BigEndian.Uint32(actualBytes[i*4:i*4+4])
    }
    for i := 0; i < 4; i++ {
        if actual[i] != expected[i] {
            t.Errorf("CrackLastSubkey failed, expected %v, got %v", expected, actual)
        }
    }
}
