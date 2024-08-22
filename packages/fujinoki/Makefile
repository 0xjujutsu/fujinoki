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

.PHONY: stage-release
stage-release:
	echo "Version: $(VERSION)"
	echo "Tag: $(TAG)"
	cat $(PKG_DIR)/../../version.txt
	git diff -- $(PKG_DIR)/../../version.txt
	git status
	@test "" = "`git cherry`" || (echo "Refusing to publish with unpushed commits" && false)

	# Stop if versions are not updated.
	@test "" != "`git diff -- $(PKG_DIR)/../../version.txt`" || (echo "Refusing to publish with unupdated version.txt" && false)

	# Prepare the packages.
	cd $(PKG_DIR) && pnpm version "$(VERSION)" --allow-same-version

	# TODO(kijv) uncommment
	# git checkout -b $(VERSION)
	# git commit -anm "publish $(VERSION) to registry"
	# git tag "$(VERSION)"
	# git push origin $(VERSION) --tags --force

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
