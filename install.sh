#!/bin/bash
# install.sh — Zeta installer (redirects to zippy)
# Usage: curl -sSf https://raw.githubusercontent.com/murphsicles/zeta/main/install.sh | sh
# Or:   curl -sSf https://raw.githubusercontent.com/murphsicles/zippy/main/install.sh | sh

# Both URLs work. Forward to the zippy installer.
exec curl -sSf https://raw.githubusercontent.com/murphsicles/zippy/main/install.sh | sh
