# Package

version       = "0.1.0"
author        = "bitwool"
description   = "A simple site generator"
license       = "MIT"
srcDir        = "src"
bin           = @["ngen"]


# Dependencies

requires "nim >= 2.2.0"
requires "markdown"
requires "nimja"


# Tasks
task run, "Build and run ngen":
  exec "nimble build"
  exec "./ngen"