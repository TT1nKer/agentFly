import 'package:flutter/material.dart';
import 'package:agent_cockpit/provider.dart';
import 'package:agent_cockpit/app_state.dart';

class PairScreen extends StatefulWidget {
  const PairScreen({super.key});

  @override
  State<PairScreen> createState() => _PairScreenState();
}

class _PairScreenState extends State<PairScreen> {
  final _codeController = TextEditingController();
  bool _submitted = false;

  void _pair() {
    final code = _codeController.text.trim();
    if (code.length != 6) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('Pairing code must be 6 digits')),
      );
      return;
    }

    setState(() => _submitted = true);
    ChangeNotifierProvider.of<AppState>(context).sendPairingCode(code);
  }

  @override
  Widget build(BuildContext context) {
    final state = ChangeNotifierProvider.of<AppState>(context);

    if (!state.isPairing && _submitted && state.pairingError.isEmpty) {
      WidgetsBinding.instance.addPostFrameCallback((_) {
        if (mounted) Navigator.pop(context, true);
      });
    }

    return Scaffold(
      appBar: AppBar(title: const Text('Pair Device')),
      body: Padding(
        padding: const EdgeInsets.all(16.0),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            const Text('Enter the 6-digit code shown on your computer:'),
            const SizedBox(height: 8),
            Text('My device: ${state.deviceId}',
                style: Theme.of(context).textTheme.bodySmall),
            const SizedBox(height: 16),
            TextField(
              controller: _codeController,
              keyboardType: TextInputType.number,
              maxLength: 6,
              enabled: !state.isPairing,
              decoration: const InputDecoration(
                border: OutlineInputBorder(),
                hintText: '482913',
              ),
            ),
            const SizedBox(height: 16),
            if (state.isPairing)
              const Center(child: CircularProgressIndicator()),
            if (state.pairingError.isNotEmpty)
              Padding(
                padding: const EdgeInsets.only(bottom: 8),
                child: Text(state.pairingError,
                    style: const TextStyle(color: Colors.red)),
              ),
            ElevatedButton.icon(
              onPressed: state.isPairing ? null : _pair,
              icon: const Icon(Icons.link),
              label: const Text('Pair'),
            ),
          ],
        ),
      ),
    );
  }

  @override
  void dispose() {
    _codeController.dispose();
    super.dispose();
  }
}
