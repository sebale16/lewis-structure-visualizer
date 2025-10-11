cmake_minimum_required( VERSION 3.20 )
project( display2 VERSION 0.1.0 LANGUAGES CXX C )
set(CMAKE_CXX_STANDARD 20)

add_executable( display2
    src/main.cpp
)

set( DAWN_FETCH_DEPENDENCIES ON )
set( DAWN_BUILD_MONOLITHIC_LIBRARY STATIC )
add_subdirectory( lib/dawn EXCLUDE_FROM_ALL )

target_link_libraries( display2 PRIVATE webgpu_dawn webgpu_glfw glfw )

set_target_properties( display2 PROPERTIES
    CXX_STANDARD_REQUIRED ON
    CXX_EXTENSIONS OFF
    COMPILE_WARNING_AS_ERROR ON
)

if (MSVC)
    target_compile_options( display2 PRIVATE /W4 )
else()
    target_compile_options( display2 PRIVATE -Wall -Wextra -pedantic )
endif()

if (XCODE)
    set_target_properties( display2 PROPERTIES
        XCODE_GENERATE_SCHEME ON
        XCODE_SCHEME_ENABLE_GPU_FRAME_CAPTURE_MODE "Metal"
    )
endif()

if (EMSCRIPTEN)
    set_target_properties( display2 PROPERTIES SUFFIX ".html" )
endif()