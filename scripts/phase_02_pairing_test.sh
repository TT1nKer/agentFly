#!/bin/bash

echo "=========================================="
echo "Phase 2: Pairing Flow Test"
echo "=========================================="

cd "$(dirname "$0")/.."

PASS=0
FAIL=0

check() {
    local name="$1"
    echo -n "  $name... "
    shift
    if "$@" > /dev/null 2>&1; then
        echo "PASS"
        PASS=$((PASS + 1))
    else
        echo "FAIL"
        FAIL=$((FAIL + 1))
    fi
}

echo ""
echo "--- Pairing Code Tests ---"
check "pair code generated" cargo test -p agent-bridge --lib pairing::tests::test_pairing_code_generated -- --nocapture
check "phone public key registered" cargo test -p agent-bridge --lib pairing::tests::test_phone_public_key_registered -- --nocapture
check "pair code single-use" cargo test -p agent-bridge --lib pairing::tests::test_pairing_code_single_use -- --nocapture
check "expired pair code rejected" cargo test -p agent-bridge --lib pairing::tests::test_expired_pairing_code_rejected -- --nocapture
check "wrong pair code rejected" cargo test -p agent-bridge --lib pairing::tests::test_wrong_pairing_code_rejected -- --nocapture

echo ""
echo "--- Device Revoke Tests ---"
check "device revoke" cargo test -p agent-bridge --lib devices::tests::test_revoke_device -- --nocapture

echo ""
echo "=========================================="
if [ $FAIL -eq 0 ]; then
    echo "pair code generated: PASS"
    echo "phone public key registered: PASS"
    echo "pair code single-use: PASS"
    echo "expired pair code rejected: PASS"
    echo "wrong pair code rejected: PASS"
    echo "PHASE 2 PASS"
    echo "=========================================="
    exit 0
else
    echo "PHASE 2 FAILED ($FAIL tests failed)"
    echo "=========================================="
    exit 1
fi
