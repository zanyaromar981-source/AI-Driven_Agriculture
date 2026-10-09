import 'package:flutter/material.dart';
import 'package:image_picker/image_picker.dart';

import '../../api/api.dart';
import '../../app_scope.dart';
import '../../geo.dart';
import '../../theme.dart';
import '../../widgets/common.dart';
import '../doctor/ask_doctor_screen.dart';
import '../doctor/doctor_widgets.dart';
import '../home/farm_drawing.dart';

/// Photos per report: the inbox route takes 4 at most (FRONTEND.md 4).
const kMaxReportPhotos = 4;

/// Report a problem (Pencil: Screen/Report): what it is, which square, a
/// photo and a note, then the farmer's own reports. The server has no
/// reports route yet, so a report goes to the Ministry inbox as a message
/// of kind `report` (FRONTEND.md 4); its type and square are in the text.
class ReportScreen extends StatefulWidget {
  const ReportScreen({
    super.key,
    required this.farm,
    required this.shape,
    this.cell,
    this.picker,
  });
  final FarmSummary farm;
  final FarmShape shape;

  /// The square tapped on the farm, or null for the whole farm.
  final CellKey? cell;

  /// For tests.
  final ImagePicker? picker;

  @override
  State<ReportScreen> createState() => _ReportScreenState();
}

/// The problem types of the design, in its order, with their icons.
const reportTypes = <(String, IconData)>[
  ('Yellow stripes', Icons.eco_outlined),
  ('Insects', Icons.bug_report_outlined),
  ('Wilting', Icons.grass_rounded),
  ('Flood', Icons.waves_rounded),
  ('Hail', Icons.cloudy_snowing),
  ('Fire', Icons.local_fire_department_outlined),
  ('Animal disease', Icons.pets_rounded),
  ('Other', Icons.more_horiz_rounded),
];

class _ReportScreenState extends State<ReportScreen> {
  late final ImagePicker _picker = widget.picker ?? ImagePicker();
  final _note = TextEditingController();
  late CellKey? _cell = widget.cell;
  String? _type;
  final List<DoctorPhoto> _photos = [];
  bool _sending = false;
  List<FarmerMessage>? _mine;

  /// One key per report being written, so a retry is not sent twice.
  String _key = _newKey();

  static String _newKey() => 'report-${DateTime.now().microsecondsSinceEpoch}';

  @override
  void initState() {
    super.initState();
    _loadMine();
  }

  @override
  void dispose() {
    _note.dispose();
    super.dispose();
  }

  Future<void> _loadMine() async {
    try {
      final all = await AppScope.read(context).api.getMyMessages();
      if (mounted) {
        setState(() => _mine = all.where((m) => m.kind == 'report').toList());
      }
    } on ApiException {
      if (mounted) setState(() => _mine = const []);
    }
  }

  Future<void> _camera() async {
    if (_photos.length >= kMaxReportPhotos) return;
    XFile? f;
    try {
      f = await _picker.pickImage(
        source: ImageSource.camera,
        maxWidth: 1600,
        maxHeight: 1600,
        imageQuality: 80,
      );
    } catch (_) {
      if (mounted) showToast(context, 'Could not open the camera');
      return;
    }
    if (f == null) return;
    final bytes = await f.readAsBytes();
    if (!mounted) return;
    if (bytes.length > kMaxPhotoBytes) {
      showToast(context, 'The photo was too big (over 4 MB)');
      return;
    }
    final png = f.name.toLowerCase().endsWith('.png');
    setState(
      () => _photos.add(
        DoctorPhoto(
          bytes: bytes,
          mime: photoMime(bytes) ?? (png ? 'image/png' : 'image/jpeg'),
        ),
      ),
    );
  }

  Future<void> _send() async {
    final type = _type;
    if (type == null || _sending) return;
    FocusScope.of(context).unfocus();
    final api = AppScope.read(context).api;
    final k = _cell;
    setState(() => _sending = true);
    try {
      await api.sendReport(
        NewReport(
          farmId: widget.farm.id,
          type: type,
          square: k == null ? null : widget.shape.label(k),
          note: _note.text,
          photos: List.of(_photos),
        ),
        idempotencyKey: _key,
      );
      if (!mounted) return;
      showToast(context, 'Report sent. The office will see it.');
      setState(() {
        _type = null;
        _photos.clear();
        _note.clear();
        _key = _newKey();
      });
      _loadMine();
    } on ApiException catch (e) {
      if (!mounted) return;
      showToast(context, switch (e) {
        _ when e.isOffline =>
          'No internet. Your report is still here; send it when you have signal.',
        _ when e.code == 'rate_limited' =>
          'Too many reports today. Please try again tomorrow.',
        _ when e.code == 'bad_photo' =>
          'The photo could not be sent. Remove it and try again.',
        _ when e.code == 'not_found' =>
          'This farm is not on the server any more.',
        _ => 'Could not send the report. Please try again.',
      }, long: true);
    } finally {
      if (mounted) setState(() => _sending = false);
    }
  }

  @override
  Widget build(BuildContext context) {
    final k = _cell;
    final mine = _mine;
    Widget heading(String t) => Padding(
      padding: const EdgeInsets.only(bottom: 8),
      child: Text(t, style: latText(size: 13, weight: FontWeight.w700)),
    );
    return DoctorPage(
      backLabel: 'Back to the farm',
      children: [
        Text(
          'Report a problem',
          style: latText(
            size: 24,
            weight: FontWeight.w700,
            letterSpacing: -0.4,
          ),
        ),
        const SizedBox(height: 6),
        Text(
          'Only you and the Ministry see this. Your phone and exact spot are '
          'never shown to others.',
          style: latText(
            size: 15,
            weight: FontWeight.w400,
            color: JColors.muted,
            height: 1.45,
          ),
        ),
        const SizedBox(height: 16),
        heading('What is it?'),
        for (final row in [reportTypes.take(4), reportTypes.skip(4)]) ...[
          Row(
            spacing: 10,
            children: [
              for (final (name, icon) in row)
                Expanded(
                  child: _TypeChip(
                    name: name,
                    icon: icon,
                    on: _type == name,
                    onTap: () =>
                        setState(() => _type = _type == name ? null : name),
                  ),
                ),
            ],
          ),
          const SizedBox(height: 10),
        ],
        const SizedBox(height: 6),
        heading('Where?'),
        Container(
          padding: const EdgeInsets.fromLTRB(12, 16, 12, 12),
          decoration: BoxDecoration(
            color: JColors.card,
            borderRadius: BorderRadius.circular(14),
          ),
          child: Column(
            spacing: 10,
            children: [
              FarmDrawing(
                shape: widget.shape,
                view: FarmView.cells,
                farmLevel: FarmStatus.none,
                selectedCell: k,
                selectedCrop: null,
                onCell: (c) => setState(() => _cell = c),
                onCrop: (_) {},
                pin: true,
                styleButton: false,
              ),
              Text(
                k == null
                    ? 'Whole farm · tap a cell to choose one'
                    : 'Cell ${widget.shape.label(k)} · tap to change',
                style: latText(
                  size: 13,
                  weight: FontWeight.w500,
                  color: JColors.muted,
                ),
              ),
            ],
          ),
        ),
        const SizedBox(height: 16),
        heading('Photo'),
        PhotoRow(
          compact: true,
          photos: _photos,
          enabled: !_sending,
          max: kMaxReportPhotos,
          onCamera: _camera,
          onRemove: (i) => setState(() => _photos.removeAt(i)),
        ),
        const SizedBox(height: 16),
        heading('Note'),
        Container(
          constraints: const BoxConstraints(minHeight: 64),
          padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
          decoration: BoxDecoration(
            color: JColors.card,
            borderRadius: BorderRadius.circular(14),
            border: Border.all(color: JColors.line),
          ),
          child: TextField(
            controller: _note,
            enabled: !_sending,
            minLines: 1,
            maxLines: 4,
            maxLength: 1500,
            style: latText(size: 15, weight: FontWeight.w400, height: 1.45),
            decoration: InputDecoration(
              border: InputBorder.none,
              isDense: true,
              contentPadding: EdgeInsets.zero,
              counterText: '',
              hintText: 'Since Monday, lower leaves first',
              hintStyle: latText(
                size: 15,
                weight: FontWeight.w400,
                color: JColors.faint,
                height: 1.45,
              ),
            ),
          ),
        ),
        const SizedBox(height: 16),
        PrimaryButton(
          label: 'Send report',
          icon: Icons.send_rounded,
          iconFirst: true,
          loading: _sending,
          onPressed: _type == null ? null : _send,
        ),
        if (_type == null) ...[
          const SizedBox(height: 6),
          Center(
            child: Text(
              'Choose what it is first.',
              style: latText(size: 12, color: JColors.muted),
            ),
          ),
        ],
        const SizedBox(height: 24),
        Text(
          'MY REPORTS',
          style: latText(
            size: 12,
            weight: FontWeight.w700,
            color: JColors.muted,
            letterSpacing: 0.6,
          ),
        ),
        const SizedBox(height: 8),
        if (mine == null)
          const Center(
            child: Padding(
              padding: EdgeInsets.all(12),
              child: CircularProgressIndicator(color: JColors.accent),
            ),
          )
        else if (mine.isEmpty)
          Text(
            'No reports yet.',
            style: latText(size: 13, color: JColors.muted),
          )
        else
          for (final m in mine) ...[
            _MineRow(message: m),
            const SizedBox(height: 8),
          ],
      ],
    );
  }
}

class _TypeChip extends StatelessWidget {
  const _TypeChip({
    required this.name,
    required this.icon,
    required this.on,
    required this.onTap,
  });
  final String name;
  final IconData icon;
  final bool on;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) => Semantics(
    button: true,
    selected: on,
    label: name,
    excludeSemantics: true,
    child: InkWell(
      onTap: onTap,
      borderRadius: BorderRadius.circular(12),
      child: Container(
        height: 64,
        padding: const EdgeInsets.symmetric(horizontal: 4),
        decoration: BoxDecoration(
          color: on ? JColors.accentSoft : JColors.card,
          borderRadius: BorderRadius.circular(12),
          border: Border.all(
            color: on ? JColors.accent : JColors.line,
            width: on ? 2 : 1,
          ),
        ),
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          spacing: 5,
          children: [
            Icon(icon, size: 20, color: on ? JColors.accent : JColors.ink),
            Text(
              name,
              maxLines: 2,
              overflow: TextOverflow.ellipsis,
              textAlign: TextAlign.center,
              style: latText(
                size: 12,
                weight: on ? FontWeight.w700 : FontWeight.w600,
                color: on ? JColors.accent : JColors.ink,
                height: 1.15,
              ),
            ),
          ],
        ),
      ),
    ),
  );
}

const _months = [
  'Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', //
  'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec',
];

/// "5 Oct · Yellow stripes · Cell E12 · Seen by officer".
String reportLine(FarmerMessage m) {
  final d = m.createdAt?.toLocal();
  final what = m.text.split(':').first.split(', square ');
  return [
    if (d != null) '${d.day} ${_months[d.month - 1]}',
    what.first,
    if (what.length > 1) 'Cell ${what[1]}',
    m.state == 'new' ? 'Sent' : 'Seen by officer',
  ].join(' · ');
}

class _MineRow extends StatelessWidget {
  const _MineRow({required this.message});
  final FarmerMessage message;

  @override
  Widget build(BuildContext context) {
    final seen = message.state != 'new';
    final reply = message.replyEn ?? message.replyKu;
    return Container(
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: JColors.card,
        borderRadius: BorderRadius.circular(12),
      ),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        spacing: 10,
        children: [
          Icon(
            seen ? Icons.check_circle_outline_rounded : Icons.send_outlined,
            size: 18,
            color: seen ? JColors.levelNormal : JColors.muted,
          ),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              spacing: 4,
              children: [
                Text(
                  reportLine(message),
                  style: latText(
                    size: 13,
                    weight: FontWeight.w500,
                    height: 1.35,
                  ),
                ),
                if (reply != null && reply.isNotEmpty)
                  Text(
                    'Office: $reply',
                    style: latText(size: 13, color: JColors.muted),
                  ),
              ],
            ),
          ),
        ],
      ),
    );
  }
}
