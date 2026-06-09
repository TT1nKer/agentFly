import 'dart:convert';
import 'package:flutter_test/flutter_test.dart';
import 'package:agent_cockpit/crypto/device_key.dart';
import 'package:agent_cockpit/crypto/signer.dart';
import 'package:agent_cockpit/api/message_service.dart';
import 'package:agent_cockpit/api/upload_service.dart';
import 'package:agent_cockpit/storage/secure_key_store.dart';

void main() {
  group('UploadService', () {
    late UploadService uploadService;

    setUp(() async {
      final deviceKey = await DeviceKey.generate();
      final signer = Signer(deviceKey.signingKey);
      final keyStore = InMemoryKeyStore();
      final messageService = MessageService(deviceKey, signer, keyStore);
      uploadService = UploadService(messageService);
    });

    test('validates size limits', () {
      expect(uploadService.validateSize('text', 50 * 1024), isEmpty);
      expect(uploadService.validateSize('text', 200 * 1024), isNotEmpty);
      expect(uploadService.validateSize('url', 5 * 1024), isNotEmpty);
      expect(uploadService.validateSize('image', 6 * 1024 * 1024), isNotEmpty);
      expect(uploadService.validateSize('file', 11 * 1024 * 1024), isNotEmpty);
    });

    test('creates valid upload message', () async {
      final content = 'hello upload test';
      final contentB64 = base64Encode(utf8.encode(content));

      final msg = await uploadService.createUploadMessage(
        mediaType: 'text',
        filename: 'test.txt',
        contentBase64: contentB64,
        sessionId: 'sess_upload_001',
      );

      expect(msg['type'], 'file.upload');
      expect(msg['payload']['media_type'], 'text');
      expect(msg['payload']['filename'], 'test.txt');
      expect(msg['payload']['size_bytes'], content.length);
      expect(msg['signature'], isNotEmpty);
    });

    test('rejects oversized upload', () async {
      final bigContent = base64Encode(List.filled(200 * 1024, 65));

      expect(
        () => uploadService.createUploadMessage(
          mediaType: 'text',
          filename: 'big.txt',
          contentBase64: bigContent,
          sessionId: 'sess_001',
        ),
        throwsException,
      );
    });
  });
}
