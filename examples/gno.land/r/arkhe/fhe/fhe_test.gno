package fhe

import (
	"testing"
)

func TestRegisterAndVerify(t *testing.T) {
	seal := RegisterFheComputation(
		"test-id", "circuit-hash", "input-hash", "output-hash", "zkp-hash", "merkle", "sub-840",
	)
	if seal == "" {
		t.Fatal("Seal should not be empty")
	}

	if !VerifyFheComputation("test-id") {
		t.Fatal("Verification failed")
	}

	if !FheComputations["test-id"].Verified {
		t.Fatal("Expected to be verified")
	}
}
