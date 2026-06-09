import 'dart:convert';
import 'package:crypto/crypto.dart' as crypto;
import 'package:pinenacl/ed25519.dart';

class Signer {
  final SigningKey _signingKey;

  Signer(this._signingKey);

  String sign(String signingString) {
    final message = Uint8List.fromList(utf8.encode(signingString));
    final signedMessage = _signingKey.sign(message);
    return base64Encode(signedMessage.signature.asTypedList);
  }

  String get publicKeyBase64 {
    return base64Encode(_signingKey.verifyKey.asTypedList);
  }

  static String buildSigningString({
    required String messageId,
    required String deviceId,
    required String type,
    required int timestampMs,
    required String nonce,
    required int seq,
    required String payloadSha256,
  }) {
    return 'v1\n'
        'message_id=$messageId\n'
        'device_id=$deviceId\n'
        'type=$type\n'
        'timestamp_ms=$timestampMs\n'
        'nonce=$nonce\n'
        'seq=$seq\n'
        'payload_sha256=$payloadSha256';
  }

  static String computePayloadSha256(Map<String, dynamic> payload) {
    final sorted = _sortJsonKeys(payload);
    final canonicalJson = jsonEncode(sorted);
    final bytes = utf8.encode(canonicalJson);
    return crypto.sha256.convert(bytes).toString();
  }

  static Map<String, dynamic> _sortJsonKeys(Map<String, dynamic> map) {
    final sortedKeys = map.keys.toList()..sort();
    final sorted = <String, dynamic>{};
    for (final key in sortedKeys) {
      final value = map[key];
      sorted[key] = value is Map<String, dynamic> ? _sortJsonKeys(value) : value;
    }
    return sorted;
  }

  static String generateNonce() {
    final random = List<int>.generate(16, (_) => DateTime.now().microsecondsSinceEpoch & 0xFF);
    return base64Encode(random);
  }

  static bool verify(String publicKeyB64, String signingString, String signatureB64) {
    try {
      final rawKey = base64Decode(publicKeyB64);
      final verifyKey = VerifyKey(Uint8List.fromList(rawKey));
      final rawSig = base64Decode(signatureB64);
      final signature = Signature(Uint8List.fromList(rawSig));
      final message = Uint8List.fromList(utf8.encode(signingString));
      return verifyKey.verify(signature: signature, message: message);
    } catch (e) {
      return false;
    }
  }
}
