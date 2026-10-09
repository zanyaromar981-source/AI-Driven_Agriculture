import 'api.dart';

/// The signed-in farmer (GET /me): what Settings shows.
class FarmerProfile {
  const FarmerProfile({required this.phone, this.name, this.lang = 'ku'});
  final String phone;
  final String? name;
  final String lang;

  factory FarmerProfile.fromJson(Map<String, dynamic> j) => FarmerProfile(
    phone: j['phone'] as String? ?? '',
    name: j['name'] as String?,
    lang: j['lang'] as String? ?? 'ku',
  );
}

/// One alert for a farm (BACKEND.md 2.7). `level` is `alarm` (red, pushed
/// to the phone) or `watch` (yellow, only in the app).
class FarmAlert {
  const FarmAlert({
    required this.id,
    required this.type,
    required this.level,
    required this.confidence,
    required this.en,
    this.ku = '',
    this.actionEn = '',
    this.actionKu = '',
    this.day,
    this.done = false,
  });
  final String id;
  final String type;
  final String level;
  final String confidence;
  final String en;
  final String ku;
  final String actionEn;
  final String actionKu;
  final DateTime? day;
  final bool done;

  bool get isAlarm => level == 'alarm';

  factory FarmAlert.fromJson(Map<String, dynamic> j) => FarmAlert(
    id: '${j['alert_id'] ?? ''}',
    type: j['type'] as String? ?? '',
    level: j['level'] == 'alarm' ? 'alarm' : 'watch',
    confidence: const ['sure', 'likely', 'unsure'].contains(j['confidence'])
        ? j['confidence'] as String
        : 'unsure',
    en: j['en'] as String? ?? '',
    ku: j['ku'] as String? ?? '',
    actionEn: j['action_en'] as String? ?? '',
    actionKu: j['action_ku'] as String? ?? '',
    day: DateTime.tryParse(j['day'] as String? ?? ''),
    done: j['done'] == true,
  );
}

/// A problem the farmer saw on a farm (Report screen). The server has no
/// reports route yet (FRONTEND.md 14), so it goes to the Ministry inbox as a
/// message of kind `report`, with the type and square written in the text.
class NewReport {
  const NewReport({
    required this.farmId,
    required this.type,
    this.square,
    this.note = '',
    this.photos = const [],
  });
  final String farmId;

  /// The farmer-facing name, e.g. "Yellow stripes".
  final String type;

  /// The square's label (e.g. "B7"), or null for the whole farm.
  final String? square;
  final String note;
  final List<DoctorPhoto> photos;

  /// The message text: "Yellow stripes, square B7: the note".
  String get text {
    final where = square == null ? '' : ', square $square';
    final n = note.trim();
    return n.isEmpty ? '$type$where' : '$type$where: $n';
  }
}

/// A message the farmer sent to the Ministry (GET /messages/mine).
class FarmerMessage {
  const FarmerMessage({
    required this.id,
    required this.kind,
    required this.text,
    required this.state,
    this.farmId,
    this.createdAt,
    this.replyEn,
    this.replyKu,
  });
  final String id;
  final String kind;
  final String text;

  /// `new`, `read`, `replied` or `closed`.
  final String state;
  final String? farmId;
  final DateTime? createdAt;
  final String? replyEn;
  final String? replyKu;

  factory FarmerMessage.fromJson(Map<String, dynamic> j) {
    final reply = j['reply'] as Map<String, dynamic>?;
    return FarmerMessage(
      id: '${j['id'] ?? ''}',
      kind: j['kind'] as String? ?? 'other',
      text: j['text'] as String? ?? '',
      state: j['state'] as String? ?? 'new',
      farmId: j['farm_id'] == null ? null : '${j['farm_id']}',
      createdAt: DateTime.tryParse(j['created_at'] as String? ?? ''),
      replyEn: reply?['text_en'] as String?,
      replyKu: reply?['text_ku'] as String?,
    );
  }
}
