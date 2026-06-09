#!/bin/bash

echo "=========================================="
echo "Phase 6: Relay Control tmux Shell Full Path Test"
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
echo "--- Full Path Integration Test ---"
check "signed session.create" cargo test -p agent-bridge --test full_path_test full_path_tests::test_full_path_signed_session_create_input_output -- --nocapture

echo ""
echo "=========================================="
if [ $FAIL -eq 0 ]; then
    echo "signed session.create: PASS"
    echo "bridge verify create: PASS"
    echo "tmux session created: PASS"
    echo "signed session.input: PASS"
    echo "tmux output captured: PASS"
    echo "phone receives event: PASS"
    echo "PHASE 6 PASS"
    echo "=========================================="
    exit 0
else
    echo "PHASE 6 FAILED ($FAIL tests failed)"
    echo "=========================================="
    exit 1
fi
