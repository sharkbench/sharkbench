package main

import (
	"net/http"

	"github.com/goccy/go-json"
	"github.com/labstack/echo/v4"
	"github.com/valyala/fasthttp"
)

var (
	client = &fasthttp.Client{}
)

func main() {
	e := echo.New()
	e.HideBanner = true
	e.HidePort = true
	e.JSONSerializer = goccyJSONSerializer{}

	e.GET("/api/v1/periodic-table/element", getElement)
	e.GET("/api/v1/periodic-table/shells", getShells)

	e.Start("0.0.0.0:3000")
}

type goccyJSONSerializer struct{}

func (goccyJSONSerializer) Serialize(c echo.Context, i interface{}, indent string) error {
	var (
		body []byte
		err  error
	)

	if indent == "" {
		body, err = json.Marshal(i)
	} else {
		body, err = json.MarshalIndent(i, "", indent)
	}
	if err != nil {
		return err
	}

	_, err = c.Response().Write(body)
	return err
}

func (goccyJSONSerializer) Deserialize(c echo.Context, i interface{}) error {
	return json.NewDecoder(c.Request().Body).Decode(i)
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

func getElement(c echo.Context) error {
	symbol := c.QueryParam("symbol")

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

	return c.JSON(http.StatusOK, element)
}

func getShells(c echo.Context) error {
	symbol := c.QueryParam("symbol")

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

	return c.JSON(http.StatusOK, ShellsResponse{Shells: shells})
}
