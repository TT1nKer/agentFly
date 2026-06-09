import 'package:flutter/material.dart';

class OutputView extends StatelessWidget {
  final String output;

  const OutputView({super.key, required this.output});

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.all(8),
      child: SelectableText(
        output,
        style: const TextStyle(fontFamily: 'monospace', fontSize: 12),
      ),
    );
  }
}
