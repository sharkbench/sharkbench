class ElementService
  require 'net/http'
  require 'json'

  def self.fetch_element(symbol)
    uri = URI('http://web-data-source/element.json')
    response = Net::HTTP.get(uri)
    element_data = JSON.parse(response)

    element_data[symbol]
  end

  def self.fetch_shells(symbol)
    uri = URI('http://web-data-source/shells.json')
    response = Net::HTTP.get(uri)
    shells_data = JSON.parse(response)

    shells_data[symbol]
  end
end
