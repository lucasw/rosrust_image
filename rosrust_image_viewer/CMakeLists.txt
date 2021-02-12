cmake_minimum_required(VERSION 3.0.2)
project(rosrust_image)

# add_compile_options(-std=c++17)

find_package(catkin REQUIRED COMPONENTS
  roscpp
  roslint
  sensor_msgs
  # std_msgs
)

# this requires a custom roslint version
roslint_rust()

find_package(Corrosion REQUIRED)

corrosion_import_crate(MANIFEST_PATH Cargo.toml)

catkin_package(
#  INCLUDE_DIRS include
#  LIBRARIES rosrust_image
  CATKIN_DEPENDS roscpp rospy std_msgs
#  DEPENDS system_lib
)

include_directories(
  ${catkin_INCLUDE_DIRS}
)

function(rust_target target_name)
  add_custom_target(${target_name}_copy_to_devel ALL
    COMMAND ${CMAKE_COMMAND} -E copy $<TARGET_FILE:${target_name}> ${CATKIN_DEVEL_PREFIX}/lib/${PROJECT_NAME}/${target_name}
  )
  add_dependencies(${target_name}_copy_to_devel ${target_name})
  corrosion_install(
    TARGETS ${target_name}
    RUNTIME DESTINATION ${CATKIN_PACKAGE_BIN_DESTINATION}
  )
endfunction(rust_target)

rust_target(image_viewer)
