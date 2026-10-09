import 'dart:typed_data';

import 'package:flutter/material.dart';
import 'package:image_picker/image_picker.dart';

import '../../api/api.dart';
import '../../app_scope.dart';
import '../../geo.dart';
import '../../theme.dart';
import '../../widgets/common.dart';
import 'doctor_answer_screen.dart';
import 'doctor_widgets.dart';

/// Ask the Doctor (Pencil: Screen/Ask). The farmer writes what they see and
/// adds up to 6 photos; the server reads the field, the weather and the
/// season first, so an answer takes about 30 seconds. English for now.
class AskDoctorScreen extends StatefulWidget {
  const AskDoctorScreen({
    super.key,
    required this.farm,
    this.cell,
    this.cellLabel,
    this.picker,
  });
  final FarmSummary farm;

  /// The square the farmer tapped before asking ("Ask about this spot").
  final CellKey? cell;
  final String? cellLabel;

  /// Replaced in tests.
  final ImagePicker? picker;

  @override
  State<AskDoctorScreen> createState() => _AskDoctorScreenState();
}

const kMaxPhotos = 6;
const kMaxPhotoBytes = 4 * 1024 * 1024;

class _AskDoctorScreenState extends State<AskDoctorScreen> {
  final _text = TextEditingController();
  final _photos = <DoctorPhoto>[];
  bool _sending = false;
  String? _error;
  late CellKey? _cell = widget.cell;

  late final ImagePicker _picker = widget.picker ?? ImagePicker();

  @override
  void initState() {
    super.initState();
    _text.addListener(() => setState(() {}));
  }

  @override
  void dispose() {
    _text.dispose();
    super.dispose();
  }

  bool get _empty => _text.text.trim().isEmpty && _photos.isEmpty;

  /// Photos are made small on the phone (1600 px, JPEG 80%), so 6 of them
  /// send quickly on a village connection.
  Future<void> _add(ImageSource source) async {
    final room = kMaxPhotos - _photos.length;
    if (room <= 0) return;
    List<XFile> picked;
    try {
      if (source == ImageSource.camera) {
        final one = await _picker.pickImage(
          source: source,
          maxWidth: 1600,
          maxHeight: 1600,
          imageQuality: 80,
        );
        picked = one == null ? const [] : [one];
      } else {
        picked = await _picker.pickMultiImage(
          maxWidth: 1600,
          maxHeight: 1600,
          imageQuality: 80,
          limit: room > 1 ? room : null,
        );
        if (room == 1 && picked.length > 1) picked = picked.take(1).toList();
      }
    } catch (_) {
      if (mounted) showToast(context, 'Could not open the camera or gallery');
      return;
    }
    var tooBig = 0;
    final added = <DoctorPhoto>[];
    for (final f in picked.take(room)) {
      final bytes = await f.readAsBytes();
      if (bytes.length > kMaxPhotoBytes) {
        tooBig++;
        continue;
      }
      final png = f.name.toLowerCase().endsWith('.png');
      added.add(
        DoctorPhoto(
          bytes: bytes,
          mime: photoMime(bytes) ?? (png ? 'image/png' : 'image/jpeg'),
        ),
      );
    }
    if (!mounted) return;
    setState(() => _photos.addAll(added));
    if (tooBig > 0) showToast(context, 'A photo was too big (over 4 MB)');
  }

  Future<void> _send() async {
    if (_empty || _sending) return;
    FocusScope.of(context).unfocus();
    final scope = AppScope.read(context);
    setState(() {
      _sending = true;
      _error = null;
    });
    try {
      final answer = await scope.api.askDoctor(
        widget.farm.id,
        DoctorQuestion(
          text: _text.text,
          photos: List.of(_photos),
          cellE: _cell?.e,
          cellN: _cell?.n,
          lang: scope.ku ? 'ku' : 'en',
        ),
      );
      if (!mounted) return;
      setState(() => _sending = false);
      await Navigator.of(context).push(
        MaterialPageRoute<void>(
          builder: (_) => DoctorAnswerScreen(
            farm: widget.farm,
            answer: answer,
            showKu: scope.ku,
          ),
        ),
      );
    } on ApiException catch (e) {
      if (!mounted) return;
      setState(() {
        _sending = false;
        _error = doctorErrorText(e);
      });
    }
  }

  @override
  Widget build(BuildContext context) {
    return DoctorPage(
      backLabel: widget.farm.name,
      children: [
        Text(
          'Ask the Doctor',
          style: latText(size: 26, weight: FontWeight.w800),
        ),
        const SizedBox(height: 4),
        Text(
          'Tell me what you see. I read your field from space, the 10-day weather and the season\'s rain.',
          style: latText(
            size: 13.5,
            weight: FontWeight.w500,
            color: JColors.muted,
            height: 1.4,
          ),
        ),
        const SizedBox(height: 14),
        _FarmChip(
          label: _cell != null && widget.cellLabel != null
              ? 'About square ${widget.cellLabel}'
              : 'About ${widget.farm.name}',
          onRemove: _cell == null || _sending
              ? null
              : () => setState(() => _cell = null),
        ),
        const SizedBox(height: 14),
        _QuestionBox(controller: _text, enabled: !_sending),
        const SizedBox(height: 16),
        Text(
          'Photos (up to $kMaxPhotos)',
          style: latText(size: 13, weight: FontWeight.w700),
        ),
        const SizedBox(height: 8),
        _PhotoRow(
          photos: _photos,
          enabled: !_sending,
          onCamera: () => _add(ImageSource.camera),
          onGallery: () => _add(ImageSource.gallery),
          onRemove: (i) => setState(() => _photos.removeAt(i)),
        ),
        const SizedBox(height: 18),
        const InfoNote(
          title: 'What the Doctor never does',
          body:
              'No pesticide or fertilizer doses, no product names. For those, ask the extension officer. Always check your field and your label.',
          icon: Icons.shield_outlined,
        ),
        const SizedBox(height: 18),
        if (_error != null) ...[
          DoctorNotice(text: _error!),
          const SizedBox(height: 12),
        ],
        if (_sending)
          const ReadingTheField()
        else
          PrimaryButton(
            label: 'Send to the Doctor',
            icon: Icons.send_rounded,
            onPressed: _empty ? null : _send,
          ),
      ],
    );
  }
}

/// What a failed ask means for the farmer, in plain words.
String doctorErrorText(ApiException e) => switch (e.code) {
  'offline' =>
    'No internet. Your question is still here; try again when you have signal.',
  'doctor_not_ready' =>
    'The Doctor is not switched on yet. Please try again later.',
  'bad_photo' =>
    'One of the photos could not be sent. Remove it and try again.',
  'empty_question' => 'Write a few words or add a photo first.',
  'not_found' =>
    'This farm is not on the server any more. Go back to My farms.',
  _ => 'The Doctor could not answer this time. Please try again.',
};

class _FarmChip extends StatelessWidget {
  const _FarmChip({required this.label, this.onRemove});
  final String label;

  /// Set when the question is about one square: asks about the whole farm.
  final VoidCallback? onRemove;

  @override
  Widget build(BuildContext context) => Align(
    alignment: Alignment.centerLeft,
    child: Container(
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 7),
      decoration: BoxDecoration(
        color: JColors.accentSoft,
        borderRadius: BorderRadius.circular(999),
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        spacing: 6,
        children: [
          const Icon(Icons.grass_rounded, size: 16, color: JColors.accent),
          Flexible(
            child: Text(
              label,
              maxLines: 1,
              overflow: TextOverflow.ellipsis,
              style: jText(
                true,
                size: 13,
                weight: FontWeight.w600,
                color: JColors.accent,
              ),
            ),
          ),
          if (onRemove != null)
            Semantics(
              button: true,
              label: 'Ask about the whole farm',
              excludeSemantics: true,
              child: InkWell(
                onTap: onRemove,
                customBorder: const CircleBorder(),
                child: const Padding(
                  padding: EdgeInsets.all(2),
                  child: Icon(
                    Icons.close_rounded,
                    size: 16,
                    color: JColors.accent,
                  ),
                ),
              ),
            ),
        ],
      ),
    ),
  );
}

class _QuestionBox extends StatelessWidget {
  const _QuestionBox({required this.controller, required this.enabled});
  final TextEditingController controller;
  final bool enabled;

  @override
  Widget build(BuildContext context) => Container(
    padding: const EdgeInsets.fromLTRB(14, 10, 14, 6),
    decoration: BoxDecoration(
      color: JColors.card,
      borderRadius: BorderRadius.circular(14),
      border: Border.all(color: JColors.line),
    ),
    child: Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          'What do you see?',
          style: latText(size: 13, weight: FontWeight.w700),
        ),
        TextField(
          controller: controller,
          enabled: enabled,
          minLines: 3,
          maxLines: 6,
          maxLength: 1000,
          textCapitalization: TextCapitalization.sentences,
          style: jText(true, size: 15, height: 1.4),
          decoration: InputDecoration(
            border: InputBorder.none,
            isDense: true,
            counterText: '',
            hintText: 'Yellow stripes on the lower leaves since Monday...',
            hintStyle: latText(size: 15, color: JColors.placeholder),
          ),
        ),
      ],
    ),
  );
}

class _PhotoRow extends StatelessWidget {
  const _PhotoRow({
    required this.photos,
    required this.enabled,
    required this.onCamera,
    required this.onGallery,
    required this.onRemove,
  });
  final List<DoctorPhoto> photos;
  final bool enabled;
  final VoidCallback onCamera;
  final VoidCallback onGallery;
  final void Function(int) onRemove;

  @override
  Widget build(BuildContext context) {
    final full = photos.length >= kMaxPhotos;
    return SizedBox(
      height: 84,
      child: ListView(
        scrollDirection: Axis.horizontal,
        children: [
          if (!full) ...[
            _AddTile(
              icon: Icons.photo_camera_outlined,
              label: 'Camera',
              onTap: enabled ? onCamera : null,
            ),
            const SizedBox(width: 10),
            _AddTile(
              icon: Icons.photo_library_outlined,
              label: 'Gallery',
              onTap: enabled ? onGallery : null,
            ),
          ],
          for (final (i, p) in photos.indexed) ...[
            const SizedBox(width: 10),
            _Thumb(
              photo: p,
              onRemove: enabled ? () => onRemove(i) : null,
              label: 'Photo ${i + 1}',
            ),
          ],
        ],
      ),
    );
  }
}

class _AddTile extends StatelessWidget {
  const _AddTile({required this.icon, required this.label, this.onTap});
  final IconData icon;
  final String label;
  final VoidCallback? onTap;

  @override
  Widget build(BuildContext context) => Semantics(
    button: true,
    label: label,
    excludeSemantics: true,
    child: InkWell(
      onTap: onTap,
      borderRadius: BorderRadius.circular(14),
      child: Container(
        width: 84,
        decoration: BoxDecoration(
          color: JColors.card,
          borderRadius: BorderRadius.circular(14),
          border: Border.all(color: JColors.accent.withValues(alpha: 0.45)),
        ),
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          spacing: 4,
          children: [
            Icon(icon, size: 24, color: JColors.accent),
            Text(
              label,
              style: latText(
                size: 12,
                weight: FontWeight.w600,
                color: JColors.accent,
              ),
            ),
          ],
        ),
      ),
    ),
  );
}

class _Thumb extends StatelessWidget {
  const _Thumb({required this.photo, required this.label, this.onRemove});
  final DoctorPhoto photo;
  final String label;
  final VoidCallback? onRemove;

  @override
  Widget build(BuildContext context) => SizedBox(
    width: 84,
    child: Stack(
      children: [
        Positioned.fill(
          child: ClipRRect(
            borderRadius: BorderRadius.circular(14),
            child: Image.memory(
              photo.bytes is Uint8List
                  ? photo.bytes as Uint8List
                  : Uint8List.fromList(photo.bytes),
              fit: BoxFit.cover,
              semanticLabel: label,
              cacheWidth: 200,
            ),
          ),
        ),
        Positioned(
          top: 4,
          right: 4,
          child: Semantics(
            button: true,
            label: 'Remove $label',
            excludeSemantics: true,
            child: InkWell(
              onTap: onRemove,
              customBorder: const CircleBorder(),
              child: Container(
                width: 26,
                height: 26,
                decoration: const BoxDecoration(
                  color: JColors.ink,
                  shape: BoxShape.circle,
                ),
                child: const Icon(
                  Icons.close_rounded,
                  size: 16,
                  color: Colors.white,
                ),
              ),
            ),
          ),
        ),
      ],
    ),
  );
}
