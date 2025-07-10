import 'package:sarus/sarus.dart';

import '../api/endpoints.dart';

final router = Router(
  globalMiddleware: [logRequests()],
  routes: [
    Route(prefix: '/api/v1', endpoints: [PeriodicEndpoints()]),
  ],
);
