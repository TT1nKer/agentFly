import 'package:flutter_secure_storage/flutter_secure_storage.dart';
import 'secure_key_store.dart';

class PlatformSecureKeyStore implements SecureKeyStore {
  final FlutterSecureStorage _storage = const FlutterSecureStorage();
  static const _keyName = 'device_private_key';

  @override
  Future<void> savePrivateKey(String privateKeyBase64) async {
    await _storage.write(key: _keyName, value: privateKeyBase64);
  }

  @override
  Future<String?> loadPrivateKey() async {
    return await _storage.read(key: _keyName);
  }

  @override
  Future<void> deletePrivateKey() async {
    await _storage.delete(key: _keyName);
  }
}
