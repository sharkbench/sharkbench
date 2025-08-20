#include "PeriodicTableController.h"

void v1::PeriodicTableController::getElement(
    const drogon::HttpRequestPtr &req,
    std::function<void(const drogon::HttpResponsePtr &)> &&callback
) const {
    // Get symbol from query parameter
    std::string symbol = req->getParameter("symbol");

    // Create a new request
    const auto httpReq = drogon::HttpRequest::newHttpRequest();
    httpReq->setPath("/element.json");

    // Send request asynchronously
    client_->sendRequest(httpReq, [callback, symbol](const drogon::ReqResult result, const drogon::HttpResponsePtr &response) {
        if (result != drogon::ReqResult::Ok) {
            const auto resp = drogon::HttpResponse::newHttpResponse();
            resp->setStatusCode(drogon::k500InternalServerError);
            callback(resp);
            return;
        }

        // Parse JSON response
        const auto jsonBody = response->getJsonObject();
        if (!jsonBody) {
            const auto resp = drogon::HttpResponse::newHttpResponse();
            resp->setStatusCode(drogon::k500InternalServerError);
            callback(resp);
            return;
        }

        // Create response
        Json::Value elementData = (*jsonBody)[symbol];
        Json::Value res;
        res["name"] = elementData["name"];
        res["number"] = elementData["number"];
        res["group"] = elementData["group"];
        auto resp = drogon::HttpResponse::newHttpJsonResponse(res);
        callback(resp);
    });
}

void v1::PeriodicTableController::getShells(
    const drogon::HttpRequestPtr &req,
    std::function<void(const drogon::HttpResponsePtr &)> &&callback
) const {
    // Get symbol from query parameter
    std::string symbol = req->getParameter("symbol");

    // Create a new request
    auto httpReq = drogon::HttpRequest::newHttpRequest();
    httpReq->setPath("/shells.json");

    // Send request asynchronously
    client_->sendRequest(httpReq, [callback, symbol](const drogon::ReqResult result, const drogon::HttpResponsePtr &response) {
        if (result != drogon::ReqResult::Ok) {
            const auto resp = drogon::HttpResponse::newHttpResponse();
            resp->setStatusCode(drogon::k500InternalServerError);
            callback(resp);
            return;
        }

        // Parse JSON response
        const auto jsonBody = response->getJsonObject();
        if (!jsonBody) {
            const auto resp = drogon::HttpResponse::newHttpResponse();
            resp->setStatusCode(drogon::k500InternalServerError);
            callback(resp);
            return;
        }

        // Create response
        Json::Value res;
        res["shells"] = (*jsonBody)[symbol];
        const auto resp = drogon::HttpResponse::newHttpJsonResponse(res);
        callback(resp);
    });
}
