import 'dart:async';
import 'dart:convert';
import 'package:agent_cockpit/api/message_service.dart';
import 'package:agent_cockpit/crypto/device_key.dart';
import 'package:agent_cockpit/crypto/signer.dart';
import 'package:agent_cockpit/storage/secure_key_store.dart';
import 'package:web_socket_channel/web_socket_channel.dart';

class WsClient {
  final String relayUrl;
  String _bridgeId = '';
  WebSocketChannel? _channel;
  MessageService? _messageService;
  bool _connected = false;
  int _reconnectAttempts = 0;

  final _eventController = StreamController<Map<String, dynamic>>.broadcast();
  final _statusController = StreamController<String>.broadcast();

  Stream<Map<String, dynamic>> get events => _eventController.stream;
  Stream<String> get status => _statusController.stream;
  bool get isConnected => _connected;
  String get bridgeId => _bridgeId;

  WsClient({
    required this.relayUrl,
    String? bridgeId,
  }) : _bridgeId = bridgeId ?? '';

  Future<void> init(DeviceKey deviceKey) async {
    final signer = Signer(deviceKey.signingKey);
    final keyStore = InMemoryKeyStore();
    _messageService = MessageService(deviceKey, signer, keyStore);
  }

  Future<void> connect() async {
    if (_messageService == null) return;

    final deviceId = _messageService!.deviceId;
    final wsUrl = Uri.parse(relayUrl).replace(
      queryParameters: {
        'device_id': deviceId,
        'device_type': 'phone',
      },
    );

    _statusController.add('connecting');

    _channel = WebSocketChannel.connect(wsUrl);
    _connected = true;
    _reconnectAttempts = 0;
    _statusController.add('connected');

    _channel!.stream.listen(
      (data) {
        try {
          final msg = jsonDecode(data as String) as Map<String, dynamic>;
          if (msg['type'] == 'bridge.info') {
            _bridgeId = msg['from'] ?? '';
          }
          _eventController.add(msg);
        } catch (_) {}
      },
      onError: (error) {
        _connected = false;
        _statusController.add('error: $error');
        _scheduleReconnect();
      },
      onDone: () {
        _connected = false;
        _statusController.add('disconnected');
        _scheduleReconnect();
      },
    );
  }

  void _scheduleReconnect() {
    if (_reconnectAttempts < 5) {
      _reconnectAttempts++;
      Future.delayed(
        Duration(seconds: 2 * _reconnectAttempts),
        () => connect(),
      );
    }
  }

  Future<void> sendMessage({
    required String type,
    required Map<String, dynamic> payload,
  }) async {
    if (_messageService == null) return;

    final signedMsg = await _messageService!.createSignedMessage(
      type: type,
      payload: payload,
    );

    signedMsg['to'] = _bridgeId;

    _channel?.sink.add(jsonEncode(signedMsg));
  }

  Future<void> disconnect() async {
    await _channel?.sink.close();
    _connected = false;
    _statusController.add('disconnected');
  }

  void dispose() {
    _eventController.close();
    _statusController.close();
    disconnect();
  }
}
