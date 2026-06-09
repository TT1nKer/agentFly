class SessionModel {
  final String sessionId;
  final String kind;
  final String title;
  final String status;
  final String createdAt;

  SessionModel({
    required this.sessionId,
    required this.kind,
    required this.title,
    required this.status,
    required this.createdAt,
  });

  factory SessionModel.fromJson(Map<String, dynamic> json) {
    return SessionModel(
      sessionId: json['session_id'] ?? '',
      kind: json['kind'] ?? '',
      title: json['title'] ?? '',
      status: json['status'] ?? '',
      createdAt: json['created_at'] ?? '',
    );
  }
}

class EventModel {
  final String eventId;
  final String? sessionId;
  final int seq;
  final String type;
  final String? content;
  final String createdAt;

  EventModel({
    required this.eventId,
    this.sessionId,
    required this.seq,
    required this.type,
    this.content,
    required this.createdAt,
  });

  factory EventModel.fromJson(Map<String, dynamic> json) {
    return EventModel(
      eventId: json['event_id'] ?? '',
      sessionId: json['session_id'],
      seq: json['seq'] ?? 0,
      type: json['type'] ?? '',
      content: json['content'],
      createdAt: json['created_at'] ?? '',
    );
  }
}
