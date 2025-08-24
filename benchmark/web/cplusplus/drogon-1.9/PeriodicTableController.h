#pragma once

#include <drogon/HttpController.h>
#include <drogon/HttpClient.h>

namespace v1 {
    class PeriodicTableController : public drogon::HttpController<PeriodicTableController> {
    public:
        METHOD_LIST_BEGIN
            ADD_METHOD_TO(PeriodicTableController::getElement, "/api/v1/periodic-table/element", drogon::Get);
            ADD_METHOD_TO(PeriodicTableController::getShells, "/api/v1/periodic-table/shells", drogon::Get);
        METHOD_LIST_END

        void getElement(const drogon::HttpRequestPtr &req, std::function<void(const drogon::HttpResponsePtr &)> &&callback) const;

        void getShells(const drogon::HttpRequestPtr &req, std::function<void(const drogon::HttpResponsePtr &)> &&callback) const;

    private:
        drogon::HttpClientPtr client_ = drogon::HttpClient::newHttpClient("http://web-data-source");
    };
}
