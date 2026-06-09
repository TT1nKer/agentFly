import 'package:flutter/material.dart';

class SettingsScreen extends StatelessWidget {
  const SettingsScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Settings')),
      body: ListView(
        children: const [
          ListTile(title: Text('Relay URL'), subtitle: Text('ws://localhost:8080/ws')),
          ListTile(title: Text('Device Name'), subtitle: Text('My Phone')),
          ListTile(title: Text('Version'), subtitle: Text('0.1.0')),
        ],
      ),
    );
  }
}
