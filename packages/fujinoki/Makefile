# TODO(kijv) move this to workflow
VERSION = $(shell cat ../../version.txt | sed -n '1 p')
TAG = $(shell cat ../../version.txt | sed -n '2 p')
PKG_DIR = $(shell pwd)

EXT :=
ifeq ($(OS),Windows_NT)
	UNAME := Windows
	EXT = .exe
else
	UNAME := $(shell uname -s)
endif

.PHONY: publish
publish:
	echo "Version: $(VERSION)"
	echo "Tag: $(TAG)"

	# Include the patch in the log.
	git format-patch HEAD~1 --stdout | cat

	npm config set --location=project "//registry.npmjs.org/:_authToken" $(NPM_TOKEN)

ifneq ($(SKIP_PUBLISH),--skip-publish)
	# TODO(kijv) actually publish
	# cargo xtask npm publish --dry-run fujinoki-cli
endif
