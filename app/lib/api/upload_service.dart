import 'dart:convert';
import 'package:agent_cockpit/api/message_service.dart';

class UploadService {
  static const maxTextSize = 100 * 1024;
  static const maxUrlSize = 4 * 1024;
  static const maxImageSize = 5 * 1024 * 1024;
  static const maxFileSize = 10 * 1024 * 1024;

  final MessageService _messageService;

  UploadService(this._messageService);

  String validateSize(String mediaType, int sizeBytes) {
    switch (mediaType) {
      case 'text':
        if (sizeBytes > maxTextSize) return 'Text exceeds 100KB limit';
        break;
      case 'url':
        if (sizeBytes > maxUrlSize) return 'URL exceeds 4KB limit';
        break;
      case 'image':
        if (sizeBytes > maxImageSize) return 'Image exceeds 5MB limit';
        break;
      case 'file':
        if (sizeBytes > maxFileSize) return 'File exceeds 10MB limit';
        break;
      default:
        return 'Unknown media type: $mediaType';
    }
    return '';
  }

  Future<Map<String, dynamic>> createUploadMessage({
    required String mediaType,
    required String filename,
    required String contentBase64,
    required String sessionId,
  }) async {
    final data = base64Decode(contentBase64);

    final sizeError = validateSize(mediaType, data.length);
    if (sizeError.isNotEmpty) {
      throw Exception(sizeError);
    }

    final msg = await _messageService.createSignedMessage(
      type: 'file.upload',
      payload: {
        'session_id': sessionId,
        'media_type': mediaType,
        'filename': filename,
        'content_sha256': _sha256(data),
        'size_bytes': data.length,
      },
    );

    msg['content_data'] = contentBase64;
    return msg;
  }

  String _sha256(List<int> data) {
    return base64Encode(data);
  }
}
