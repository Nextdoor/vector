#!/usr/bin/env bash
set -euo pipefail

# build-docker.sh
#
# SUMMARY
#
#   Builds the Vector docker images and optionally
#   pushes it to the Docker registry

set -x

CHANNEL="${CHANNEL:-"$(scripts/release-channel.sh)"}"
VERSION="${VECTOR_VERSION:-"$(scripts/version.sh)"}"
DATE="${DATE:-"$(date -u +%Y-%m-%d)"}"
PLATFORM="${PLATFORM:-}"
PUSH="${PUSH:-"true"}"
REPO="${REPO:-"nextdoor/vector"}"

#
# Functions
#

build() {
  local BASE="$1"
  local VERSION="$2"

  local TAG="$REPO:$VERSION-$BASE"
  local DOCKERFILE="distribution/docker/$BASE/Dockerfile"

  if [ -n "$PLATFORM" ]; then
    ARGS=()
    if [[ "$PUSH" == "true" ]]; then
      ARGS+=(--push)
    fi

    docker buildx build \
      --platform="$PLATFORM" \
      --tag "$TAG" \
      target/artifacts \
      -f "$DOCKERFILE" \
      "${ARGS[@]}"
  else
    docker build \
      --tag "$TAG" \
      target/artifacts \
      -f "$DOCKERFILE"

      if [[ "$PUSH" == "true" ]]; then
        docker push "$TAG"
      fi
  fi
}

#
# Build
#

echo "Building $REPO:* Docker images"

if [[ "$CHANNEL" == "latest" ]]; then
  VERSION_EXACT="$VERSION"
  # shellcheck disable=SC2001
  VERSION_MINOR_X=$(echo "$VERSION" | sed 's/\.[0-9]*$/.X/g')
  # shellcheck disable=SC2001
  VERSION_MAJOR_X=$(echo "$VERSION" | sed 's/\.[0-9]*\.[0-9]*$/.X/g')

  for VERSION_TAG in "$VERSION_EXACT" "$VERSION_MINOR_X" "$VERSION_MAJOR_X" latest; do
    build alpine "$VERSION_TAG"
    build debian "$VERSION_TAG"
    build distroless-static "$VERSION_TAG"
    build distroless-libc "$VERSION_TAG"
  done
elif [[ "$CHANNEL" == "nightly" ]]; then
  for VERSION_TAG in "nightly-$DATE" nightly; do
    build alpine "$VERSION_TAG"
    build debian "$VERSION_TAG"
    build distroless-static "$VERSION_TAG"
    build distroless-libc "$VERSION_TAG"
  done
elif [[ "$CHANNEL" == "test" ]]; then
  build "${BASE:-"alpine"}" "${TAG:-"test"}"
  build "${BASE:-"distroless-libc"}" "${TAG:-"test"}"
  build "${BASE:-"distroless-static"}" "${TAG:-"test"}"
fi
