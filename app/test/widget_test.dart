import 'package:flutter_test/flutter_test.dart';
import 'package:flutter/material.dart';
import 'package:agent_cockpit/provider.dart';
import 'package:agent_cockpit/app_state.dart';
import 'package:agent_cockpit/screens/connection_screen.dart';
import 'package:agent_cockpit/screens/session_list_screen.dart';
import 'package:agent_cockpit/screens/session_detail_screen.dart';
import 'package:agent_cockpit/screens/pair_screen.dart';
import 'package:agent_cockpit/screens/settings_screen.dart';

Widget wrapWithProvider(Widget child) {
  return ChangeNotifierProvider(
    create: (_) => AppState(),
    child: MaterialApp(home: child),
  );
}

void main() {
  group('Screens', () {
    testWidgets('ConnectionScreen renders', (tester) async {
      await tester.pumpWidget(wrapWithProvider(const ConnectionScreen()));
      await tester.pump();
      expect(find.text('Agent Cockpit'), findsOneWidget);
    });

    testWidgets('PairScreen renders', (tester) async {
      await tester.pumpWidget(wrapWithProvider(const PairScreen()));
      await tester.pump();
      expect(find.text('Pair Device'), findsOneWidget);
      expect(find.byType(TextField), findsOneWidget);
    });

    testWidgets('SessionListScreen renders', (tester) async {
      await tester.pumpWidget(wrapWithProvider(const SessionListScreen()));
      await tester.pump();
      expect(find.text('Sessions'), findsOneWidget);
    });

    testWidgets('SessionDetailScreen renders', (tester) async {
      await tester.pumpWidget(wrapWithProvider(
        const SessionDetailScreen(sessionId: 'test_123'),
      ));
      await tester.pump();
      expect(find.text('test_123'), findsOneWidget);
    });

    testWidgets('SettingsScreen renders', (tester) async {
      await tester.pumpWidget(MaterialApp(home: SettingsScreen()));
      expect(find.text('Settings'), findsOneWidget);
    });
  });
}
