import 'package:flutter/material.dart';

class ChangeNotifierProvider<T extends ChangeNotifier> extends StatefulWidget {
  final T Function(BuildContext) create;
  final Widget child;

  const ChangeNotifierProvider({
    super.key,
    required this.create,
    required this.child,
  });

  @override
  State<ChangeNotifierProvider<T>> createState() => _ChangeNotifierProviderState<T>();

  static T of<T extends ChangeNotifier>(BuildContext context) {
    final provider = context.findAncestorStateOfType<_ChangeNotifierProviderState<T>>();
    assert(provider != null, 'No ChangeNotifierProvider<$T> found');
    return provider!.value;
  }
}

class _ChangeNotifierProviderState<T extends ChangeNotifier> extends State<ChangeNotifierProvider<T>> {
  late T value;

  @override
  void initState() {
    super.initState();
    value = widget.create(context);
    value.addListener(_update);
  }

  void _update() => setState(() {});

  @override
  Widget build(BuildContext context) => widget.child;

  @override
  void dispose() {
    value.removeListener(_update);
    value.dispose();
    super.dispose();
  }
}
