import 'package:flutter_test/flutter_test.dart';
import 'package:jutyar/main.dart';

void main() {
  testWidgets('sign-in screen shows the brand', (tester) async {
    await tester.pumpWidget(const JutyarApp());
    expect(find.text('Jutyar'), findsWidgets);
  });
}
