module Api
  module V1
    class PeriodicTableController < ActionController::API
      def element
        symbol = params[:symbol]
        element_data = ElementService.fetch_element(symbol)

        render json: {
          name: element_data["name"],
          number: element_data["number"],
          group: element_data["group"]
        }
      end

      def shells
        symbol = params[:symbol]
        shells_data = ElementService.fetch_shells(symbol)

        render json: {
          shells: shells_data
        }
      end
    end
  end
end
