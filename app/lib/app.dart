import 'package:flutter/material.dart';
import 'screens/connection_screen.dart';

class AgentCockpitApp extends StatelessWidget {
  const AgentCockpitApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Agent Cockpit',
      theme: ThemeData(
        colorScheme: ColorScheme.fromSeed(seedColor: Colors.blueGrey),
        useMaterial3: true,
      ),
      home: const ConnectionScreen(),
    );
  }
}
