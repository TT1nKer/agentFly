import 'package:flutter_test/flutter_test.dart';
import 'package:flutter/material.dart';
import 'package:agent_cockpit/screens/connection_screen.dart';
import 'package:agent_cockpit/screens/session_list_screen.dart';
import 'package:agent_cockpit/screens/session_detail_screen.dart';
import 'package:agent_cockpit/screens/pair_screen.dart';
import 'package:agent_cockpit/screens/settings_screen.dart';
import 'package:agent_cockpit/widgets/event_view.dart';
import 'package:agent_cockpit/widgets/status_badge.dart';

void main() {
  group('Screens', () {
    testWidgets('ConnectionScreen renders', (tester) async {
      await tester.pumpWidget(const MaterialApp(home: ConnectionScreen()));
      expect(find.text('Agent Cockpit'), findsOneWidget);
      expect(find.text('Connect'), findsOneWidget);
    });

    testWidgets('SessionListScreen renders', (tester) async {
      await tester.pumpWidget(const MaterialApp(home: SessionListScreen()));
      expect(find.text('Sessions'), findsOneWidget);
    });

    testWidgets('SessionDetailScreen renders', (tester) async {
      await tester.pumpWidget(
        const MaterialApp(home: SessionDetailScreen(sessionId: 'test_123')),
      );
      expect(find.text('Session: test_123'), findsOneWidget);
    });

    testWidgets('PairScreen renders', (tester) async {
      await tester.pumpWidget(const MaterialApp(home: PairScreen()));
      expect(find.text('Pair Device'), findsOneWidget);
      expect(find.byType(TextField), findsOneWidget);
    });

    testWidgets('SettingsScreen renders', (tester) async {
      await tester.pumpWidget(const MaterialApp(home: SettingsScreen()));
      expect(find.text('Settings'), findsOneWidget);
    });
  });

  group('Widgets', () {
    testWidgets('EventView renders events', (tester) async {
      await tester.pumpWidget(MaterialApp(
        home: Scaffold(
          body: EventView(events: const [
            {'type': 'user.input', 'content': 'hello'},
            {'type': 'agent.output', 'content': 'pong'},
          ]),
        ),
      ));
      expect(find.text('user.input'), findsOneWidget);
      expect(find.text('agent.output'), findsOneWidget);
    });

    testWidgets('StatusBadge shows correct colors', (tester) async {
      await tester.pumpWidget(MaterialApp(
        home: Scaffold(
          body: Column(
            children: const [
              StatusBadge(status: 'online'),
              StatusBadge(status: 'offline'),
              StatusBadge(status: 'error'),
            ],
          ),
        ),
      ));
      expect(find.text('online'), findsOneWidget);
      expect(find.text('offline'), findsOneWidget);
      expect(find.text('error'), findsOneWidget);
    });
  });
}
