Rails.application.routes.draw do
  namespace :api do
    namespace :v1 do
      get 'periodic-table/element', to: 'periodic_table#element'
      get 'periodic-table/shells', to: 'periodic_table#shells'
    end
  end
end
