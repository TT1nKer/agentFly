import 'package:agent_cockpit/crypto/signer.dart';
import 'package:agent_cockpit/crypto/device_key.dart';
import 'package:agent_cockpit/storage/secure_key_store.dart';

class MessageService {
  final DeviceKey _deviceKey;
  final Signer _signer;
  int _seq = 0;
  final SecureKeyStore _keyStore;

  MessageService(this._deviceKey, this._signer, this._keyStore);

  String get deviceId => _deviceKey.deviceId;

  Future<Map<String, dynamic>> createSignedMessage({
    required String type,
    required Map<String, dynamic> payload,
  }) async {
    _seq++;

    final messageId = 'msg_${DateTime.now().millisecondsSinceEpoch}_${_seq}';
    final timestampMs = DateTime.now().millisecondsSinceEpoch;
    final nonce = Signer.generateNonce();
    final payloadSha256 = Signer.computePayloadSha256(payload);

    final signingString = Signer.buildSigningString(
      messageId: messageId,
      deviceId: deviceId,
      type: type,
      timestampMs: timestampMs,
      nonce: nonce,
      seq: _seq,
      payloadSha256: payloadSha256,
    );

    final signature = _signer.sign(signingString);

    return {
      'version': 1,
      'message_id': messageId,
      'device_id': deviceId,
      'type': type,
      'timestamp_ms': timestampMs,
      'nonce': nonce,
      'seq': _seq,
      'payload': payload,
      'payload_sha256': payloadSha256,
      'signature': signature,
    };
  }

  Future<void> saveState() async {
    await _keyStore.savePrivateKey(_deviceKey.privateKeyBase64);
  }
}
