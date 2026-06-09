import 'dart:convert';
import 'package:pinenacl/ed25519.dart';

class DeviceKey {
  final SigningKey _signingKey;
  final String deviceId;

  DeviceKey._(this._signingKey, this.deviceId);

  static Future<DeviceKey> generate() async {
    return generateSync();
  }

  static DeviceKey generateSync() {
    final signingKey = SigningKey.generate();
    final random = DateTime.now().microsecondsSinceEpoch & 0xFFFFFFFF;
    final deviceId = 'device_${random.toRadixString(16).padLeft(8, '0')}';
    return DeviceKey._(signingKey, deviceId);
  }

  String get publicKeyBase64 {
    return base64Encode(_signingKey.verifyKey.asTypedList);
  }

  String get privateKeyBase64 {
    return base64Encode(_signingKey.asTypedList);
  }

  SigningKey get signingKey => _signingKey;

  VerifyKey get verifyKey => _signingKey.verifyKey;
}
