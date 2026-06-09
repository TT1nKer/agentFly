class WsClient {
  final String url;

  WsClient({required this.url});

  Future<void> connect() async {}

  Future<void> send(String message) async {}

  void onMessage(void Function(String) handler) {}

  Future<void> close() async {}
}
