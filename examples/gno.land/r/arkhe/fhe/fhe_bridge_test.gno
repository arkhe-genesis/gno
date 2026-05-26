package fhe

import "testing"

func TestFheEscrow(t *testing.T) {
	encryptedBalance := []byte("encrypted_data")
	id := CreateEscrow("circle-001", 100, encryptedBalance)

	if id != 1 {
		t.Fatalf("expected escrow ID to be 1, got %d", id)
	}

	_, circleID, status, amount := QueryEscrow(id)

	if circleID != "circle-001" {
		t.Fatalf("expected circleID to be circle-001, got %s", circleID)
	}
	if status != "active" {
		t.Fatalf("expected status to be active, got %s", status)
	}
	if amount != 100 {
		t.Fatalf("expected amount to be 100, got %d", amount)
	}

	TriggerFheCompute(id, "circuit-001", "AND(1,2)")
	_, _, status2, _ := QueryEscrow(id)

	if status2 != "computing" {
		t.Fatalf("expected status to be computing, got %s", status2)
	}

	zkProof := make([]byte, 33) // Mock proof > 32 bytes
	resultProof := []byte("result")

	CompleteFheCompute(id, resultProof, zkProof)
	_, _, status3, _ := QueryEscrow(id)

	if status3 != "completed" {
		t.Fatalf("expected status to be completed, got %s", status3)
	}

	Withdraw(id)
	_, _, status4, _ := QueryEscrow(id)

	if status4 != "withdrawn" {
		t.Fatalf("expected status to be withdrawn, got %s", status4)
	}
}
