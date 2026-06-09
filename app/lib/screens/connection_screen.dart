import 'package:flutter/material.dart';
import 'package:agent_cockpit/provider.dart';
import 'package:agent_cockpit/app_state.dart';
import 'pair_screen.dart';
import 'session_list_screen.dart';

class ConnectionScreen extends StatefulWidget {
  const ConnectionScreen({super.key});

  @override
  State<ConnectionScreen> createState() => _ConnectionScreenState();
}

class _ConnectionScreenState extends State<ConnectionScreen> {
  final _urlController = TextEditingController(text: 'ws://10.0.2.2:8080');

  @override
  void initState() {
    super.initState();
    final state = ChangeNotifierProvider.of<AppState>(context);
    if (state.isConnected) {
      _navigateToSessions();
    }
  }

  void _connect() {
    final state = ChangeNotifierProvider.of<AppState>(context);
    state.connect(_urlController.text).then((_) {
      if (state.isConnected) {
        _navigateToSessions();
      }
    });
  }

  void _navigateToSessions() {
    Navigator.pushReplacement(
      context,
      MaterialPageRoute(builder: (_) => const SessionListScreen()),
    );
  }

  @override
  Widget build(BuildContext context) {
    final state = ChangeNotifierProvider.of<AppState>(context);

    return Scaffold(
      appBar: AppBar(title: const Text('Agent Cockpit')),
      body: Padding(
        padding: const EdgeInsets.all(16.0),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            Text('Device: ${state.deviceId}',
              style: Theme.of(context).textTheme.bodySmall),
            const SizedBox(height: 8),
            TextField(
              controller: _urlController,
              decoration: const InputDecoration(
                labelText: 'Relay URL',
                border: OutlineInputBorder(),
                hintText: 'ws://your-relay:8080',
              ),
            ),
            const SizedBox(height: 16),
            ElevatedButton.icon(
              onPressed: state.connectionStatus == 'connecting' ? null : _connect,
              icon: state.connectionStatus == 'connecting'
                  ? const SizedBox(width: 16, height: 16,
                      child: CircularProgressIndicator(strokeWidth: 2))
                  : const Icon(Icons.power),
              label: Text(state.connectionStatus == 'connecting' ? 'Connecting...' : 'Connect'),
            ),
            const SizedBox(height: 16),
            ElevatedButton.icon(
              onPressed: () {
                Navigator.push(
                  context,
                  MaterialPageRoute(builder: (_) => const PairScreen()),
                );
              },
              icon: const Icon(Icons.link),
              label: const Text('Pair with Bridge'),
            ),
            const Spacer(),
            const Text('v0.1.0', textAlign: TextAlign.center,
                style: TextStyle(color: Colors.grey, fontSize: 12)),
          ],
        ),
      ),
    );
  }

  @override
  void dispose() {
    _urlController.dispose();
    super.dispose();
  }
}
