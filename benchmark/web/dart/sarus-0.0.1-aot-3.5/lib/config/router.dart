import 'package:sarus/sarus.dart';

import '../api/endpoints.dart';

final router = Router(
  routes: [
    Route(prefix: '/api/v1', endpoints: [PeriodicEndpoints()]),
  ],
);
