import 'dart:async';
import 'package:flutter/foundation.dart';
import 'package:agent_cockpit/crypto/device_key.dart';
import 'package:agent_cockpit/crypto/signer.dart';
import 'package:agent_cockpit/api/ws_client.dart';
import 'package:agent_cockpit/api/message_service.dart';
import 'package:agent_cockpit/api/models.dart';
import 'package:agent_cockpit/storage/platform_key_store.dart';

class AppState extends ChangeNotifier {
  DeviceKey? _deviceKey;
  MessageService? _messageService;
  WsClient? _wsClient;
  String _relayUrl = 'ws://localhost:8080';
  String _bridgeId = '';
  String _connectionStatus = 'disconnected';
  bool _isPairing = false;
  String _pairingError = '';

  final List<SessionModel> _sessions = [];
  final List<EventModel> _events = [];
  StreamSubscription? _eventSub;
  StreamSubscription? _statusSub;

  String get relayUrl => _relayUrl;
  String get bridgeId => _bridgeId;
  String get connectionStatus => _connectionStatus;
  String get deviceId => _deviceKey?.deviceId ?? '';
  bool get isConnected => _connectionStatus == 'connected';
  bool get isPairing => _isPairing;
  String get pairingError => _pairingError;
  List<SessionModel> get sessions => List.unmodifiable(_sessions);
  List<EventModel> get events => List.unmodifiable(_events);

  AppState() {
    _initKeys();
  }

  Future<void> _initKeys() async {
    _deviceKey = DeviceKey.generateSync();
    final signer = Signer(_deviceKey!.signingKey);
    _messageService = MessageService(_deviceKey!, signer, PlatformSecureKeyStore());
    notifyListeners();
  }

  Future<void> connect(String url) async {
    _relayUrl = url;
    _connectionStatus = 'connecting';
    notifyListeners();

    _wsClient = WsClient(relayUrl: url);
    await _wsClient!.init(_deviceKey!);

    _statusSub = _wsClient!.status.listen((s) {
      _connectionStatus = s;
      notifyListeners();
    });

    _eventSub = _wsClient!.events.listen(_handleEvent);

    await _wsClient!.connect();
  }

  void _handleEvent(Map<String, dynamic> msg) {
    final type = msg['type'] ?? '';

    if (type == 'session.list') {
      _sessions.clear();
      for (final s in (msg['sessions'] as List? ?? [])) {
        _sessions.add(SessionModel.fromJson(s));
      }
      notifyListeners();
    } else if (type == 'session.created') {
      _sessions.add(SessionModel(
        sessionId: msg['session_id'] ?? '',
        kind: msg['kind'] ?? 'shell',
        title: msg['title'] ?? '',
        status: 'created',
        createdAt: DateTime.now().toIso8601String(),
      ));
      notifyListeners();
    } else if (type == 'agent.output' || type == 'echo.pong') {
      _events.add(EventModel(
        eventId: 'evt_${DateTime.now().millisecondsSinceEpoch}',
        sessionId: msg['session_id'],
        seq: _events.length + 1,
        type: type,
        content: msg['content'] ?? msg.toString(),
        createdAt: DateTime.now().toIso8601String(),
      ));
      notifyListeners();
    } else if (type == 'bridge.info') {
      _bridgeId = msg['from'] ?? '';
      notifyListeners();
    } else if (type == 'paired') {
      _isPairing = false;
      _pairingError = '';
      _bridgeId = msg['from'] ?? '';
      notifyListeners();
    } else if (type == 'error' && msg['error'] == 'pairing_failed') {
      _isPairing = false;
      _pairingError = msg['detail'] ?? 'Pairing failed';
      notifyListeners();
    }
  }

  Future<void> sendPairingCode(String code) async {
    if (_messageService == null || _wsClient == null) return;
    _isPairing = true;
    _pairingError = '';
    notifyListeners();

    await _wsClient!.sendMessage(
      type: 'pairing.request',
      payload: {
        'pairing_code': code,
        'public_key': _deviceKey!.publicKeyBase64,
        'device_name': 'My Phone',
        'platform': defaultTargetPlatform.name,
      },
    );
  }

  Future<void> createSession(String kind) async {
    if (_wsClient == null) return;
    final sessionId = 'sess_${DateTime.now().millisecondsSinceEpoch}';
    await _wsClient!.sendMessage(
      type: 'session.create',
      payload: {
        'session_id': sessionId,
        'kind': kind,
        'workspace': '/tmp/agent_cockpit',
        'title': '$kind session',
      },
    );
  }

  Future<void> sendInput(String sessionId, String content) async {
    if (_wsClient == null) return;
    _events.add(EventModel(
      eventId: 'evt_${DateTime.now().millisecondsSinceEpoch}',
      sessionId: sessionId,
      seq: _events.length + 1,
      type: 'user.input',
      content: content,
      createdAt: DateTime.now().toIso8601String(),
    ));
    notifyListeners();

    await _wsClient!.sendMessage(
      type: 'session.input',
      payload: {
        'session_id': sessionId,
        'content': content,
      },
    );
  }

  Future<void> refreshSessions() async {
    if (_wsClient == null) return;
    await _wsClient!.sendMessage(
      type: 'session.list',
      payload: {'from': deviceId},
    );
  }

  @override
  void dispose() {
    _eventSub?.cancel();
    _statusSub?.cancel();
    _wsClient?.dispose();
    super.dispose();
  }
}
