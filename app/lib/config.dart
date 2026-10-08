/// Test switch. true = no input checks in sign-in (empty phone and empty code
/// go through) so the screens can be clicked through. Set to false before release.
const kTestMode = true;

/// The real server's address, set when building:
///   flutter build apk --release --dart-define=API_URL=https://api.example.com
/// Empty (the default) = the demo server inside the app (FakeApi).
const kApiUrl = String.fromEnvironment('API_URL');
