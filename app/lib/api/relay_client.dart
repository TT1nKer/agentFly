class RelayClient {
  final String baseUrl;

  RelayClient({required this.baseUrl});

  Future<bool> checkHealth() async {
    return true;
  }

  Future<void> connect() async {
  }

  Future<void> disconnect() async {
  }

  Future<void> sendMessage(Map<String, dynamic> message) async {
  }
}
