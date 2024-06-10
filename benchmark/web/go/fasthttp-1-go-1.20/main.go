package main

import (
	"github.com/goccy/go-json"
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

type Element struct {
	Name   string `json:"name"`
	Number int    `json:"number"`
	Group  int    `json:"group"`
}

type ShellsResponse struct {
	Shells []int `json:"shells"`
}

type ElementsData map[string]Element

type ShellsData map[string][]int

func getElement(ctx *fasthttp.RequestCtx) {
    symbol := string(ctx.QueryArgs().Peek("symbol"))

    req := fasthttp.AcquireRequest()
    resp := fasthttp.AcquireResponse()
    defer fasthttp.ReleaseRequest(req)
    defer fasthttp.ReleaseResponse(resp)

    req.SetRequestURI("http://web-data-source/element.json")

    client.Do(req, resp)
    body := resp.Body()

    var elements ElementsData
    json.Unmarshal(body, &elements)

    element := elements[symbol]

    response, _ := json.Marshal(element)

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

    var shellsData ShellsData
    json.Unmarshal(body, &shellsData)

    shells := shellsData[symbol]

    response, _ := json.Marshal(ShellsResponse{Shells: shells})

    ctx.SetContentType("application/json")
    ctx.SetStatusCode(fasthttp.StatusOK)
    ctx.Write(response)
}
