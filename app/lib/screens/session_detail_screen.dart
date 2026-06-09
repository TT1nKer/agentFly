import 'package:flutter/material.dart';

class SessionDetailScreen extends StatelessWidget {
  final String sessionId;

  const SessionDetailScreen({super.key, required this.sessionId});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: Text('Session: $sessionId')),
      body: const Center(child: Text('Session output will appear here')),
    );
  }
}
