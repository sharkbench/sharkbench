class ElementService
  require 'net/http'
  require 'json'
  require 'connection_pool'

  @@connection_pool = ConnectionPool.new(size: 32, timeout: 5) do
    http = Net::HTTP.new('web-data-source', 80)
    http.keep_alive_timeout = 30
    http.start
    http
  end

  def self.fetch_element(symbol)
    @@connection_pool.with do |http|
      request = Net::HTTP::Get.new('/element.json')
      response = http.request(request)
      element_data = JSON.parse(response.body)
      element_data[symbol]
    end
  end

  def self.fetch_shells(symbol)
    @@connection_pool.with do |http|
      request = Net::HTTP::Get.new('/shells.json')
      response = http.request(request)
      shells_data = JSON.parse(response.body)
      shells_data[symbol]
    end
  end
end
