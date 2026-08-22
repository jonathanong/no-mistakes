import 'package:app/user.dart';
import 'package:http/http.dart' as http;

Future<void> loadUsers() async {
  await http.get(Uri.parse("/api/users"));
  User.list();
}
