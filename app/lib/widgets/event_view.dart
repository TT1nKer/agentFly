import 'package:flutter/material.dart';

class EventView extends StatelessWidget {
  final List<Map<String, dynamic>> events;

  const EventView({super.key, required this.events});

  @override
  Widget build(BuildContext context) {
    return ListView.builder(
      itemCount: events.length,
      itemBuilder: (context, index) {
        final event = events[index];
        return ListTile(
          title: Text(event['type'] ?? ''),
          subtitle: Text(event['content'] ?? ''),
          dense: true,
        );
      },
    );
  }
}
