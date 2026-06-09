import 'dart:convert';
import 'package:flutter_test/flutter_test.dart';
import 'package:agent_cockpit/crypto/device_key.dart';
import 'package:agent_cockpit/crypto/signer.dart';

void main() {
  test('Generate cross-platform test vectors for Rust', () async {
    final deviceKey = await DeviceKey.generate();
    final signer = Signer(deviceKey.signingKey);

    final payload = {
      'session_id': 'cross_test',
      'content': 'hello from dart',
    };

    final signingString = Signer.buildSigningString(
      messageId: 'msg_cross_001',
      deviceId: deviceKey.deviceId,
      type: 'session.input',
      timestampMs: 1781000000000,
      nonce: base64Encode(List.generate(16, (i) => i)),
      seq: 42,
      payloadSha256: Signer.computePayloadSha256(payload),
    );

    final signature = signer.sign(signingString);

    final testVector = {
      'public_key_base64': deviceKey.publicKeyBase64,
      'signing_string': signingString,
      'signature_base64': signature,
      'expected_verify': true,
    };

    final json = const JsonEncoder.withIndent('  ').convert(testVector);

    print('=== CROSS TEST VECTOR (Rust) ===');
    print(json);
    expect(true, isTrue);
  });
}
