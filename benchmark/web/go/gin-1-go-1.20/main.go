package main

import (
	"encoding/json"

	"github.com/gin-gonic/gin"
	"github.com/valyala/fasthttp"
)

var (
	client = &fasthttp.Client{}
)

func main() {
	gin.SetMode(gin.ReleaseMode)
	router := gin.New()

	router.GET("/api/v1/periodic-table/element", getElement)
	router.GET("/api/v1/periodic-table/shells", getShells)

	router.Run(":3000")
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

func getElement(c *gin.Context) {
	symbol := c.Query("symbol")

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

	c.JSON(200, element)
}

func getShells(c *gin.Context) {
	symbol := c.Query("symbol")

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

	c.JSON(200, ShellsResponse{Shells: shells})
}
