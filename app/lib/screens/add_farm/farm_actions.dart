import 'package:flutter/material.dart';

import '../../api/api.dart';
import '../../app_scope.dart';
import '../../store/local_store.dart';
import '../../store/outbox.dart';
import '../../theme.dart';
import '../../widgets/common.dart';
import 'corners_screen.dart';
import 'farm_edit.dart';

/// Open a farm in the editor (border, crops, name). Uses the server's copy, or
/// the copy Home keeps on the phone when there is no internet. Returns when the
/// editor closes; the caller should reload the farm.
Future<void> openFarmEditor(BuildContext context, String farmId) async {
  final scope = AppScope.read(context);
  Farm? farm;
  try {
    farm = await scope.api.getFarm(farmId);
  } on ApiException catch (e) {
    if (!e.isOffline) {
      if (context.mounted) showToast(context, '${scope.s.error} (${e.code})');
      return;
    }
    final j = await LocalStore.read('farm_$farmId');
    final f = j?['farm'];
    if (f is Map<String, dynamic>) farm = Farm.fromJson(f);
  }
  if (!context.mounted) return;
  final found = farm;
  if (found == null) {
    showToast(context, scope.s.noInternet);
    return;
  }
  await Navigator.of(context).push(
    MaterialPageRoute<void>(
      builder: (_) => CornersScreen(edit: FarmEdit.fromFarm(found)),
    ),
  );
}

/// Ask, then delete the farm: on the phone at once, on the server now or
/// when there is internet. Returns true when it was deleted.
Future<bool> confirmDeleteFarm(BuildContext context, FarmSummary farm) async {
  final scope = AppScope.read(context);
  final s = scope.s;
  final ku = scope.ku;
  final yes = await showDialog<bool>(
    context: context,
    builder: (ctx) => Directionality(
      textDirection: ku ? TextDirection.rtl : TextDirection.ltr,
      child: AlertDialog(
        title: Text(
          s.deleteTitle(farm.name),
          style: jText(ku, size: 18, weight: FontWeight.w700),
        ),
        content: Text(
          s.deleteBody,
          style: jText(ku, size: 14, color: JColors.muted, height: 1.6),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(ctx, false),
            child: Text(s.cancel, style: jText(ku, size: 14)),
          ),
          FilledButton(
            style: FilledButton.styleFrom(backgroundColor: JColors.levelAlarm),
            onPressed: () => Navigator.pop(ctx, true),
            child: Text(
              s.delete,
              style: jText(
                ku,
                size: 14,
                weight: FontWeight.w700,
                color: Colors.white,
              ),
            ),
          ),
        ],
      ),
    ),
  );
  if (yes != true || !context.mounted) return false;
  await Outbox.instance.delete(farm.id);
  await LocalStore.delete('farm_${farm.id}');
  await Outbox.instance.flush(scope.api);
  if (context.mounted) {
    showToast(
      context,
      Outbox.instance.deletes.contains(farm.id) ? s.deletedOffline : s.deleted,
    );
  }
  return true;
}

/// "Edit" and "Delete farm" for one farm, as a small menu button. [onEdited]
/// runs after the editor closes; [onDeleted] after a confirmed delete.
class FarmMenuButton extends StatelessWidget {
  const FarmMenuButton({
    super.key,
    required this.farm,
    required this.onEdited,
    required this.onDeleted,
  });

  final FarmSummary farm;
  final VoidCallback onEdited;
  final VoidCallback onDeleted;

  @override
  Widget build(BuildContext context) {
    final scope = AppScope.of(context);
    final s = scope.s;
    return PopupMenuButton<String>(
      tooltip: s.farmMenu,
      icon: const Icon(Icons.more_vert_rounded, color: JColors.ink),
      color: JColors.card,
      shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(14)),
      onSelected: (v) async {
        if (v == 'edit') {
          await openFarmEditor(context, farm.id);
          onEdited();
        } else if (await confirmDeleteFarm(context, farm)) {
          onDeleted();
        }
      },
      itemBuilder: (_) => [
        PopupMenuItem(
          value: 'edit',
          child: Row(
            textDirection: scope.ku ? TextDirection.rtl : TextDirection.ltr,
            spacing: 10,
            children: [
              const Icon(Icons.edit_outlined, size: 20, color: JColors.accent),
              Text(s.editFarm, style: jText(scope.ku, size: 15)),
            ],
          ),
        ),
        PopupMenuItem(
          value: 'delete',
          child: Row(
            textDirection: scope.ku ? TextDirection.rtl : TextDirection.ltr,
            spacing: 10,
            children: [
              const Icon(
                Icons.delete_outline_rounded,
                size: 20,
                color: JColors.levelAlarm,
              ),
              Text(
                s.deleteFarm,
                style: jText(scope.ku, size: 15, color: JColors.levelAlarm),
              ),
            ],
          ),
        ),
      ],
    );
  }
}
