abstract class SecureKeyStore {
  Future<void> savePrivateKey(String privateKeyBase64);
  Future<String?> loadPrivateKey();
  Future<void> deletePrivateKey();
}

class InMemoryKeyStore implements SecureKeyStore {
  String? _key;

  @override
  Future<void> savePrivateKey(String privateKeyBase64) async {
    _key = privateKeyBase64;
  }

  @override
  Future<String?> loadPrivateKey() async {
    return _key;
  }

  @override
  Future<void> deletePrivateKey() async {
    _key = null;
  }
}
