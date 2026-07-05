cmake_minimum_required(VERSION 3.20)
project(fixed_decimal CXX)

add_library(fixed_decimal INTERFACE)
target_include_directories(fixed_decimal INTERFACE ${CMAKE_CURRENT_SOURCE_DIR}/include)
target_compile_features(fixed_decimal INTERFACE cxx_std_23)

if(PROJECT_IS_TOP_LEVEL)
  enable_testing()
  add_executable(fixed_decimal_tests tests/test_fixed_decimal.cpp)
  target_link_libraries(fixed_decimal_tests PRIVATE fixed_decimal)
  if(NOT MSVC)
    target_compile_options(fixed_decimal_tests PRIVATE -Wall -Wextra -Werror)
  endif()
  add_test(NAME unit COMMAND fixed_decimal_tests)
  add_test(NAME conformance
           COMMAND fixed_decimal_tests ${CMAKE_CURRENT_SOURCE_DIR}/../tests/vectors.csv)
endif()
