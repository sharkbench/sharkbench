cmake_minimum_required(VERSION 3.28)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_FLAGS "-O3 -Wall -Wextra")
set(CMAKE_CXX_FLAGS_DEBUG "-Og -g -Wall -Wextra")

find_package(Drogon CONFIG REQUIRED)

add_executable(drogon main.cpp
        PeriodicTableController.cpp
        PeriodicTableController.h)

target_link_libraries(drogon PRIVATE Drogon::Drogon)
