cmake_minimum_required(VERSION 3.0.0)
project(molecular_orbital_viewer VERSION 0.1.0)
cmake_policy(SET CMP0072 NEW)

FIND_PACKAGE(OpenGL REQUIRED)

add_executable(molecular_orbital_viewer src/glad.c src/glad.h src/main.cpp)

target_link_libraries(molecular_orbital_viewer 
    glfw
    OpenGL::GL
    X11
    pthread
    Xrandr
    Xi
    dl
)
