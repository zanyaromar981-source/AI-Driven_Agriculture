/// Test switch. true = an empty code goes through on the code screen and
/// corners can be placed by tapping the map. The phone number is always
/// checked: the server sends real codes and refuses any other number.
/// Set to false before release.
const kTestMode = true;

/// The real server's address, set when building:
///   flutter build apk --release --dart-define=API_URL=https://api.example.com
/// Empty (the default) = the demo server inside the app (FakeApi).
const kApiUrl = String.fromEnvironment('API_URL');
