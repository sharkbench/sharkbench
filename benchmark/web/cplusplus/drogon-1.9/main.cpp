#include <drogon/drogon.h>
#include "PeriodicTableController.h"

int main() {
    drogon::app()
            .addListener("0.0.0.0", 3000)
            .setLogLevel(trantor::Logger::kDebug)
            .setThreadNum(16)
            .run();
    return 0;
}
