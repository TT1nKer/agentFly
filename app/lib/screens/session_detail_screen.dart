import 'package:flutter/material.dart';
import 'package:agent_cockpit/provider.dart';
import 'package:agent_cockpit/app_state.dart';

class SessionDetailScreen extends StatefulWidget {
  final String sessionId;

  const SessionDetailScreen({super.key, required this.sessionId});

  @override
  State<SessionDetailScreen> createState() => _SessionDetailScreenState();
}

class _SessionDetailScreenState extends State<SessionDetailScreen> {
  final _inputController = TextEditingController();
  final _scrollController = ScrollController();

  void _sendInput() {
    final text = _inputController.text.trim();
    if (text.isEmpty) return;

    ChangeNotifierProvider.of<AppState>(context)
        .sendInput(widget.sessionId, text);
    _inputController.clear();
  }

  @override
  Widget build(BuildContext context) {
    final state = ChangeNotifierProvider.of<AppState>(context);
    final sessionEvents = state.events
        .where((e) => e.sessionId == widget.sessionId)
        .toList();

    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (_scrollController.hasClients) {
        _scrollController.jumpTo(_scrollController.position.maxScrollExtent);
      }
    });

    return Scaffold(
      appBar: AppBar(
        title: Text(widget.sessionId),
        actions: [
          IconButton(
            icon: const Icon(Icons.stop),
            onPressed: () async {
              await state.sendInput(widget.sessionId, '/exit');
              if (mounted) Navigator.pop(context);
            },
          ),
        ],
      ),
      body: Column(
        children: [
          Expanded(
            child: ListView.builder(
              controller: _scrollController,
              itemCount: sessionEvents.length,
              itemBuilder: (context, index) {
                final event = sessionEvents[index];
                final isUser = event.type == 'user.input';
                return Container(
                  padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 6),
                  margin: const EdgeInsets.symmetric(vertical: 2),
                  decoration: BoxDecoration(
                    color: isUser
                        ? Theme.of(context).colorScheme.primaryContainer.withValues(alpha: 0.3)
                        : null,
                  ),
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Row(
                        children: [
                          Text(event.type, style: const TextStyle(fontSize: 11, color: Colors.grey)),
                          const Spacer(),
                          Text('#${event.seq}', style: const TextStyle(fontSize: 11, color: Colors.grey)),
                        ],
                      ),
                      if (event.content != null && event.content!.isNotEmpty)
                        SelectableText(
                          event.content!,
                          style: TextStyle(
                            fontFamily: 'monospace',
                            fontSize: 12,
                            color: isUser ? Colors.blue[700] : null,
                          ),
                        ),
                    ],
                  ),
                );
              },
            ),
          ),
          const Divider(height: 1),
          SafeArea(
            child: Padding(
              padding: const EdgeInsets.all(8.0),
              child: Row(
                children: [
                  Expanded(
                    child: TextField(
                      controller: _inputController,
                      decoration: const InputDecoration(
                        hintText: 'Type your command...',
                        border: OutlineInputBorder(),
                        contentPadding: EdgeInsets.symmetric(horizontal: 12, vertical: 8),
                      ),
                      onSubmitted: (_) => _sendInput(),
                    ),
                  ),
                  const SizedBox(width: 8),
                  IconButton.filled(
                    onPressed: _sendInput,
                    icon: const Icon(Icons.send),
                  ),
                ],
              ),
            ),
          ),
        ],
      ),
    );
  }

  @override
  void dispose() {
    _inputController.dispose();
    _scrollController.dispose();
    super.dispose();
  }
}
