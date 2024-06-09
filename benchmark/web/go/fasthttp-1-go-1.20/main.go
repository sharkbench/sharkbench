package main

import (
	"encoding/json"
	"github.com/valyala/fasthttp"
)

var (
	client = &fasthttp.Client{}
)

func main() {
	// Setting up FastHTTP router
	router := fasthttp.RequestHandler(func(ctx *fasthttp.RequestCtx) {
		switch string(ctx.Path()) {
		case "/api/v1/periodic-table/element":
			getElement(ctx)
		case "/api/v1/periodic-table/shells":
			getShells(ctx)
		default:
			ctx.Error("Unsupported path", fasthttp.StatusNotFound)
		}
	})

	fasthttp.ListenAndServe(":3000", router)
}

func getElement(ctx *fasthttp.RequestCtx) {
	symbol := string(ctx.QueryArgs().Peek("symbol"))

	req := fasthttp.AcquireRequest()
	resp := fasthttp.AcquireResponse()
	defer fasthttp.ReleaseRequest(req)
	defer fasthttp.ReleaseResponse(resp)

	req.SetRequestURI("http://web-data-source/element.json")

	client.Do(req, resp)

	body := resp.Body()

	var json_data map[string]interface{}
	json.Unmarshal(body, &json_data)

	entry, _ := json_data[symbol].(map[string]interface{})

	// Prepare response in JSON format
	response, _ := json.Marshal(map[string]interface{}{
		"name":   entry["name"],
		"number": entry["number"],
		"group":  entry["group"],
	})

	ctx.SetContentType("application/json")
	ctx.SetStatusCode(fasthttp.StatusOK)
	ctx.Write(response)
}

func getShells(ctx *fasthttp.RequestCtx) {
	symbol := string(ctx.QueryArgs().Peek("symbol"))

	req := fasthttp.AcquireRequest()
	resp := fasthttp.AcquireResponse()
	defer fasthttp.ReleaseRequest(req)
	defer fasthttp.ReleaseResponse(resp)

	req.SetRequestURI("http://web-data-source/shells.json")

	client.Do(req, resp)

	body := resp.Body()

	var json_data map[string]interface{}
	json.Unmarshal(body, &json_data)

	shells, _ := json_data[symbol].([]interface{})

	// Prepare response in JSON format
	response, _ := json.Marshal(map[string]interface{}{
		"shells": shells,
	})

	ctx.SetContentType("application/json")
	ctx.SetStatusCode(fasthttp.StatusOK)
	ctx.Write(response)
}
