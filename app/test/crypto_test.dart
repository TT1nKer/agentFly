import 'dart:convert';
import 'package:flutter_test/flutter_test.dart';
import 'package:agent_cockpit/crypto/device_key.dart';
import 'package:agent_cockpit/crypto/signer.dart';

void main() {
  group('Crypto - DeviceKey', () {
    test('generates keypair', () async {
      final deviceKey = await DeviceKey.generate();
      expect(deviceKey.publicKeyBase64, isNotEmpty);
      expect(deviceKey.privateKeyBase64, isNotEmpty);
      expect(deviceKey.deviceId, startsWith('device_'));
    });

    test('public key is 32 bytes base64 encoded', () async {
      final deviceKey = await DeviceKey.generate();
      final rawKey = base64Decode(deviceKey.publicKeyBase64);
      expect(rawKey.length, 32);
    });
  });

  group('Crypto - Signer', () {
    test('signs and verifies correctly', () async {
      final deviceKey = await DeviceKey.generate();
      final signer = Signer(deviceKey.signingKey);

      final signingString = Signer.buildSigningString(
        messageId: 'msg_001',
        deviceId: deviceKey.deviceId,
        type: 'session.input',
        timestampMs: 1781000000000,
        nonce: 'X0Jz2N8cCj9YhWm4xQw=',
        seq: 1042,
        payloadSha256: '9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08',
      );

      final signature = signer.sign(signingString);
      expect(signature, isNotEmpty);

      final valid = Signer.verify(
        deviceKey.publicKeyBase64,
        signingString,
        signature,
      );
      expect(valid, isTrue);
    });

    test('tampered payload rejected', () async {
      final deviceKey = await DeviceKey.generate();
      final signer = Signer(deviceKey.signingKey);

      final originalString = Signer.buildSigningString(
        messageId: 'msg_001',
        deviceId: deviceKey.deviceId,
        type: 'session.input',
        timestampMs: 1781000000000,
        nonce: 'X0Jz2N8cCj9YhWm4xQw=',
        seq: 1042,
        payloadSha256: '9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08',
      );

      final signature = signer.sign(originalString);

      final tamperedString = Signer.buildSigningString(
        messageId: 'msg_001',
        deviceId: deviceKey.deviceId,
        type: 'session.input',
        timestampMs: 1781000000000,
        nonce: 'X0Jz2N8cCj9YhWm4xQw=',
        seq: 1042,
        payloadSha256: '0000000000000000000000000000000000000000000000000000000000000000',
      );

      final valid = Signer.verify(
        deviceKey.publicKeyBase64,
        tamperedString,
        signature,
      );
      expect(valid, isFalse);
    });

    test('bad signature rejected', () async {
      final deviceKey1 = await DeviceKey.generate();
      final deviceKey2 = await DeviceKey.generate();
      final signer2 = Signer(deviceKey2.signingKey);

      final signingString = Signer.buildSigningString(
        messageId: 'msg_001',
        deviceId: 'device_12345678',
        type: 'session.input',
        timestampMs: 1781000000000,
        nonce: 'X0Jz2N8cCj9YhWm4xQw=',
        seq: 1042,
        payloadSha256: '9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08',
      );

      final badSig = signer2.sign(signingString);

      final valid = Signer.verify(
        deviceKey1.publicKeyBase64,
        signingString,
        badSig,
      );
      expect(valid, isFalse);
    });

    test('computePayloadSha256 is deterministic', () {
      final hash1 = Signer.computePayloadSha256({
        'session_id': 'sess_001',
        'content': 'hello',
      });
      final hash2 = Signer.computePayloadSha256({
        'session_id': 'sess_001',
        'content': 'hello',
      });
      expect(hash1, hash2);
    });

    test('buildSigningString is consistent', () {
      final s1 = Signer.buildSigningString(
        messageId: 'msg_001',
        deviceId: 'phone_abc',
        type: 'session.input',
        timestampMs: 1781000000000,
        nonce: 'abc123',
        seq: 1,
        payloadSha256: 'deadbeef',
      );
      expect(s1, contains('v1'));
      expect(s1, contains('message_id=msg_001'));
      expect(s1, contains('device_id=phone_abc'));
    });
  });
}
