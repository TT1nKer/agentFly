import 'package:flutter_test/flutter_test.dart';
import 'package:agent_cockpit/crypto/device_key.dart';
import 'package:agent_cockpit/crypto/signer.dart';
import 'package:agent_cockpit/api/message_service.dart';
import 'package:agent_cockpit/storage/secure_key_store.dart';

void main() {
  group('MessageService', () {
    late DeviceKey deviceKey;
    late Signer signer;
    late MessageService service;
    late InMemoryKeyStore keyStore;

    setUp(() async {
      deviceKey = await DeviceKey.generate();
      signer = Signer(deviceKey.signingKey);
      keyStore = InMemoryKeyStore();
      service = MessageService(deviceKey, signer, keyStore);
    });

    test('creates valid signed message', () async {
      final msg = await service.createSignedMessage(
        type: 'session.input',
        payload: {
          'session_id': 'sess_001',
          'content': 'hello world',
        },
      );

      expect(msg['version'], 1);
      expect(msg['type'], 'session.input');
      expect(msg['device_id'], deviceKey.deviceId);
      expect(msg['signature'], isNotEmpty);
      expect(msg['seq'], 1);

      final valid = Signer.verify(
        deviceKey.publicKeyBase64,
        Signer.buildSigningString(
          messageId: msg['message_id'],
          deviceId: msg['device_id'],
          type: msg['type'],
          timestampMs: msg['timestamp_ms'],
          nonce: msg['nonce'],
          seq: msg['seq'],
          payloadSha256: msg['payload_sha256'],
        ),
        msg['signature'],
      );
      expect(valid, isTrue);
    });

    test('seq increments', () async {
      final msg1 = await service.createSignedMessage(
        type: 'echo.ping',
        payload: {'echo': 'first'},
      );
      final msg2 = await service.createSignedMessage(
        type: 'echo.ping',
        payload: {'echo': 'second'},
      );

      expect(msg1['seq'], 1);
      expect(msg2['seq'], 2);
    });

    test('tampered payload fails verification', () async {
      final msg = await service.createSignedMessage(
        type: 'session.input',
        payload: {
          'session_id': 'sess_001',
          'content': 'safe',
        },
      );

      final tamperedPayload = {
        'session_id': 'sess_001',
        'content': 'malicious',
      };

      final tamperedHash = Signer.computePayloadSha256(tamperedPayload);
      final signingString = Signer.buildSigningString(
        messageId: msg['message_id'],
        deviceId: msg['device_id'],
        type: msg['type'],
        timestampMs: msg['timestamp_ms'],
        nonce: msg['nonce'],
        seq: msg['seq'],
        payloadSha256: tamperedHash,
      );

      final valid = Signer.verify(
        deviceKey.publicKeyBase64,
        signingString,
        msg['signature'],
      );
      expect(valid, isFalse);
    });

    test('different message_id each time', () async {
      final msg1 = await service.createSignedMessage(
        type: 'echo.ping',
        payload: {'echo': 'test'},
      );
      final msg2 = await service.createSignedMessage(
        type: 'echo.ping',
        payload: {'echo': 'test'},
      );

      expect(msg1['message_id'], isNot(msg2['message_id']));
    });
  });
}
