#!/bin/bash
set -e

echo "=========================================="
echo "Phase 1: Crypto Local Test"
echo "=========================================="

cd "$(dirname "$0")/.."

PASS=0
FAIL=0

run_test() {
    local name="$1"
    local test_filter="$2"
    echo -n "  $name... "
    if cargo test -p agent-bridge "$test_filter" -- --nocapture 2>&1 | grep -q "test result: ok"; then
        echo "PASS"
        PASS=$((PASS + 1))
    else
        echo "FAIL"
        FAIL=$((FAIL + 1))
    fi
}

echo ""
echo "Building bridge..."
cargo build -p agent-bridge 2>&1 | tail -1

echo ""
echo "--- Crypto Module Tests ---"
run_test "valid signature" "test_valid_signature_passes"
run_test "tampered payload rejected" "test_tampered_payload_rejected"
run_test "bad signature rejected" "test_bad_signature_rejected"
run_test "replay nonce rejected" "test_replay_nonce_rejected"
run_test "bad seq rejected" "test_bad_seq_rejected"

echo ""
echo "--- Basic Crypto Tests ---"
run_test "keypair generation" "test_keypair_generation"
run_test "sign and verify" "test_sign_and_verify"
run_test "tampered rejected (crypto)" "test_tampered_payload_rejected"
run_test "bad sig rejected (crypto)" "test_bad_signature_rejected"

echo ""
echo "--- Device Trust Tests ---"
run_test "device not trusted" "test_device_not_trusted_rejected"

echo ""
echo "--- Echo Adapter Tests ---"
cargo test -p agent-bridge --lib adapters::echo::tests -- --nocapture 2>&1 | tail -3

echo ""
echo "=========================================="
if [ $FAIL -eq 0 ]; then
    echo "valid signature: PASS"
    echo "tampered payload rejected: PASS"
    echo "bad signature rejected: PASS"
    echo "replay nonce rejected: PASS"
    echo "bad seq rejected: PASS"
    echo "PHASE 1 PASS"
    echo "=========================================="
    exit 0
else
    echo "PHASE 1 FAILED ($FAIL tests failed)"
    echo "=========================================="
    exit 1
fi
