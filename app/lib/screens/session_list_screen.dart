import 'package:flutter/material.dart';
import 'package:agent_cockpit/provider.dart';
import 'package:agent_cockpit/app_state.dart';
import 'package:agent_cockpit/api/models.dart';
import 'session_detail_screen.dart';
import 'pair_screen.dart';

class SessionListScreen extends StatefulWidget {
  const SessionListScreen({super.key});

  @override
  State<SessionListScreen> createState() => _SessionListScreenState();
}

class _SessionListScreenState extends State<SessionListScreen> {
  @override
  void initState() {
    super.initState();
    Future.microtask(() {
      ChangeNotifierProvider.of<AppState>(context).refreshSessions();
    });
  }

  void _createSession(String kind) {
    ChangeNotifierProvider.of<AppState>(context).createSession(kind);
  }

  void _openSession(SessionModel session) {
    Navigator.push(
      context,
      MaterialPageRoute(
        builder: (_) => SessionDetailScreen(sessionId: session.sessionId),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final state = ChangeNotifierProvider.of<AppState>(context);

    return Scaffold(
      appBar: AppBar(
        title: const Text('Sessions'),
        actions: [
          IconButton(
            icon: const Icon(Icons.refresh),
            onPressed: () => state.refreshSessions(),
          ),
          IconButton(
            icon: const Icon(Icons.link),
            onPressed: () => Navigator.push(
              context,
              MaterialPageRoute(builder: (_) => const PairScreen()),
            ),
          ),
        ],
      ),
      body: Column(
        children: [
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
            color: Theme.of(context).colorScheme.surfaceContainerHighest,
            child: Row(
              children: [
                Text('Bridge: ${state.bridgeId.isNotEmpty ? state.bridgeId : "not paired"}'),
                const Spacer(),
                Container(
                  width: 8, height: 8,
                  decoration: BoxDecoration(
                    shape: BoxShape.circle,
                    color: state.isConnected ? Colors.green : Colors.red,
                  ),
                ),
                const SizedBox(width: 4),
                Text(state.connectionStatus, style: const TextStyle(fontSize: 12)),
              ],
            ),
          ),
          Expanded(
            child: state.sessions.isEmpty
                ? const Center(child: Text('No sessions. Create one below.'))
                : ListView.builder(
                    itemCount: state.sessions.length,
                    itemBuilder: (context, index) {
                      final session = state.sessions[index];
                      return ListTile(
                        leading: Icon(_iconForKind(session.kind)),
                        title: Text(session.title),
                        subtitle: Text(session.sessionId),
                        trailing: Text(session.status, style: const TextStyle(fontSize: 12)),
                        onTap: () => _openSession(session),
                      );
                    },
                  ),
          ),
        ],
      ),
      bottomNavigationBar: SafeArea(
        child: Padding(
          padding: const EdgeInsets.all(8.0),
          child: Row(
            mainAxisAlignment: MainAxisAlignment.spaceEvenly,
            children: [
              _kindButton('shell'),
              _kindButton('opencode'),
              _kindButton('hermes'),
            ],
          ),
        ),
      ),
    );
  }

  Widget _kindButton(String kind) {
    return ElevatedButton.icon(
      onPressed: () => _createSession(kind),
      icon: Icon(_iconForKind(kind), size: 18),
      label: Text(kind),
    );
  }

  IconData _iconForKind(String kind) {
    return switch (kind) {
      'shell' => Icons.terminal,
      'opencode' => Icons.code,
      'hermes' => Icons.psychology,
      'echo' => Icons.replay,
      _ => Icons.circle,
    };
  }
}
