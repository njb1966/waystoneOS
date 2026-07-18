cmake_minimum_required(VERSION 3.22)

project(WaystoneWorkspaceQt VERSION 0.1.0 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

find_package(Qt6 REQUIRED COMPONENTS Widgets)

add_executable(waystone-workspace
    src/cli_adapter.cpp
    src/main.cpp
    src/workspace_config.cpp
    src/workspace_pages.cpp
)

target_link_libraries(waystone-workspace PRIVATE Qt6::Widgets)

install(TARGETS waystone-workspace RUNTIME DESTINATION bin)
