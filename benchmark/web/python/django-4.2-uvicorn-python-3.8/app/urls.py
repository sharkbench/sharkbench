from django.urls import path
from .views import get_element, get_shells

urlpatterns = [
    path('api/v1/periodic-table/element', get_element, name='get_element'),
    path('api/v1/periodic-table/shells', get_shells, name='get_shells'),
]
