package main

import (
	"encoding/json"
	"github.com/gin-gonic/gin"
	"io/ioutil"
	"net/http"
)

var (
	client = &http.Client{}
)

type ElementData map[string]struct {
	Name    string   `json:"name"`
	Number  int      `json:"number"`
	Group   int      `json:"group"`
	Shells  []int    `json:"shells"`
}

func fetchData(url string) ElementData {
	response, _ := client.Get(url)
	defer response.Body.Close()

	body, _ := ioutil.ReadAll(response.Body)

	var data ElementData
	json.Unmarshal(body, &data)
	return data
}

func getElements(c *gin.Context) {
	symbol := c.Query("symbol")
	data := fetchData("http://web-data-source/data.json")

	element := data[symbol]

	c.JSON(http.StatusOK, gin.H{
		"name":   element.Name,
		"number": element.Number,
		"group":  element.Group,
	})
}

func getShells(c *gin.Context) {
	symbol := c.Query("symbol")
	data := fetchData("http://web-data-source/data.json")

	element := data[symbol]

	c.JSON(http.StatusOK, gin.H{"shells": element.Shells})
}

func main() {
	gin.SetMode(gin.ReleaseMode)

	router := gin.New()

	router.GET("/api/v1/periodic-table/element", getElements)
	router.GET("/api/v1/periodic-table/shells", getShells)

	router.Run(":3000")
}
