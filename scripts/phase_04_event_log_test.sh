#!/bin/bash

echo "=========================================="
echo "Phase 4: Event Log Test"
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
echo "--- Event Log Tests ---"
check "event written" cargo test -p agent-bridge --lib event_log::store::tests::test_event_written -- --nocapture
check "event seq increasing" cargo test -p agent-bridge --lib event_log::store::tests::test_event_seq_increasing -- --nocapture
check "restart keeps events" cargo test -p agent-bridge --lib event_log::store::tests::test_restart_keeps_events -- --nocapture
check "fetch after seq" cargo test -p agent-bridge --lib event_log::store::tests::test_fetch_after_seq -- --nocapture

echo ""
echo "=========================================="
if [ $FAIL -eq 0 ]; then
    echo "event written: PASS"
    echo "event seq increasing: PASS"
    echo "restart keeps events: PASS"
    echo "fetch after seq: PASS"
    echo "PHASE 4 PASS"
    echo "=========================================="
    exit 0
else
    echo "PHASE 4 FAILED ($FAIL tests failed)"
    echo "=========================================="
    exit 1
fi
