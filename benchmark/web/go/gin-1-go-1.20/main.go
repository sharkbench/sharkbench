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

func getElement(c *gin.Context) {
	symbol := c.Query("symbol")

	req := fasthttp.AcquireRequest()
	resp := fasthttp.AcquireResponse()
	defer fasthttp.ReleaseRequest(req) // Release request instance back to pool
	defer fasthttp.ReleaseResponse(resp) // Release response instance back to pool

	req.SetRequestURI("http://web-data-source/element.json")

	client.Do(req, resp)

	body := resp.Body()

	var json_data map[string]interface{}
	json.Unmarshal(body, &json_data)

	entry, _ := json_data[symbol].(map[string]interface{})

	c.JSON(200, gin.H{
		"name":   entry["name"],
		"number": entry["number"],
		"group":  entry["group"],
	})
}

func getShells(c *gin.Context) {
	symbol := c.Query("symbol")

	req := fasthttp.AcquireRequest()
	resp := fasthttp.AcquireResponse()
	defer fasthttp.ReleaseRequest(req) // Release request instance back to pool
	defer fasthttp.ReleaseResponse(resp) // Release response instance back to pool

	req.SetRequestURI("http://web-data-source/shells.json")

	client.Do(req, resp)

	body := resp.Body()

	var json_data map[string]interface{}
	json.Unmarshal(body, &json_data)

	shells, _ := json_data[symbol].([]interface{})

	c.JSON(200, gin.H{"shells": shells})
}
