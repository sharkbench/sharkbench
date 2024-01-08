package main

import (
    "github.com/gin-gonic/gin"
    "net/http"
    "encoding/json"
    "io/ioutil"
)

var (
	client = &http.Client{}
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

    resp, _ := client.Get("http://web-data-source/element.json")
    defer resp.Body.Close()

    body, _ := ioutil.ReadAll(resp.Body)

    var json_data map[string]interface{}
    json.Unmarshal(body, &json_data)

    entry, _ := json_data[symbol].(map[string]interface{})

    c.JSON(http.StatusOK, gin.H{
        "name":   entry["name"],
        "number": entry["number"],
        "group":  entry["group"],
    })
}

func getShells(c *gin.Context) {
    symbol := c.Query("symbol")

    resp, _ := client.Get("http://web-data-source/shells.json")
    defer resp.Body.Close()

    body, _ := ioutil.ReadAll(resp.Body)

    var json_data map[string]interface{}
    json.Unmarshal(body, &json_data)

    shells, _ := json_data[symbol].([]interface{})

    c.JSON(http.StatusOK, gin.H{"shells": shells})
}
